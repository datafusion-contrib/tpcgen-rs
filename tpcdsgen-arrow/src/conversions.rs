//! Routines to convert TPC-DS types to Arrow types

use arrow::array::{Decimal128Array, StringViewArray, StringViewBuilder};
use tpcdsgen::types::{Address, Date, Decimal};

/// Julian day number for the Unix epoch (1970-01-01)
const UNIX_EPOCH_JULIAN: i32 = 2440588;

/// Convert a TPC-DS Decimal to an i128 value suitable for Decimal128Array.
///
/// TPC-DS Decimal stores the number as an unscaled integer (e.g. 1234 means 12.34
/// when precision=2). Arrow Decimal128 also stores unscaled integers, so we just
/// cast the i64 to i128.
#[inline(always)]
pub fn decimal_to_i128(d: Decimal) -> i128 {
    d.get_number() as i128
}

#[deprecated(note = "use a fixed TPC-DS decimal array helper")]
pub fn decimal128_array_from_iter<I>(values: I) -> Decimal128Array
where
    I: Iterator<Item = Decimal>,
{
    Decimal128Array::from_iter_values(values.map(decimal_to_i128))
        .with_precision_and_scale(38, 2)
        .unwrap()
}

#[deprecated(note = "use a fixed TPC-DS decimal array helper")]
pub fn decimal128_array_from_opt_iter<I>(values: I) -> Decimal128Array
where
    I: Iterator<Item = Option<Decimal>>,
{
    Decimal128Array::from_iter(values.map(|value| value.map(decimal_to_i128)))
        .with_precision_and_scale(38, 2)
        .unwrap()
}

/// Convert a TPC-DS Date to an Arrow Date32 (days since Unix epoch 1970-01-01).
///
/// TPC-DS Date is stored as a Julian day number internally. Julian day 2440588
/// corresponds to 1970-01-01.
#[inline(always)]
pub fn date_to_date32(d: &Date) -> i32 {
    d.to_julian_days() - UNIX_EPOCH_JULIAN
}

/// Convert a Julian day i64 to an Arrow Date32. Returns None if julian_days < 0.
#[inline(always)]
pub fn julian_to_date32(julian_days: i64) -> Option<i32> {
    if julian_days < 0 {
        None
    } else {
        Some(julian_days as i32 - UNIX_EPOCH_JULIAN)
    }
}

/// Build a TPC-DS DECIMAL(5,2) array from unscaled integer values.
pub fn decimal128_5_2_array(values: impl IntoIterator<Item = Option<i128>>) -> Decimal128Array {
    Decimal128Array::from_iter(values)
        .with_precision_and_scale(5, 2)
        .unwrap()
}

/// Build a TPC-DS DECIMAL(7,2) array from unscaled integer values.
pub fn decimal128_7_2_array(values: impl IntoIterator<Item = Option<i128>>) -> Decimal128Array {
    Decimal128Array::from_iter(values)
        .with_precision_and_scale(7, 2)
        .unwrap()
}

/// Build a TPC-DS DECIMAL(15,2) array from unscaled integer values.
pub fn decimal128_15_2_array(values: impl IntoIterator<Item = Option<i128>>) -> Decimal128Array {
    Decimal128Array::from_iter(values)
        .with_precision_and_scale(15, 2)
        .unwrap()
}

/// Build a TPC-DS DECIMAL(5,2) array from whole-number GMT offsets.
pub fn gmt_offset_decimal128_array(
    values: impl IntoIterator<Item = Option<i32>>,
) -> Decimal128Array {
    Decimal128Array::from_iter(
        values
            .into_iter()
            .map(|value| value.map(|value| i128::from(value) * 100)),
    )
    .with_precision_and_scale(5, 2)
    .unwrap()
}

/// Build a StringViewArray from an iterator of &str values (non-nullable).
pub fn string_view_array_from_iter<'a, I>(values: I) -> StringViewArray
where
    I: Iterator<Item = &'a str>,
{
    let values: Vec<&str> = values.collect();
    let size_hint = values.len();
    let mut builder = StringViewBuilder::with_capacity(size_hint);
    for v in values {
        builder.append_value(v);
    }
    builder.finish()
}

/// Build a StringViewArray from an iterator of optional &str values (nullable).
pub fn string_view_array_from_opt_iter<'a, I>(values: I) -> StringViewArray
where
    I: Iterator<Item = Option<&'a str>>,
{
    let values: Vec<Option<&str>> = values.collect();
    let size_hint = values.len();
    let mut builder = StringViewBuilder::with_capacity(size_hint);
    for v in values {
        match v {
            Some(s) => builder.append_value(s),
            None => builder.append_null(),
        }
    }
    builder.finish()
}

/// Build a StringViewArray from an iterator of owned String values (nullable).
pub fn string_view_array_from_string_opt_iter<I>(values: I) -> StringViewArray
where
    I: Iterator<Item = Option<String>>,
{
    let values: Vec<Option<String>> = values.collect();
    let size_hint = values.len();
    let mut builder = StringViewBuilder::with_capacity(size_hint);
    for v in values {
        match v {
            Some(ref s) => builder.append_value(s.as_str()),
            None => builder.append_null(),
        }
    }
    builder.finish()
}

/// Convert a boolean value to "Y" or "N" static str.
#[inline(always)]
pub fn bool_to_yn(b: bool) -> &'static str {
    if b {
        "Y"
    } else {
        "N"
    }
}

/// Check whether the bit at `pos` is set in a null bitmap, indicating a NULL value.
#[inline(always)]
pub fn is_null(nbm: i64, pos: u32) -> bool {
    (nbm >> pos) & 1 != 0
}

/// Return `Some(val)` unless the null bitmap bit at `pos` is set.
#[inline(always)]
pub fn opt<T>(nbm: i64, pos: u32, val: T) -> Option<T> {
    if is_null(nbm, pos) {
        None
    } else {
        Some(val)
    }
}

/// Return a checked Arrow Int32 value unless the null bitmap bit is set.
#[inline(always)]
pub fn integer_opt(nbm: i64, pos: u32, value: i64) -> Option<i32> {
    if is_null(nbm, pos) {
        None
    } else {
        Some(i32::try_from(value).expect("TPC-DS INTEGER value exceeds i32 range"))
    }
}

/// Return `Some(sk)` unless null bitmap bit is set OR sk < 0 (sentinel for absent FK).
#[inline(always)]
pub fn sk_opt(nbm: i64, pos: u32, sk: i64) -> Option<i64> {
    if is_null(nbm, pos) || sk < 0 {
        None
    } else {
        Some(sk)
    }
}

/// Return a checked Arrow Int32 surrogate key unless null or an absent-key sentinel.
#[inline(always)]
pub fn integer_sk_opt(nbm: i64, pos: u32, sk: i64) -> Option<i32> {
    if is_null(nbm, pos) || sk < 0 {
        None
    } else {
        Some(i32::try_from(sk).expect("TPC-DS INTEGER surrogate key exceeds i32 range"))
    }
}

/// Expand an [`Address`] into 10 individual column arrays (street_number, street_name,
/// street_type, suite_number, city, county, state, zip, country, gmt_offset).
///
/// Returns `([StringViewArray; 9], Decimal128Array)`.
pub fn address_columns<'a>(
    rows: impl Iterator<Item = (&'a Address, i64, u32)> + 'a,
) -> (
    StringViewArray,
    StringViewArray,
    StringViewArray,
    StringViewArray,
    StringViewArray,
    StringViewArray,
    StringViewArray,
    StringViewArray,
    StringViewArray,
    Decimal128Array,
) {
    let rows: Vec<_> = rows.collect();
    let street_number =
        string_view_array_from_string_opt_iter(rows.iter().map(|(a, nbm, base)| {
            if is_null(*nbm, *base) {
                None
            } else {
                Some(a.get_street_number().to_string())
            }
        }));
    let mut street_name_b = StringViewBuilder::with_capacity(rows.len());
    let mut street_type_b = StringViewBuilder::with_capacity(rows.len());
    let mut suite_number_b = StringViewBuilder::with_capacity(rows.len());
    let mut city_b = StringViewBuilder::with_capacity(rows.len());
    let mut county_b = StringViewBuilder::with_capacity(rows.len());
    let mut state_b = StringViewBuilder::with_capacity(rows.len());
    let mut zip_b = StringViewBuilder::with_capacity(rows.len());
    let mut country_b = StringViewBuilder::with_capacity(rows.len());

    // Each address sub-field has its own null bit at base+offset (0=street_number,
    // 1=street_name, ..., 9=gmt_offset), matching the per-column null_bit_map layout.
    for (a, nbm, base) in &rows {
        if is_null(*nbm, *base + 1) {
            street_name_b.append_null();
        } else {
            street_name_b.append_value(a.get_street_name());
        }
        if is_null(*nbm, *base + 2) {
            street_type_b.append_null();
        } else {
            street_type_b.append_value(a.get_street_type());
        }
        if is_null(*nbm, *base + 3) {
            suite_number_b.append_null();
        } else {
            suite_number_b.append_value(a.get_suite_number());
        }
        if is_null(*nbm, *base + 4) {
            city_b.append_null();
        } else {
            city_b.append_value(a.get_city());
        }
        match a.get_county() {
            Some(c) if !is_null(*nbm, *base + 5) => county_b.append_value(c),
            _ => county_b.append_null(),
        }
        if is_null(*nbm, *base + 6) {
            state_b.append_null();
        } else {
            state_b.append_value(a.get_state());
        }
        if is_null(*nbm, *base + 7) {
            zip_b.append_null();
        } else {
            zip_b.append_value(format!("{:05}", a.get_zip()));
        }
        if is_null(*nbm, *base + 8) {
            country_b.append_null();
        } else {
            country_b.append_value(a.get_country());
        }
    }
    let gmt_offset = gmt_offset_decimal128_array(rows.iter().map(|(a, nbm, base)| {
        if is_null(*nbm, *base + 9) {
            None
        } else {
            Some(a.get_gmt_offset())
        }
    }));
    (
        street_number,
        street_name_b.finish(),
        street_type_b.finish(),
        suite_number_b.finish(),
        city_b.finish(),
        county_b.finish(),
        state_b.finish(),
        zip_b.finish(),
        country_b.finish(),
        gmt_offset,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::Array;

    #[test]
    fn test_decimal_to_i128() {
        let d = Decimal::new(12345, 2).unwrap();
        assert_eq!(decimal_to_i128(d), 12345);
    }

    #[test]
    fn test_whole_number_decimal128_array_scales_values() {
        let array = gmt_offset_decimal128_array([Some(-5), None, Some(9)]);

        assert_eq!(array.value(0), -500);
        assert!(array.is_null(1));
        assert_eq!(array.value(2), 900);
        assert_eq!(array.precision(), 5);
        assert_eq!(array.scale(), 2);
    }

    #[test]
    fn test_julian_to_date32_epoch() {
        // Julian day 2440588 = 1970-01-01, so offset should be 0
        assert_eq!(julian_to_date32(2440588), Some(0));
    }

    #[test]
    fn test_julian_to_date32_negative() {
        assert_eq!(julian_to_date32(-1), None);
    }

    #[test]
    fn test_bool_to_yn() {
        assert_eq!(bool_to_yn(true), "Y");
        assert_eq!(bool_to_yn(false), "N");
    }

    #[test]
    fn test_integer_opt_boundaries() {
        assert_eq!(integer_opt(0, 0, i64::from(i32::MIN)), Some(i32::MIN));
        assert_eq!(integer_opt(0, 0, i64::from(i32::MAX)), Some(i32::MAX));
        assert_eq!(integer_opt(1, 0, i64::from(i32::MAX) + 1), None);
    }

    #[test]
    #[should_panic(expected = "TPC-DS INTEGER value exceeds i32 range")]
    fn test_integer_opt_rejects_overflow() {
        integer_opt(0, 0, i64::from(i32::MAX) + 1);
    }

    #[test]
    fn test_integer_sk_opt_handles_absent_keys() {
        assert_eq!(integer_sk_opt(0, 0, -1), None);
        assert_eq!(integer_sk_opt(1, 0, i64::from(i32::MAX) + 1), None);
        assert_eq!(integer_sk_opt(0, 0, i64::from(i32::MAX)), Some(i32::MAX));
    }
}
