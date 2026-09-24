//! Shared command-line argument parsing.

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

#[cfg(test)]
mod tests {
    use super::*;

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
