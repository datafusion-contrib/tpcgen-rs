use crate::row::table_row::DatField;
use std::fmt;

/// Income band table row (IncomeBandRow)
#[derive(Debug, Clone)]
pub struct IncomeBandRow {
    null_bit_map: i64,
    pub(crate) ib_income_band_sk: i32,
    pub(crate) ib_lower_bound: i32,
    pub(crate) ib_upper_bound: i32,
}

impl IncomeBandRow {
    pub fn new(
        null_bit_map: i64,
        ib_income_band_sk: i32,
        ib_lower_bound: i32,
        ib_upper_bound: i32,
    ) -> Self {
        IncomeBandRow {
            null_bit_map,
            ib_income_band_sk,
            ib_lower_bound,
            ib_upper_bound,
        }
    }

    /// Check if a column should be null based on the null bitmap (TableRowWithNulls logic)
    fn should_be_null(&self, column_position: i32) -> bool {
        ((self.null_bit_map >> column_position) & 1) == 1
    }

    pub fn null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    pub fn get_ib_income_band_sk(&self) -> i32 {
        self.ib_income_band_sk
    }

    #[deprecated(note = "use get_ib_income_band_sk")]
    pub fn get_ib_income_band_id(&self) -> i32 {
        self.get_ib_income_band_sk()
    }

    pub fn get_ib_lower_bound(&self) -> i32 {
        self.ib_lower_bound
    }

    pub fn get_ib_upper_bound(&self) -> i32 {
        self.ib_upper_bound
    }
}

/// DAT field helper for this row's columns.
impl IncomeBandRow {
    pub(crate) fn field<T>(&self, value: T, column_position: i32) -> DatField<T> {
        DatField::new(value, self.should_be_null(column_position))
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces one
/// `|`-terminated field per column.
impl fmt::Display for IncomeBandRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}|",
            self.field(self.ib_income_band_sk, 0),
            self.field(self.ib_lower_bound, 1),
            self.field(self.ib_upper_bound, 2),
        )
    }
}
