use crate::row::table_row::DatField;
use std::fmt;

/// Reason table row (ReasonRow)
#[derive(Debug, Clone)]
pub struct ReasonRow {
    null_bit_map: i64,
    pub(crate) r_reason_sk: i64,
    pub(crate) r_reason_id: String,
    pub(crate) r_reason_desc: String,
}

impl ReasonRow {
    pub fn new(
        null_bit_map: i64,
        r_reason_sk: i64,
        r_reason_id: String,
        r_reason_desc: String,
    ) -> Self {
        ReasonRow {
            null_bit_map,
            r_reason_sk,
            r_reason_id,
            r_reason_desc,
        }
    }

    /// Check if a column should be null based on the null bitmap (TableRowWithNulls logic)
    pub(crate) fn should_be_null(&self, column_position: i32) -> bool {
        ((self.null_bit_map >> column_position) & 1) == 1
    }

    pub fn null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    pub fn get_r_reason_sk(&self) -> i64 {
        self.r_reason_sk
    }

    pub fn get_r_reason_id(&self) -> &str {
        &self.r_reason_id
    }

    pub fn get_r_reason_desc(&self) -> &str {
        &self.r_reason_desc
    }

    #[deprecated(note = "use get_r_reason_desc")]
    pub fn get_r_reason_description(&self) -> &str {
        self.get_r_reason_desc()
    }
}

/// DAT field helper: NULL is driven purely by the null bit
/// (reason applies no key sentinel check, only the null bit).
impl ReasonRow {
    pub(crate) fn field<T>(&self, value: T, column_position: i32) -> DatField<T> {
        DatField::new(value, self.should_be_null(column_position))
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces one
/// `|`-terminated field per column.
impl fmt::Display for ReasonRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}|",
            self.field(self.r_reason_sk, 0),
            self.field(&self.r_reason_id, 1),
            self.field(&self.r_reason_desc, 2),
        )
    }
}
