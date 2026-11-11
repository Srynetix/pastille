# 4. Write the LU solver rather than take a linear algebra crate

- **Status:** accepted
- **Date:** 2026-09-17
- **Ticket:** M-07
- **Deciders:** Denis Bourge

## Context and problem statement

MNA produces a dense `A·x = z` that has to be solved every time the circuit changes and,
once M-09 lands, several times per timestep. The ticket asks explicitly for the choice to
be recorded: own implementation, or a vetted crate.

Two constraints shape it. The bundle is on the cold-load path — M-21 budgets three
seconds — so a large dependency is not free. And convergence work (C-11) is the declared
project-killer risk, which means the numerical layer is somewhere we will be reading,
instrumenting and changing, not somewhere we call and forget.

Circuit sizes are small. C-15 targets 50 nodes; CircuitJS1 only switches from dense to
sparse above 150. Nothing here is a linear-algebra problem in the sense those crates are
built for.

## Decision

**Own implementation**: Doolittle LU with partial pivoting, dense, row-major, about 90
lines in `crates/core/src/solver/lu.rs`.

The factorisation is a value (`Lu`) rather than a one-shot `solve`, because Newton
re-solves the same matrix against a moving right-hand side.

## Options considered

### `nalgebra` — rejected

Mature, well-tested, has exactly the factorisation needed. Rejected on weight: it pulls a
large generic tower into a wasm bundle to use one decomposition on matrices of a few
dozen rows, and its singularity handling would still need wrapping to produce our typed
error rather than an `Option`.

### `faer` — rejected

Faster and more modern, aimed at large dense problems and multi-threading. Both of those
advantages sit outside the range we operate in, and neither applies in a single-threaded
wasm bundle.

### Own implementation — chosen

Small enough to read in one sitting, no dependency on the cold-load path, and the error
behaviour is ours to define. The design doc already argues this for the solver as a whole
(§7.2): the numerical core is the part that teaches the domain, and it is the part we
will be debugging when C-11 gets difficult. A factorisation we can instrument beats one
we can only call.

The cost is real and accepted: correctness is now our problem. It is mitigated by tests
that check the residual ‖A·x − b‖∞ rather than only comparing against known answers, so
they stay meaningful for systems whose solution we have not worked out by hand.

## Consequences

1. **A singular matrix is a typed error, never NaN.** `SolveError::Singular { column }`
   names where factorisation gave up. This matters beyond tidiness: a NaN propagates
   silently into node voltages and the user ends up looking at a circuit showing `NaN V`
   with nothing to act on. C-25 maps this error to language a beginner can use.

2. **The singularity threshold is relative, not absolute.** A megohm network has
   conductances around 1e-6 and is perfectly solvable; a fixed absolute threshold would
   reject it. The pivot is compared against `1e-14 × ` the largest magnitude in the
   matrix, and there is a test that a 1e-6 system solves.

3. **Sparsity is not addressed.** Dense is O(n³), which is irrelevant at 50 nodes and
   will not be at 500. The switch point is a measurement for C-15, and the `Lu` type is
   the seam where a sparse implementation would slot in.

4. **No iterative refinement, no condition number estimate.** Both are things a
   production solver has. Revisit if C-11 shows convergence failures that turn out to be
   accuracy failures in disguise.
