use crate::business_key_generator::make_business_key;
use crate::config::Session;
use crate::distribution::HoursDistribution;
use crate::error::Result;
use crate::row::{AbstractRowGenerator, TimeDimRow};
use crate::table::Table;

pub struct TimeDimRowGenerator {
    base: AbstractRowGenerator,
}

impl TimeDimRowGenerator {
    /// Generate source rows `1..=row_count`.
    pub fn new(_session: Session, row_count: u64) -> Self {
        TimeDimRowGenerator {
            base: AbstractRowGenerator::new(Table::TimeDim, row_count),
        }
    }

    /// Start generating at `starting_row_number` (1-based), fast forwarding
    /// the random number streams to that row.
    pub fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64) {
        self.base
            .skip_rows_until_starting_row_number(starting_row_number);
    }

    /// Restrict generation to source rows
    /// `starting_row_number..=ending_row_number` (1-based, inclusive).
    ///
    /// The ending row number is clamped to the table's row count.
    pub fn set_source_row_range(&mut self, starting_row_number: u64, ending_row_number: u64) {
        self.base
            .set_source_row_range(starting_row_number, ending_row_number);
    }

    fn generate_time_dim_row(&mut self, row_number: u64) -> Result<TimeDimRow> {
        let row_number_i64 = i64::try_from(row_number).expect("row number fits in i64");

        // Create null bitmap - TimeDim has very few nulls
        let null_bit_map = 0i64;

        // Row number represents seconds since midnight (0-based)
        let t_time_sk = row_number_i64 - 1;
        let t_time_id = make_business_key(row_number);
        let t_time = (row_number_i64 - 1) as i32;

        // Extract time components
        let mut time_temp = t_time as i64;
        let t_second = (time_temp % 60) as i32;
        time_temp /= 60;
        let t_minute = (time_temp % 60) as i32;
        time_temp /= 60;
        let t_hour = (time_temp % 24) as i32;

        // Get hour information for shift and meal time
        let hour_info = HoursDistribution::get_hour_info_for_hour(t_hour);
        let t_am_pm = hour_info.get_am_pm().to_string();
        let t_shift = hour_info.get_shift().to_string();
        let t_sub_shift = hour_info.get_sub_shift().to_string();
        let t_meal_time = hour_info.get_meal().to_string();

        // Create the row
        let row = TimeDimRow::new(
            null_bit_map,
            t_time_sk,
            t_time_id,
            t_time,
            t_hour,
            t_minute,
            t_second,
            t_am_pm,
            t_shift,
            t_sub_shift,
            t_meal_time,
        );

        Ok(row)
    }
}

impl Iterator for TimeDimRowGenerator {
    type Item = TimeDimRow;

    fn next(&mut self) -> Option<TimeDimRow> {
        let row_number = self.base.next_row_number()?;
        let row = self.generate_time_dim_row(row_number).expect("row gen");
        self.base.finish_row();
        Some(row)
    }
}
