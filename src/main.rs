use clap::{ArgGroup, Parser};

/// Convert DNA base-pair counts to readable units.
#[derive(Parser, Debug)]
#[command(
    name = "dnacalc",
    version,
    about = "Convert DNA base-pair counts to readable units",
    group(
        ArgGroup::new("unit")
            .args(["kbp", "mbp", "gbp", "tbp", "pbp"])
            .multiple(false)
    )
)]
struct Cli {
    /// Force kilobase pairs (Kbp).
    #[arg(short = 'k', short_alias = 'K', group = "unit")]
    kbp: bool,

    /// Force megabase pairs (Mbp).
    #[arg(short = 'm', short_alias = 'M', group = "unit")]
    mbp: bool,

    /// Force gigabase pairs (Gbp).
    #[arg(short = 'g', short_alias = 'G', group = "unit")]
    gbp: bool,

    /// Force terabase pairs (Tbp).
    #[arg(short = 't', short_alias = 'T', group = "unit")]
    tbp: bool,

    /// Force petabase pairs (Pbp).
    #[arg(short = 'p', short_alias = 'P', group = "unit")]
    pbp: bool,
    
    /// Remove the space between the number and unit (e.g. 100Kbp).
    #[arg(short = 'n', long = "no-space")]
    no_space: bool,

    /// Non-negative base-pair count. Grouping separators are supported.
    #[arg(value_name = "BASE_PAIRS")]
    bases: String,
}

/// Parse CLI arguments using Clap's derive-generated parser.
fn parse_cli() -> Cli {
    Cli::parse()
}

fn run() -> Result<(), String> {
    let cli = parse_cli();

    let forced_unit = if cli.kbp {
        Some('k')
    } else if cli.mbp {
        Some('m')
    } else if cli.gbp {
        Some('g')
    } else if cli.tbp {
        Some('t')
    } else if cli.pbp {
        Some('p')
    } else {
        None
    };

    let result = format_dna_length_with_unit(&cli.bases, forced_unit)?;
    println!("{}", format_output(&result, cli.no_space));

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

const UNITS: [(u128, &str); 6] = [
    (1_000_000_000_000_000, "Pbp"),
    (1_000_000_000_000, "Tbp"),
    (1_000_000_000, "Gbp"),
    (1_000_000, "Mbp"),
    (1_000, "Kbp"),
    (1, "bp"),
];

fn invalid_input(input: &str) -> String {
    format!("Invalid DNA length: {input}")
}

fn pow10(exp: u32) -> u128 {
    10_u128.pow(exp)
}

// Parse a non-negative integer, optionally using consistent
// thousands separators: commas, dots, or underscores.
fn parse_bases(input: &str) -> Result<u128, String> {
    let invalid = || invalid_input(input);

    if input.is_empty() {
        return Err(invalid());
    }

    if input.bytes().all(|b| b.is_ascii_digit()) {
        return input.parse::<u128>().map_err(|_| invalid());
    }

    let mut separator: Option<char> = None;

    for c in input.chars() {
        if matches!(c, ',' | '.' | '_') {
            match separator {
                None => separator = Some(c),
                Some(previous) if previous == c => {}
                Some(_) => return Err(invalid()),
            }
        }
    }

    let separator = separator.ok_or_else(invalid)?;
    let groups: Vec<&str> = input.split(separator).collect();

    if groups[0].is_empty()
        || groups[0].len() > 3
        || !groups[0].bytes().all(|b| b.is_ascii_digit())
    {
        return Err(invalid());
    }

    for group in &groups[1..] {
        if group.len() != 3 || !group.bytes().all(|b| b.is_ascii_digit()) {
            return Err(invalid());
        }
    }

    groups.concat().parse::<u128>().map_err(|_| invalid())
}

// Find floor(log10(bases / divisor)) using integer arithmetic.
fn decimal_exponent(bases: u128, divisor: u128) -> i32 {
    if bases == 0 {
        return 0;
    }

    let mut exponent =
        bases.to_string().len() as i32 - divisor.to_string().len() as i32;

    if exponent >= 0 {
        if bases < divisor * pow10(exponent as u32) {
            exponent -= 1;
        }
    } else if bases * pow10((-exponent) as u32) < divisor {
        exponent -= 1;
    }

    exponent
}

// Format a rational value to three significant digits.
// Uses integer arithmetic to avoid f64 precision loss.
fn format_value(bases: u128, divisor: u128, suffix: &str) -> String {
    if bases == 0 {
        return format!("0 {suffix}");
    }

    let exponent = decimal_exponent(bases, divisor);
    let decimal_places = 2 - exponent;

    let formatted = if decimal_places >= 0 {
        let scale = pow10(decimal_places as u32);
        let numerator = bases * scale;

        let mut rounded = numerator / divisor;
        let remainder = numerator % divisor;

        // Round half up without floating-point arithmetic.
        if remainder >= divisor / 2 + divisor % 2 {
            rounded += 1;
        }

        let mut digits = rounded.to_string();

        if decimal_places > 0 {
            let places = decimal_places as usize;

            if digits.len() <= places {
                digits = format!("{}{}", "0".repeat(places + 1 - digits.len()), digits);
            }

            let split = digits.len() - places;
            digits.insert(split, '.');

            while digits.ends_with('0') {
                digits.pop();
            }

            if digits.ends_with('.') {
                digits.pop();
            }
        }

        digits
    } else {
        let scale = pow10((-decimal_places) as u32);
        let denominator = divisor * scale;

        let mut rounded = bases / denominator;
        let remainder = bases % denominator;

        if remainder >= denominator / 2 + denominator % 2 {
            rounded += 1;
        }

        format!("{}{}", rounded, "0".repeat((-decimal_places) as usize))
    };

    format!("{formatted} {suffix}")
}

fn format_dna_length_with_unit(
    input: &str,
    forced_unit: Option<char>,
) -> Result<String, String> {
    let bases = parse_bases(input)?;

    if let Some(flag) = forced_unit {
        let (divisor, suffix) = match flag.to_ascii_lowercase() {
            'k' => (1_000_u128, "Kbp"),
            'm' => (1_000_000_u128, "Mbp"),
            'g' => (1_000_000_000_u128, "Gbp"),
            't' => (1_000_000_000_000_u128, "Tbp"),
            'p' => (1_000_000_000_000_000_u128, "Pbp"),
            _ => return Err(format!("Unknown unit flag: -{flag}")),
        };

        return Ok(format_value(bases, divisor, suffix));
    }

    let (divisor, suffix) = UNITS
        .iter()
        .find(|(divisor, _)| bases >= *divisor)
        .copied()
        .unwrap_or((1, "bp"));

    // Promote if rounding would produce 1000 in the current unit.
    let threshold = (divisor * 1999) / 2 + (divisor * 1999) % 2;

    if bases >= threshold {
        if let Some(index) = UNITS.iter().position(|u| u.0 == divisor) {
            if index > 0 {
                let (next_divisor, next_suffix) = UNITS[index - 1];
                return Ok(format_value(bases, next_divisor, next_suffix));
            }
        }
    }

    Ok(format_value(bases, divisor, suffix))
}

fn format_output(result: &str, no_space: bool) -> String {
    if no_space {
        result.replace(' ', "")
    } else {
        result.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::format_dna_length_with_unit;

    #[test]
    fn test_valid_number_formats() {
        let cases = [
            ("112", 112),
            ("11200", 11200),
            ("11,200", 11200),
            ("11,200,000", 11200000),
            ("11.200", 11200),
            ("11.200.000", 11200000),
            ("11_200", 11200),
            ("11_200_000", 11200000),
            ("000112", 112),
            ("1,000", 1000),
        ];

        for (input, expected) in cases {
            assert_eq!(
                super::parse_bases(input).unwrap(),
                expected,
                "Input: {input}"
            );
        }
    }

    #[test]
    fn test_reject_malformed_numbers() {
        let cases = [
            "",
            ",112",
            "112,",
            "1,,200",
            "11,,200",
            "1,20",
            "11,20,000",
            "1,2000",
            "1234,567",
            ".112",
            "112.",
            "1..200",
            "11.20.000",
            "11.200,000",
            "1_,200",
            "11__200",
            "11_20",
            "11 200",
            "-112",
            "+112",
            "11.2e3",
            "abc",
        ];

        for input in cases {
            assert!(
                super::parse_bases(input).is_err(),
                "Expected rejection for malformed input: {input:?}"
            );

            assert!(
                format_dna_length_with_unit(input, None).is_err(),
                "Formatter should reject malformed input: {input:?}"
            );
        }
    }

    #[test]
    fn test_automatic_units() {
        let cases = [
            ("112", "112 bp"),
            ("11200", "11.2 Kbp"),
            ("11200000", "11.2 Mbp"),
            ("11200000000", "11.2 Gbp"),
            ("11200000000000", "11.2 Tbp"),
            ("11200000000000000", "11.2 Pbp"),
        ];

        for (input, expected) in cases {
            assert_eq!(
                format_dna_length_with_unit(input, None).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn test_comma_separators() {
        let cases = [
            ("11,200", "11.2 Kbp"),
            ("11,200,000", "11.2 Mbp"),
            ("11,200,000,000", "11.2 Gbp"),
            ("11,200,000,000,000", "11.2 Tbp"),
            ("11,200,000,000,000,000", "11.2 Pbp"),
        ];

        for (input, expected) in cases {
            assert_eq!(
                format_dna_length_with_unit(input, None).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn test_dot_separators() {
        let cases = [
            ("11.200", "11.2 Kbp"),
            ("11.200.000", "11.2 Mbp"),
            ("11.200.000.000", "11.2 Gbp"),
            ("11.200.000.000.000", "11.2 Tbp"),
            ("11.200.000.000.000.000", "11.2 Pbp"),
        ];

        for (input, expected) in cases {
            assert_eq!(
                format_dna_length_with_unit(input, None).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn test_underscore_separators() {
        let cases = [
            ("11_200", "11.2 Kbp"),
            ("11_200_000", "11.2 Mbp"),
            ("11_200_000_000", "11.2 Gbp"),
            ("11_200_000_000_000", "11.2 Tbp"),
            ("11_200_000_000_000_000", "11.2 Pbp"),
        ];

        for (input, expected) in cases {
            assert_eq!(
                format_dna_length_with_unit(input, None).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn test_forced_units() {
        let cases = [
            ("11200", 'k', "11.2 Kbp"),
            ("11200", 'K', "11.2 Kbp"),
            ("11200", 'm', "0.0112 Mbp"),
            ("11200000", 'g', "0.0112 Gbp"),
            ("11200000000000", 't', "11.2 Tbp"),
            ("11200000000000000", 'p', "11.2 Pbp"),
            ("11200000", 'P', "0.0000000112 Pbp"),
        ];

        for (input, flag, expected) in cases {
            assert_eq!(
                format_dna_length_with_unit(input, Some(flag)).unwrap(),
                expected,
                "Input: {input}, flag: {flag}"
            );
        }
    }

    #[test]
    fn test_whole_number_zeroes_are_preserved() {
        assert_eq!(
            format_dna_length_with_unit("100000", None).unwrap(),
            "100 Kbp"
        );
    }

    #[test]
    fn test_all_forced_units_against_all_inputs() {
        let inputs = [
            "112",
            "11200",
            "11200000",
            "11200000000",
            "11200000000000",
            "11200000000000000",
            "11,200",
            "11,200,000",
            "11,200,000,000",
            "11,200,000,000,000",
            "11,200,000,000,000,000",
            "11.200",
            "11.200.000",
            "11.200.000.000",
            "11.200.000.000.000",
            "11.200.000.000.000.000",
            "11_200",
            "11_200_000",
            "11_200_000_000",
            "11_200_000_000_000",
            "11_200_000_000_000_000"
        ];

        let flags = ['k', 'K', 'm', 'M', 'g', 'G', 't', 'T', 'p', 'P'];

        let divisors = [
            ('k', 1_000_u128, "Kbp"),
            ('m', 1_000_000_u128, "Mbp"),
            ('g', 1_000_000_000_u128, "Gbp"),
            ('t', 1_000_000_000_000_u128, "Tbp"),
            ('p', 1_000_000_000_000_000_u128, "Pbp"),
        ];

        for flag in flags {
            let (divisor, suffix) = divisors
                .iter()
                .find(|(unit, _, _)| *unit == flag.to_ascii_lowercase())
                .map(|(_, divisor, suffix)| (*divisor, *suffix))
                .unwrap();

            for input in inputs {
                let result =
                    format_dna_length_with_unit(input, Some(flag)).unwrap();

                let bases = super::parse_bases(input).unwrap();
                let expected = super::format_value(bases, divisor, suffix);

                assert_eq!(
                    result, expected,
                    "Failed for flag -{flag} and input {input}"
                );
            }
        }
    }

    #[test]
    fn test_zero() {
        assert_eq!(
            format_dna_length_with_unit("0", None).unwrap(),
            "0 bp"
        );

        for flag in ['k', 'm', 'g', 't', 'p', 'K', 'M', 'G', 'T', 'P'] {
            let result =
                format_dna_length_with_unit("0", Some(flag)).unwrap();

            assert!(result.starts_with("0 "), "Flag: {flag}");
        }
    }

    #[test]
    fn test_rejects_u128_overflow() {
        assert!(
            super::parse_bases("340282366920938463463374607431768211456")
                .is_err()
        );
    }

    #[test]
    fn test_u128_max_is_formatted() {
        let input = u128::MAX.to_string();

        let result = format_dna_length_with_unit(&input, None).unwrap();

        assert!(result.ends_with(" Pbp"), "{result}");
        assert!(!result.contains("inf"));
        assert!(!result.contains("NaN"));
    }

    #[test]
    fn test_invalid_forced_unit() {
        assert!(
            format_dna_length_with_unit("11200", Some('x')).is_err()
        );
    }

    #[test]
    fn test_automatic_unit_boundary() {
        let cases = [
            ("999499", "999 Kbp"),
            ("999500", "1 Mbp"),
            ("999999", "1 Mbp"),
            ("1000000", "1 Mbp"),
        ];

        for (input, expected) in cases {
            assert_eq!(
                format_dna_length_with_unit(input, None).unwrap(),
                expected,
                "Input: {input}"
            );
        }
    }
    
    #[test]
    fn test_output_with_space_by_default() {
        assert_eq!(format_output("100 Kbp", false), "100 Kbp");
        assert_eq!(format_output("11.2 Mbp", false), "11.2 Mbp");
        assert_eq!(format_output("42 bp", false), "42 bp");
    }
    
    #[test]
    fn test_output_without_space() {
        assert_eq!(format_output("100 Kbp", true), "100Kbp");
        assert_eq!(format_output("11.2 Mbp", true), "11.2Mbp");
        assert_eq!(format_output("42 bp", true), "42bp");
        assert_eq!(format_output("0 Pbp", true), "0Pbp");
    }
    
    #[test]
    fn test_no_space_with_automatic_units() {
        let result = format_dna_length_with_unit("11200", None).unwrap();
        assert_eq!(format_output(&result, true), "11.2Kbp");
    
        let result = format_dna_length_with_unit("11200000", None).unwrap();
        assert_eq!(format_output(&result, true), "11.2Mbp");
    }
    
    #[test]
    fn test_no_space_with_forced_units() {
        let result = format_dna_length_with_unit("11200", Some('k')).unwrap();
        assert_eq!(format_output(&result, true), "11.2Kbp");
    
        let result = format_dna_length_with_unit("11200", Some('m')).unwrap();
        assert_eq!(format_output(&result, true), "0.0112Mbp");
    }
    
    #[test]
    fn test_no_space_does_not_change_numeric_formatting() {
        let result = format_dna_length_with_unit("999999", None).unwrap();
    
        assert_eq!(format_output(&result, false), "1 Mbp");
        assert_eq!(format_output(&result, true), "1Mbp");
    }
}
