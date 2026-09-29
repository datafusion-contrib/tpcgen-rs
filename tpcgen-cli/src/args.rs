//! Shared command-line argument parsing.

/// Scale factors approved for TPC-H results.
///
/// See https://github.com/datafusion-contrib/tpcgen-rs/issues/526
pub(crate) const TPCH_SCALE_FACTORS: &[f64] = &[
    1.0, 10.0, 30.0, 100.0, 300.0, 1_000.0, 3_000.0, 10_000.0, 30_000.0, 100_000.0,
];

/// Scale factors approved for TPC-DS results.
///
/// SF 1 (the qualification database) is also included.
///
/// See https://github.com/datafusion-contrib/tpcgen-rs/issues/526
pub(crate) const TPCDS_SCALE_FACTORS: &[f64] =
    &[1.0, 1_000.0, 3_000.0, 10_000.0, 30_000.0, 100_000.0];

/// Largest scale factor defined by the TPC-H and TPC-DS specifications.
pub(crate) const MAX_BENCHMARK_SCALE_FACTOR: f64 = TPCH_SCALE_FACTORS[TPCH_SCALE_FACTORS.len() - 1];

/// Logs at INFO level if `scale_factor` is not one of the `approved` sizes.
pub(crate) fn log_if_unapproved_scale_factor(benchmark: &str, scale_factor: f64, approved: &[f64]) {
    if !approved.contains(&scale_factor) {
        let approved = approved
            .iter()
            .map(|sf| sf.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        log::info!(
            "Scale factor {scale_factor} is not an approved {benchmark} scale factor ({approved})"
        );
    }
}

/// Default number of generation threads: the available parallelism, or 1 if
/// it cannot be determined.
pub(crate) fn default_num_threads() -> usize {
    std::thread::available_parallelism().map_or(1, |n| n.get())
}

/// Parse a delimiter string, handling the `\t` escape sequence.
///
/// Restrict delimiters to comma, pipe, tab, and semicolon so unquoted fields
/// remain intact. Reject unsupported values before generation starts.
pub(crate) fn parse_delimiter(value: &str) -> Result<char, String> {
    match value {
        "," => Ok(','),
        "|" => Ok('|'),
        "\t" | "\\t" => Ok('\t'),
        ";" => Ok(';'),
        _ => Err(format!(
            "CSV delimiter must be ',' (comma), '|' (pipe), '\\t' (tab), or ';' (semicolon), got {value:?}"
        )),
    }
}

/// Parse a scale factor, rejecting NaN, infinite, and negative values.
///
/// Any finite scale factor is accepted, including above the benchmark maximum.
pub(crate) fn parse_scale_factor(value: &str) -> Result<f64, String> {
    value
        .parse::<f64>()
        .ok()
        .filter(|scale| scale.is_finite() && *scale >= 0.0)
        .ok_or_else(|| format!("expected a non-negative number, got {value:?}"))
}

/// Validate CLI partition options before starting generation.
pub(crate) fn validate_partition_options(
    part: Option<i32>,
    parts: Option<i32>,
) -> Result<(), String> {
    let Some(parts) = parts else {
        return if part.is_some() {
            Err("The --part option requires the --parts option to be set".to_string())
        } else {
            Ok(())
        };
    };
    if parts < 1 {
        return Err(format!(
            "Invalid --parts value '{parts}'. Expected a number greater than zero"
        ));
    }
    if let Some(part) = part {
        if part < 1 {
            return Err(format!(
                "Invalid --part value '{part}'. Expected a number greater than zero"
            ));
        }
        if part > parts {
            return Err(format!(
                "Invalid --part value '{part}'. Expected at most the value of --parts ({parts})"
            ));
        }
    }
    Ok(())
}

/// Parses a positive byte count using case-insensitive GNU size suffixes.
///
/// * `KB`, `MB`, `GB`, `TB`: powers of 1000
/// * `K`, `M`, `G`, `T` and `KiB`, `MiB`, `GiB`, `TiB`: powers of 1024
pub(crate) fn parse_row_group_bytes(value: &str) -> Result<i64, String> {
    if value.starts_with('-') {
        return Err("must be greater than zero".to_string());
    }

    let number_end = value
        .find(|character: char| !character.is_ascii_digit())
        .unwrap_or(value.len());
    let (number, suffix) = value.split_at(number_end);

    let number = number.parse::<i64>().map_err(|error| error.to_string())?;
    if number == 0 {
        return Err("must be greater than zero".to_string());
    }

    let multiplier = match suffix.to_ascii_lowercase().as_str() {
        "" | "b" => 1,
        "kb" => 1_000,
        "k" | "kib" => 1 << 10,
        "mb" => 1_000_000,
        "m" | "mib" => 1 << 20,
        "gb" => 1_000_000_000,
        "g" | "gib" => 1 << 30,
        "tb" => 1_000_000_000_000,
        "t" | "tib" => 1_i64 << 40,
        _ => return Err(format!("unknown byte unit '{suffix}'")),
    };

    number
        .checked_mul(multiplier)
        .ok_or_else(|| "byte value is too large".to_string())
}

/// Asserts that each format subcommand in `commands` lists its format-specific
/// options under the heading returned by `heading_for` (e.g. "Parquet Options"),
/// while options shared via `common` stay under the default heading.
#[cfg(test)]
pub(crate) fn assert_format_options_grouped(
    commands: clap::Command,
    common: clap::Command,
    heading_for: impl Fn(&str) -> &'static str,
) {
    let common_ids: std::collections::HashSet<_> =
        common.get_arguments().map(|arg| arg.get_id()).collect();

    for subcommand in commands.get_subcommands() {
        let name = subcommand.get_name();
        let expected = heading_for(name);
        for arg in subcommand.get_arguments() {
            let id = arg.get_id();
            let heading = arg.get_help_heading();
            if common_ids.contains(id) {
                assert_eq!(
                    heading, None,
                    "shared option `{id}` on `{name}` should not have a help heading"
                );
            } else {
                assert_eq!(
                    heading,
                    Some(expected),
                    "`{name}` option `{id}` should set `help_heading = \"{expected}\"`"
                );
            }
        }
    }
}

/// Requires an explicit info-logging decision for every format-specific option.
#[cfg(test)]
pub(crate) fn assert_format_options_have_logging_policy(
    commands: clap::Command,
    common: clap::Command,
) {
    use std::collections::BTreeSet;

    let common_ids: BTreeSet<_> = common
        .get_arguments()
        .map(|arg| arg.get_id().as_str())
        .collect();

    for subcommand in commands.get_subcommands() {
        let name = subcommand.get_name();
        let (logged, omitted): (&[&str], &[&str]) = match name {
            "tbl" | "dat" => (&[], &[]),
            "csv" => (&["delimiter"], &[]),
            // Per-column overrides are too detailed for the info-level summary.
            "parquet" => (&["compression", "row_group_bytes"], &["column_encoding"]),
            other => panic!("add a logging policy for the `{other}` subcommand"),
        };
        let expected: BTreeSet<_> = logged.iter().chain(omitted).copied().collect();
        let actual: BTreeSet<_> = subcommand
            .get_arguments()
            .map(|arg| arg.get_id().as_str())
            .filter(|id| !common_ids.contains(id))
            .collect();
        assert_eq!(
            actual, expected,
            "`{name}` options changed: decide whether to log each option at info or intentionally omit it"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_factor_boundaries_and_fractions() {
        for (value, expected) in [
            ("0", 0.0),
            ("0.001", 0.001),
            ("100000", 100_000.0),
            ("100001", 100_001.0),
        ] {
            assert_eq!(parse_scale_factor(value), Ok(expected));
        }
        for value in ["-1", "NaN", "inf", "-inf"] {
            assert!(parse_scale_factor(value).is_err());
        }
    }

    #[test]
    fn csv_delimiter_parses_and_validates_values() {
        for delimiter in (0..=127u8).map(char::from).chain(['\u{20ac}', '\u{e9}']) {
            let expected = matches!(delimiter, ',' | '|' | '\t' | ';');
            let parsed = parse_delimiter(&delimiter.to_string());
            assert_eq!(parsed.is_ok(), expected, "{delimiter:?}");
            if expected {
                assert_eq!(parsed.unwrap(), delimiter);
            }
        }
        assert_eq!(parse_delimiter("\\t"), Ok('\t'));
        for value in ["", "\\n", "\\r", "\\\\", "\\t\\t", "||", "tab"] {
            assert!(parse_delimiter(value).is_err(), "{value:?}");
        }
    }

    #[test]
    fn row_group_bytes_parses_and_validates_values() {
        assert_eq!(parse_row_group_bytes("1"), Ok(1));
        assert_eq!(parse_row_group_bytes(&i64::MAX.to_string()), Ok(i64::MAX));
        assert_eq!(parse_row_group_bytes("8mb"), Ok(8 * 1000 * 1000));
        assert_eq!(parse_row_group_bytes("8M"), Ok(8 * 1024 * 1024));
        assert_eq!(parse_row_group_bytes("8MiB"), Ok(8 * 1024 * 1024));
        assert_eq!(
            parse_row_group_bytes("-1"),
            Err("must be greater than zero".to_string())
        );
        assert_eq!(
            parse_row_group_bytes("0MB"),
            Err("must be greater than zero".to_string())
        );
        assert!(parse_row_group_bytes(&(i64::MAX as u64 + 1).to_string()).is_err());
        assert!(parse_row_group_bytes(&format!("{}KB", i64::MAX)).is_err());
        assert!(parse_row_group_bytes("MiB").is_err());
        assert!(parse_row_group_bytes("1.5MB").is_err());
        assert!(parse_row_group_bytes("+1").is_err());
        assert!(parse_row_group_bytes("8watts").is_err());
    }
}
