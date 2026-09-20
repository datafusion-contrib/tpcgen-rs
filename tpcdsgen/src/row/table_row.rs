use std::fmt;

/// A single DAT-format field: formats the value, or nothing when the column
/// is NULL.
///
/// Row types use this in their `Display` impls (which emit the DAT line) so
/// that NULL handling stays out of the format string.
pub(crate) struct DatField<T>(Option<T>);

impl<T: fmt::Display> fmt::Display for DatField<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(value) => value.fmt(f),
            None => Ok(()),
        }
    }
}

impl<T> DatField<T> {
    /// DAT field for a regular value: NULL when the row's null bit is set.
    pub(crate) fn new(value: T, is_null: bool) -> Self {
        DatField((!is_null).then_some(value))
    }
}

/// DAT field from an already-computed optional value, for callers whose
/// value is only constructible when non-NULL.
impl<T> From<Option<T>> for DatField<T> {
    fn from(value: Option<T>) -> Self {
        DatField(value)
    }
}

impl DatField<i64> {
    /// DAT field for a surrogate key: NULL when the row's null bit is set or
    /// the key is -1 (the generators' "no reference" sentinel).
    pub(crate) fn key(key: i64, is_null: bool) -> Self {
        DatField((!is_null && key != -1).then_some(key))
    }
}

impl DatField<&'static str> {
    /// DAT field for a boolean, formatted as `Y`/`N`.
    pub(crate) fn yes_no(value: bool, is_null: bool) -> Self {
        DatField::new(if value { "Y" } else { "N" }, is_null)
    }
}

/// A DAT field that prints the literal `NULL` for NULL columns instead of an
/// empty string — a quirk of the Java call_center and household_demographics
/// rows that we preserve for byte-for-byte compatibility.
pub(crate) struct NullLiteralField<T>(Option<T>);

impl<T: fmt::Display> fmt::Display for NullLiteralField<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(value) => value.fmt(f),
            None => f.write_str("NULL"),
        }
    }
}

impl<T> NullLiteralField<T> {
    /// DAT field printing the literal `NULL` when the row's null bit is set.
    pub(crate) fn new(value: T, is_null: bool) -> Self {
        NullLiteralField((!is_null).then_some(value))
    }
}

/// A double-quoted CSV field: formats the value wrapped in `"`, or nothing
/// when the column is NULL.
///
/// Used by the CSV wrappers in [`crate::csv`] for free-text columns whose
/// values may contain the CSV delimiter (see [`crate::csv`]). The generated
/// text never contains `"` or newlines, so no escaping is needed.
pub(crate) struct CsvQuoted<T>(Option<T>);

impl<T> CsvQuoted<T> {
    /// Quoted CSV field for a free-text value: NULL (empty) when the row's
    /// null bit is set.
    pub(crate) fn new(value: T, is_null: bool) -> Self {
        CsvQuoted((!is_null).then_some(value))
    }
}

impl<T: fmt::Display> fmt::Display for CsvQuoted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(value) => write!(f, "\"{value}\""),
            None => Ok(()),
        }
    }
}

/// A quoted CSV field that prints the literal `NULL` (unquoted) for NULL
/// columns — the CSV counterpart of [`NullLiteralField`], keeping the CSV
/// values identical to the DAT values for the call_center quirk columns.
pub(crate) struct CsvQuotedNullLiteral<T>(Option<T>);

impl<T> CsvQuotedNullLiteral<T> {
    /// Quoted CSV field printing the literal `NULL` when the row's null bit
    /// is set.
    pub(crate) fn new(value: T, is_null: bool) -> Self {
        CsvQuotedNullLiteral((!is_null).then_some(value))
    }
}

impl<T: fmt::Display> fmt::Display for CsvQuotedNullLiteral<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(value) => write!(f, "\"{value}\""),
            None => f.write_str("NULL"),
        }
    }
}

/// Zero-padded five-digit zip code (`{:05}`).
pub(crate) struct Zip5(i32);

impl fmt::Display for Zip5 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:05}", self.0)
    }
}

impl DatField<Zip5> {
    /// DAT field for a zip code, zero-padded to five digits.
    pub(crate) fn zip(zip: i32, is_null: bool) -> Self {
        DatField::new(Zip5(zip), is_null)
    }
}
