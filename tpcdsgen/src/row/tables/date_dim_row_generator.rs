use crate::business_key_generator::make_business_key;
use crate::config::Session;
use crate::distribution::CalendarDistribution;
use crate::row::{AbstractRowGenerator, DateDimRow, RowGenerator, RowGeneratorResult};
use crate::table::Table;
use crate::types::Date;

/// Constants for date calculations
const TODAYS_DATE: Date = Date::new(2003, 1, 8); // January 8, 2003
const CURRENT_QUARTER: i32 = 1;
const CURRENT_WEEK: i32 = 2; // Week number for TODAYS_DATE

const WEEKDAY_NAMES: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

pub struct DateDimRowGenerator {
    base: AbstractRowGenerator,
}

impl Default for DateDimRowGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl DateDimRowGenerator {
    pub fn new() -> Self {
        DateDimRowGenerator {
            base: AbstractRowGenerator::new(Table::DateDim),
        }
    }
}

impl RowGenerator for DateDimRowGenerator {
    fn generate_row_and_child_rows(
        &mut self,
        row_number: u64,
        _session: &Session,
        _parent_row_generator: Option<&mut dyn RowGenerator>,
        _child_row_generator: Option<&mut dyn RowGenerator>,
    ) -> crate::error::Result<RowGeneratorResult> {
        let row_number_i64 = i64::try_from(row_number).expect("row number fits in i64");

        // Create null bitmap - DateDim has very few nulls
        let null_bit_map = 0i64;

        // Base date is January 1, 1900
        let base_date = Date::new(1900, 1, 1);
        let base_julian = base_date.to_julian_days();

        // Each row represents one day, starting from base date
        let d_date_sk = row_number_i64 + base_julian as i64;
        let d_date_id = make_business_key(u64::try_from(d_date_sk).expect("date key fits in u64"));
        let date = Date::from_julian_days(d_date_sk as i32);

        // Extract date components
        let d_year = date.year();
        let d_dow = date.day_of_week(); // 0 = Sunday, 6 = Saturday
        let d_moy = date.month();
        let d_dom = date.day();

        // Calculate sequence numbers (assumes table starts on year boundary)
        let d_week_seq = row_number.div_ceil(7) as i32;
        let d_month_seq = (d_year - 1900) * 12 + d_moy - 1;
        // Note: Java has a bug where it uses dMoy/3 instead of (dMoy-1)/3
        // This incorrectly puts March in Q2. We replicate this bug for compatibility.
        let d_quarter_seq = (d_year - 1900) * 4 + d_moy / 3 + 1;

        // Get day index for distributions (1-based day of year)
        let day_index = date.day_of_year();
        let d_qoy = CalendarDistribution::get_quarter_at_index(day_index);

        // Fiscal year is identical to calendar year in TPC-DS
        let d_fy_year = d_year;
        let d_fy_quarter_seq = d_quarter_seq;
        let d_fy_week_seq = d_week_seq;

        // Get day name
        let d_day_name = WEEKDAY_NAMES[d_dow as usize].to_string();

        // Calculate quarter name (e.g., "2024Q1")
        let d_quarter_name = format!("{}Q{}", d_year, d_qoy);

        // Determine holiday and weekend flags
        let d_holiday = CalendarDistribution::get_is_holiday_flag_at_index(day_index) != 0;
        // Note: Java implementation has a bug where Friday and Saturday are weekend days
        // We replicate this bug for compatibility
        let d_weekend = d_dow == 5 || d_dow == 6; // Friday or Saturday (bug compatibility)

        // Following holiday flag
        let d_following_holiday = if day_index == 1 {
            // First day of year - check last day of previous year
            // Note: This matches the C/Java bug where it uses 365 + leap year flag
            let last_day_prev_year = if Date::is_leap_year(d_year - 1) {
                366
            } else {
                365
            };
            CalendarDistribution::get_is_holiday_flag_at_index(last_day_prev_year) != 0
        } else {
            CalendarDistribution::get_is_holiday_flag_at_index(day_index - 1) != 0
        };

        // First and last day of month (as julian days)
        let first_of_month = Date::new(d_year, d_moy, 1);
        let d_first_dom = first_of_month.to_julian_days();
        let d_last_dom = date.last_day_of_month().to_julian_days();

        // Same day last year and last quarter (as julian days)
        let d_same_day_ly = date.same_day_last_year().to_julian_days();
        let d_same_day_lq = date.same_day_last_quarter().to_julian_days();

        // Current flags (relative to TODAYS_DATE)
        // Note: Java has a bug where it compares julian days to day of month
        // This will never be true, but we replicate the bug for compatibility
        let d_current_day = d_date_sk == TODAYS_DATE.day() as i64; // Bug: comparing julian to day of month
        let d_current_year = d_year == TODAYS_DATE.year();
        let d_current_month = d_current_year && d_moy == TODAYS_DATE.month();
        let d_current_quarter = d_current_year && d_qoy == CURRENT_QUARTER;
        let d_current_week = d_current_year && d_week_seq == CURRENT_WEEK;

        // Create the row
        let row = DateDimRow::new(
            null_bit_map,
            d_date_sk,
            d_date_id,
            date,
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
        );

        Ok(RowGeneratorResult::new(row))
    }

    fn consume_remaining_seeds_for_row(&mut self) {
        self.base.consume_remaining_seeds_for_row();
    }

    fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64) {
        self.base
            .skip_rows_until_starting_row_number(starting_row_number);
    }
}
