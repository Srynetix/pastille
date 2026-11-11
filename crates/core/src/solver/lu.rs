// SPDX-License-Identifier: MIT OR Apache-2.0

//! Dense LU factorisation with partial pivoting.
//!
//! Doolittle with row interchanges, kept deliberately small (see
//! `docs/adrs/0004-own-lu-solver.md` for why this is not a crate). The
//! factorisation is a value of its own rather than a one-shot `solve`, because
//! Newton–Raphson (M-09) re-solves the same matrix against a new right-hand
//! side on every iteration where the matrix has not changed.

/// Why a system could not be solved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SolveError {
    /// No usable pivot in this column: the matrix is singular, or close enough
    /// that going on would produce numbers with no meaning.
    ///
    /// Returned instead of letting a division by a near-zero pivot fill the
    /// result with infinities and NaN, which is how a beginner ends up staring
    /// at a circuit showing `NaN V`.
    Singular { column: usize },
    /// The matrix already contained a non-finite entry before factorisation.
    NotFinite,
    /// Right-hand side length does not match the factorisation.
    DimensionMismatch { expected: usize, got: usize },
}

impl core::fmt::Display for SolveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SolveError::Singular { column } => {
                write!(f, "matrix is singular at column {column}")
            }
            SolveError::NotFinite => write!(f, "matrix contains a non-finite value"),
            SolveError::DimensionMismatch { expected, got } => {
                write!(f, "expected a right-hand side of {expected}, got {got}")
            }
        }
    }
}

impl core::error::Error for SolveError {}

/// How small a pivot may be before the matrix counts as singular, relative to
/// the largest magnitude in the original matrix.
///
/// Relative rather than absolute because circuits are stamped in SI units: a
/// megohm network has conductances around 1e-6 and is perfectly solvable, while
/// a pivot of 1e-6 in a matrix whose entries reach 1e3 is noise.
const SINGULARITY_TOLERANCE: f64 = 1e-14;

/// An LU factorisation, ready to be solved against any number of right-hand
/// sides.
#[derive(Debug, Clone, PartialEq)]
pub struct Lu {
    size: usize,
    /// L below the diagonal, U on and above it. L's unit diagonal is implicit.
    lu: Vec<f64>,
    /// Row `i` of the factorisation is row `permutation[i]` of the input.
    permutation: Vec<usize>,
}

impl Lu {
    /// Factorises a row-major square matrix.
    pub fn factor(matrix: &[f64], size: usize) -> Result<Lu, SolveError> {
        let mut lu = Lu {
            size: 0,
            lu: Vec::new(),
            permutation: Vec::new(),
        };
        lu.refactor(matrix, size)?;
        Ok(lu)
    }

    /// Factorises another matrix in the room this one already has.
    ///
    /// The same factorisation as [`Lu::factor`], without allocating: Newton
    /// factorises once an iteration, and a live 555 at audio frequency runs
    /// some 150 000 iterations a second (C-94).
    pub fn refactor(&mut self, matrix: &[f64], size: usize) -> Result<(), SolveError> {
        debug_assert_eq!(matrix.len(), size * size);

        let mut scale = 0.0_f64;
        for &value in matrix {
            if !value.is_finite() {
                return Err(SolveError::NotFinite);
            }
            scale = scale.max(value.abs());
        }
        // An all-zero matrix is singular in its first column, not a special case.
        let tolerance = SINGULARITY_TOLERANCE * scale.max(f64::MIN_POSITIVE);

        self.size = size;
        self.lu.clear();
        self.lu.extend_from_slice(matrix);
        self.permutation.clear();
        self.permutation.extend(0..size);
        let (lu, permutation) = (&mut self.lu, &mut self.permutation);

        for column in 0..size {
            // Partial pivoting: the largest magnitude in the column, at or below
            // the diagonal, becomes the pivot. Without this, a legitimate zero on
            // the diagonal — a node that happens to stamp nothing into its own
            // position — would look singular.
            let (pivot_row, pivot_value) = (column..size)
                .map(|row| (row, lu[row * size + column].abs()))
                .max_by(|a, b| a.1.total_cmp(&b.1))
                .expect("column is within the matrix");

            if pivot_value <= tolerance {
                return Err(SolveError::Singular { column });
            }

            if pivot_row != column {
                for k in 0..size {
                    lu.swap(column * size + k, pivot_row * size + k);
                }
                permutation.swap(column, pivot_row);
            }

            let pivot = lu[column * size + column];
            for row in (column + 1)..size {
                let factor = lu[row * size + column] / pivot;
                lu[row * size + column] = factor;
                if factor == 0.0 {
                    continue;
                }
                for k in (column + 1)..size {
                    lu[row * size + k] -= factor * lu[column * size + k];
                }
            }
        }

        Ok(())
    }

    pub fn size(&self) -> usize {
        self.size
    }

    /// Solves `A·x = rhs` using this factorisation.
    pub fn solve(&self, rhs: &[f64]) -> Result<Vec<f64>, SolveError> {
        let mut x = Vec::with_capacity(self.size);
        self.solve_into(rhs, &mut x)?;
        Ok(x)
    }

    /// Solves `A·x = rhs` into `x`, reusing its room, for the same reason as
    /// [`Lu::refactor`].
    pub fn solve_into(&self, rhs: &[f64], x: &mut Vec<f64>) -> Result<(), SolveError> {
        if rhs.len() != self.size {
            return Err(SolveError::DimensionMismatch {
                expected: self.size,
                got: rhs.len(),
            });
        }

        // Apply the row interchanges the factorisation made.
        x.clear();
        x.extend(self.permutation.iter().map(|&row| rhs[row]));

        // Forward substitution through L, whose diagonal is an implicit 1.
        for row in 1..self.size {
            let start = row * self.size;
            let below_diagonal = &self.lu[start..start + row];
            let sum: f64 = x[row]
                - below_diagonal
                    .iter()
                    .zip(&x[..row])
                    .map(|(l, solved)| l * solved)
                    .sum::<f64>();
            x[row] = sum;
        }

        // Back substitution through U.
        for row in (0..self.size).rev() {
            let start = row * self.size;
            let above_diagonal = &self.lu[start + row + 1..start + self.size];
            let sum: f64 = x[row]
                - above_diagonal
                    .iter()
                    .zip(&x[row + 1..])
                    .map(|(u, solved)| u * solved)
                    .sum::<f64>();
            x[row] = sum / self.lu[start + row];
        }

        Ok(())
    }
}

/// Factorises and solves in one step, for callers with a single right-hand side.
pub fn solve(matrix: &[f64], rhs: &[f64], size: usize) -> Result<Vec<f64>, SolveError> {
    Lu::factor(matrix, size)?.solve(rhs)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Largest absolute difference between two vectors.
    fn max_error(left: &[f64], right: &[f64]) -> f64 {
        left.iter()
            .zip(right)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0, f64::max)
    }

    /// Residual ‖A·x − b‖∞, the check that does not depend on knowing x.
    fn residual(matrix: &[f64], x: &[f64], rhs: &[f64], size: usize) -> f64 {
        (0..size)
            .map(|row| {
                let ax: f64 = (0..size).map(|col| matrix[row * size + col] * x[col]).sum();
                (ax - rhs[row]).abs()
            })
            .fold(0.0, f64::max)
    }

    #[test]
    fn solves_a_system_worked_out_by_hand() {
        // 2x + y = 5
        //  x + 3y = 10   →  x = 1, y = 3
        let a = [2.0, 1.0, 1.0, 3.0];
        let b = [5.0, 10.0];

        let x = solve(&a, &b, 2).unwrap();

        assert!(max_error(&x, &[1.0, 3.0]) < 1e-12);
    }

    #[test]
    fn pivots_past_a_zero_on_the_diagonal() {
        // Perfectly solvable, but unusable without row interchange.
        let a = [0.0, 1.0, 1.0, 0.0];
        let b = [2.0, 3.0];

        let x = solve(&a, &b, 2).unwrap();

        assert!(max_error(&x, &[3.0, 2.0]) < 1e-12);
    }

    #[test]
    fn a_singular_matrix_is_an_error_not_a_nan() {
        // The second row is twice the first: no unique solution.
        let a = [1.0, 2.0, 2.0, 4.0];
        let b = [3.0, 6.0];

        let error = solve(&a, &b, 2).unwrap_err();

        assert_eq!(error, SolveError::Singular { column: 1 });
    }

    #[test]
    fn an_all_zero_matrix_fails_on_its_first_column() {
        let a = [0.0; 4];
        let b = [1.0, 1.0];

        assert_eq!(
            solve(&a, &b, 2).unwrap_err(),
            SolveError::Singular { column: 0 }
        );
    }

    #[test]
    fn a_non_finite_entry_is_caught_before_factorising() {
        let a = [1.0, f64::INFINITY, 0.0, 1.0];
        let b = [1.0, 1.0];

        assert_eq!(solve(&a, &b, 2).unwrap_err(), SolveError::NotFinite);
    }

    #[test]
    fn small_conductances_are_not_mistaken_for_singularity() {
        // A megohm network: every entry is around 1e-6 and the system is fine.
        // An absolute singularity threshold would reject this.
        let g = 1e-6;
        let a = [g, -g, -g, 2.0 * g];
        let b = [1e-3, 0.0];

        let x = solve(&a, &b, 2).unwrap();

        assert!(residual(&a, &x, &b, 2) < 1e-15);
    }

    #[test]
    fn one_factorisation_serves_several_right_hand_sides() {
        // What Newton–Raphson needs in M-09: the matrix holds still while the
        // right-hand side moves.
        let a = [4.0, 3.0, 6.0, 3.0];
        let lu = Lu::factor(&a, 2).unwrap();

        let first = lu.solve(&[10.0, 12.0]).unwrap();
        let second = lu.solve(&[1.0, 0.0]).unwrap();

        assert!(residual(&a, &first, &[10.0, 12.0], 2) < 1e-12);
        assert!(residual(&a, &second, &[1.0, 0.0], 2) < 1e-12);
    }

    #[test]
    fn a_mismatched_right_hand_side_is_refused() {
        let lu = Lu::factor(&[1.0, 0.0, 0.0, 1.0], 2).unwrap();

        assert_eq!(
            lu.solve(&[1.0]).unwrap_err(),
            SolveError::DimensionMismatch {
                expected: 2,
                got: 1
            }
        );
    }

    #[test]
    fn solves_a_larger_system_accurately() {
        // Diagonally dominant 5x5, the shape a resistor network produces.
        let size = 5;
        let mut a = vec![0.0; size * size];
        for row in 0..size {
            for col in 0..size {
                a[row * size + col] = if row == col {
                    4.0
                } else if row.abs_diff(col) == 1 {
                    -1.0
                } else {
                    0.0
                };
            }
        }
        let b: Vec<f64> = (0..size).map(|i| i as f64).collect();

        let x = solve(&a, &b, size).unwrap();

        assert!(residual(&a, &x, &b, size) < 1e-12);
    }

    #[test]
    fn a_one_by_one_system_still_works() {
        assert!(max_error(&solve(&[2.0], &[6.0], 1).unwrap(), &[3.0]) < 1e-12);
        assert_eq!(
            solve(&[0.0], &[6.0], 1).unwrap_err(),
            SolveError::Singular { column: 0 }
        );
    }
}
