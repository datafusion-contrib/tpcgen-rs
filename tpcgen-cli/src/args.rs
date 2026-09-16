//! Shared command-line argument parsing.

pub(crate) fn parse_row_group_bytes(value: &str) -> Result<i64, String> {
    let bytes = value.parse::<i64>().map_err(|err| err.to_string())?;
    if bytes <= 0 {
        return Err("must be greater than zero".to_string());
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_group_bytes_enforces_i64_range() {
        assert_eq!(parse_row_group_bytes("1"), Ok(1));
        assert_eq!(parse_row_group_bytes(&i64::MAX.to_string()), Ok(i64::MAX));
        assert!(parse_row_group_bytes("9223372036854775808").is_err());
    }
}
