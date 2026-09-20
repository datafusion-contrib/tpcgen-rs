use crate::row::table_row::DatField;
use crate::types::Date;
use std::fmt;

/// Represents a row in the DATE_DIM table
#[derive(Debug, Clone)]
pub struct DateDimRow {
    // Null bitmap for handling NULL values
    null_bit_map: i64,

    // Primary key
    pub d_date_sk: i64,

    // Date identifier (YYYYMMDD format)
    pub d_date_id: String,

    // Date value
    pub d_date: Date,

    // Sequence numbers
    pub d_month_seq: i32,
    pub d_week_seq: i32,
    pub d_quarter_seq: i32,

    // Year components
    pub d_year: i32,
    pub d_dow: i32, // Day of week (0-6)
    pub d_moy: i32, // Month of year (1-12)
    pub d_dom: i32, // Day of month (1-31)
    pub d_qoy: i32, // Quarter of year (1-4)

    // Fiscal year components
    pub d_fy_year: i32,
    pub d_fy_quarter_seq: i32,
    pub d_fy_week_seq: i32,

    // Names
    pub d_day_name: String,     // Monday, Tuesday, etc.
    pub d_quarter_name: String, // 2024Q1, 2024Q2, etc.

    // Flags
    pub d_holiday: bool,
    pub d_weekend: bool,
    pub d_following_holiday: bool,

    // First and last day of month
    pub d_first_dom: i32,
    pub d_last_dom: i32,

    // Same day references (julian days)
    pub d_same_day_ly: i32, // Same day last year
    pub d_same_day_lq: i32, // Same day last quarter

    // Current flags (relative to a reference date)
    pub d_current_day: bool,
    pub d_current_week: bool,
    pub d_current_month: bool,
    pub d_current_quarter: bool,
    pub d_current_year: bool,
}

impl DateDimRow {
    /// Create a new DateDimRow with all fields
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        null_bit_map: i64,
        d_date_sk: i64,
        d_date_id: String,
        d_date: Date,
        d_month_seq: i32,
        d_week_seq: i32,
        d_quarter_seq: i32,
        d_year: i32,
        d_dow: i32,
        d_moy: i32,
        d_dom: i32,
        d_qoy: i32,
        d_fy_year: i32,
        d_fy_quarter_seq: i32,
        d_fy_week_seq: i32,
        d_day_name: String,
        d_quarter_name: String,
        d_holiday: bool,
        d_weekend: bool,
        d_following_holiday: bool,
        d_first_dom: i32,
        d_last_dom: i32,
        d_same_day_ly: i32,
        d_same_day_lq: i32,
        d_current_day: bool,
        d_current_week: bool,
        d_current_month: bool,
        d_current_quarter: bool,
        d_current_year: bool,
    ) -> Self {
        DateDimRow {
            null_bit_map,
            d_date_sk,
            d_date_id,
            d_date,
            d_month_seq,
            d_week_seq,
            d_quarter_seq,
            d_year,
            d_dow,
            d_moy,
            d_dom,
            d_qoy,
            d_fy_year,
            d_fy_quarter_seq,
            d_fy_week_seq,
            d_day_name,
            d_quarter_name,
            d_holiday,
            d_weekend,
            d_following_holiday,
            d_first_dom,
            d_last_dom,
            d_same_day_ly,
            d_same_day_lq,
            d_current_day,
            d_current_week,
            d_current_month,
            d_current_quarter,
            d_current_year,
        }
    }

    pub fn null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    /// Check if a column should be NULL based on the null bitmap
    fn is_field_null(&self, column_index: usize) -> bool {
        (self.null_bit_map & (1 << column_index)) != 0
    }

    /// Format a boolean value for output
    pub(crate) fn format_boolean(value: bool) -> &'static str {
        if value {
            "Y"
        } else {
            "N"
        }
    }
}

/// DAT field helper for this row's columns.
impl DateDimRow {
    pub(crate) fn field<T>(&self, value: T, column_index: usize) -> DatField<T> {
        DatField::new(value, self.is_field_null(column_index))
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces one
/// `|`-terminated field per column.
impl fmt::Display for DateDimRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|",
            self.field(self.d_date_sk, 0),
            self.field(&self.d_date_id, 1),
            self.field(&self.d_date, 2),
            self.field(self.d_month_seq, 3),
            self.field(self.d_week_seq, 4),
            self.field(self.d_quarter_seq, 5),
            self.field(self.d_year, 6),
            self.field(self.d_dow, 7),
            self.field(self.d_moy, 8),
            self.field(self.d_dom, 9),
            self.field(self.d_qoy, 10),
            self.field(self.d_fy_year, 11),
            self.field(self.d_fy_quarter_seq, 12),
            self.field(self.d_fy_week_seq, 13),
            self.field(&self.d_day_name, 14),
            self.field(&self.d_quarter_name, 15),
            self.field(Self::format_boolean(self.d_holiday), 16),
            self.field(Self::format_boolean(self.d_weekend), 17),
            self.field(Self::format_boolean(self.d_following_holiday), 18),
            self.field(self.d_first_dom, 19),
            self.field(self.d_last_dom, 20),
            self.field(self.d_same_day_ly, 21),
            self.field(self.d_same_day_lq, 22),
            self.field(Self::format_boolean(self.d_current_day), 23),
            self.field(Self::format_boolean(self.d_current_week), 24),
            self.field(Self::format_boolean(self.d_current_month), 25),
            self.field(Self::format_boolean(self.d_current_quarter), 26),
            self.field(Self::format_boolean(self.d_current_year), 27),
        )
    }
}
