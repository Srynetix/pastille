// SPDX-License-Identifier: MIT OR Apache-2.0

//! Reading and writing component values the way people write them.
//!
//! A beginner types `220`, a datasheet says `4.7k`, and the part itself is
//! marked `4k7`. All three mean the same resistor, and refusing any of them
//! teaches nothing except that the app is fussy.
//!
//! Parsing lives in the core rather than the app because the same grammar has to
//! serve the inspector (M-18), the save file (C-04) and capacitors and inductors
//! when they arrive (C-18) — three consumers, one definition.

/// Why a value could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueError {
    Empty,
    /// Nothing that looks like a number.
    NotANumber(String),
    /// A suffix that is not a known prefix or unit.
    UnknownSuffix(String),
}

impl core::fmt::Display for ValueError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ValueError::Empty => write!(f, "this needs a value"),
            ValueError::NotANumber(text) => write!(f, "{text:?} is not a number"),
            ValueError::UnknownSuffix(suffix) => {
                write!(f, "{suffix:?} is not a unit I know")
            }
        }
    }
}

impl core::error::Error for ValueError {}

/// Multiplier for an SI prefix.
///
/// Case matters and cannot be smoothed over: `1M` is a megohm and `1m` is a
/// milliohm, nine orders of magnitude apart. Accepting either for both would
/// silently change a circuit.
fn prefix(symbol: char) -> Option<f64> {
    Some(match symbol {
        'p' => 1e-12,
        'n' => 1e-9,
        'u' | 'µ' | 'μ' => 1e-6,
        'm' => 1e-3,
        'k' | 'K' => 1e3,
        'M' => 1e6,
        'G' => 1e9,
        _ => return None,
    })
}

/// Unit letters that carry no multiplier and can simply be dropped.
///
/// Lowercased through the full Unicode rules rather than the ASCII ones: `Ω`
/// and the ohm sign both fold to `ω`, and an ASCII-only fold leaves them
/// unrecognised.
fn is_bare_unit(rest: &str) -> bool {
    matches!(
        rest.to_lowercase().as_str(),
        "" | "r"
            | "ω"
            | "ohm"
            | "ohms"
            | "v"
            | "volt"
            | "volts"
            | "f"
            | "farad"
            | "h"
            | "henry"
            | "a"
            | "amp"
            | "amps"
            | "s"
    )
}

/// Whether a unit letter can stand in for the decimal point, as in `1R5`.
fn is_separator_unit(symbol: char) -> bool {
    is_bare_unit(&symbol.to_string())
}

/// Reads a written value into SI units.
///
/// ```text
/// 220      220        4.7k   4700      1M    1000000
/// 220R     220        4k7    4700      1m    0.001
/// ```
pub fn parse(text: &str) -> Result<f64, ValueError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(ValueError::Empty);
    }

    // Split off the leading number.
    let split = text
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == '+'))
        .unwrap_or(text.len());
    let (number, rest) = text.split_at(split);

    let magnitude: f64 = number
        .parse()
        .map_err(|_| ValueError::NotANumber(text.to_string()))?;

    let rest = rest.trim();
    if is_bare_unit(rest) {
        return Ok(magnitude);
    }

    let mut characters = rest.chars();
    let symbol = characters.next().expect("rest is not empty");
    // A unit letter in this position is a decimal point, not a multiplier:
    // `1R5` is 1.5 Ω, the way it is printed on the part.
    let multiplier = prefix(symbol)
        .or_else(|| is_separator_unit(symbol).then_some(1.0))
        .ok_or_else(|| ValueError::UnknownSuffix(rest.to_string()))?;
    let tail = characters.as_str().trim();

    // The prefix may stand where the decimal point would be — `4k7` is 4.7k,
    // which is how the value is printed on the part itself.
    if let Some(digit_count) = all_digits(tail) {
        let decimals: f64 = tail
            .parse()
            .map_err(|_| ValueError::NotANumber(text.to_string()))?;
        return Ok((magnitude + decimals / 10f64.powi(digit_count)) * multiplier);
    }

    if !is_bare_unit(tail) {
        return Err(ValueError::UnknownSuffix(rest.to_string()));
    }

    Ok(magnitude * multiplier)
}

/// Number of digits, if the whole string is digits.
fn all_digits(text: &str) -> Option<i32> {
    if text.is_empty() || !text.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(text.len() as i32)
}

/// Writes a value the way it would be spoken, with its unit.
///
/// The inverse of [`parse`] for everything [`parse`] produces, so a value
/// shown, edited and saved does not drift.
pub fn format(value: f64, unit: &str) -> String {
    const STEPS: [(f64, &str); 7] = [
        (1e9, "G"),
        (1e6, "M"),
        (1e3, "k"),
        (1.0, ""),
        (1e-3, "m"),
        (1e-6, "µ"),
        (1e-9, "n"),
    ];

    if value == 0.0 {
        return format!("0 {unit}").trim_end().to_string();
    }

    let magnitude = value.abs();
    for (scale, symbol) in STEPS {
        if magnitude >= scale {
            return format!("{} {symbol}{unit}", trim(value / scale))
                .trim_end()
                .to_string();
        }
    }

    format!("{} p{unit}", trim(value / 1e-12))
        .trim_end()
        .to_string()
}

/// Writes a value the way it should appear in an editable field: no unit and no
/// space, so `4.7 kΩ` reads back as `4.7k` — which is what a person would type.
pub fn format_editable(value: f64) -> String {
    format(value, "").replace(' ', "")
}

/// Drops the trailing zeroes a fixed format leaves behind.
fn trim(value: f64) -> String {
    let text = format!("{value:.3}");
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_three_forms_the_acceptance_names() {
        assert_eq!(parse("220").unwrap(), 220.0);
        assert_eq!(parse("220R").unwrap(), 220.0);
        assert_eq!(parse("1k").unwrap(), 1_000.0);
    }

    #[test]
    fn reads_a_decimal_prefix() {
        assert_eq!(parse("4.7k").unwrap(), 4_700.0);
        assert!((parse("0.1u").unwrap() - 1e-7).abs() < 1e-18);
    }

    #[test]
    fn reads_the_marking_printed_on_the_part() {
        // RKM notation: the prefix stands where the decimal point would be, so
        // a smudged dot cannot change the value by a thousand.
        assert_eq!(parse("4k7").unwrap(), 4_700.0);
        assert_eq!(parse("1R5").unwrap(), 1.5);
        assert_eq!(parse("2M2").unwrap(), 2_200_000.0);
    }

    #[test]
    fn keeps_mega_and_milli_apart() {
        // Nine orders of magnitude. Smoothing over the case would silently
        // change the circuit rather than refuse the input.
        assert_eq!(parse("1M").unwrap(), 1e6);
        assert_eq!(parse("1m").unwrap(), 1e-3);
    }

    #[test]
    fn ignores_the_unit_and_the_spaces_around_it() {
        for text in ["220 ohm", "220Ω", "220 R", " 220 ", "220 ohms"] {
            assert_eq!(parse(text).unwrap(), 220.0, "{text}");
        }
        assert_eq!(parse("9 V").unwrap(), 9.0);
        // 100 × 1e-9 is not bit-identical to 1e-7, and never will be.
        assert!((parse("100 nF").unwrap() - 1e-7).abs() < 1e-18);
    }

    #[test]
    fn reads_a_negative_value() {
        // A reversed battery is a real thing to type; whether it is sensible for
        // a given component is the caller's judgement, not the parser's.
        assert_eq!(parse("-9").unwrap(), -9.0);
    }

    #[test]
    fn refuses_what_it_cannot_read() {
        assert_eq!(parse("").unwrap_err(), ValueError::Empty);
        assert_eq!(parse("   ").unwrap_err(), ValueError::Empty);
        assert!(matches!(parse("abc"), Err(ValueError::NotANumber(_))));
        assert!(matches!(parse("1.2.3"), Err(ValueError::NotANumber(_))));
        assert!(matches!(
            parse("220 bananas"),
            Err(ValueError::UnknownSuffix(_))
        ));
        assert!(matches!(parse("10 zΩ"), Err(ValueError::UnknownSuffix(_))));
    }

    #[test]
    fn writes_values_the_way_they_are_spoken() {
        assert_eq!(format(220.0, "Ω"), "220 Ω");
        assert_eq!(format(4_700.0, "Ω"), "4.7 kΩ");
        assert_eq!(format(1e6, "Ω"), "1 MΩ");
        assert_eq!(format(1e-7, "F"), "100 nF");
        assert_eq!(format(9.0, "V"), "9 V");
        assert_eq!(format(0.0, "V"), "0 V");
    }

    #[test]
    fn an_editable_value_is_written_the_way_it_would_be_typed() {
        assert_eq!(format_editable(4_700.0), "4.7k");
        assert_eq!(format_editable(220.0), "220");
        assert_eq!(format_editable(1e6), "1M");
        assert_eq!(format_editable(1e-7), "100n");

        // And every one of them is something the parser accepts.
        for value in [4_700.0, 220.0, 1e6, 1e-7, 9.0] {
            let written = format_editable(value);
            assert!(parse(&written).is_ok(), "{written:?} does not parse");
        }
    }

    #[test]
    fn a_written_value_reads_back_as_itself() {
        // The round trip that keeps a value from drifting as it is shown,
        // edited and saved.
        for value in [220.0, 4_700.0, 1e6, 1e-7, 9.0, 0.047, 3.3] {
            let written = format(value, "");
            let read = parse(&written).unwrap_or_else(|error| panic!("{written:?}: {error}"));
            assert!(
                (read - value).abs() < value.abs() * 1e-9,
                "{value} wrote as {written:?} and read back as {read}"
            );
        }
    }
}
