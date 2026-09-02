use crate::business_key_generator::make_business_key;
use crate::table::Table;
use crate::types::Date;

const ONE_HALF_DATE: i64 =
    Date::JULIAN_DATA_START_DATE + (Date::JULIAN_DATA_END_DATE - Date::JULIAN_DATA_START_DATE) / 2;
const ONE_THIRD_PERIOD: i64 = (Date::JULIAN_DATA_END_DATE - Date::JULIAN_DATA_START_DATE) / 3;
const ONE_THIRD_DATE: i64 = Date::JULIAN_DATA_START_DATE + ONE_THIRD_PERIOD;
const TWO_THIRDS_DATE: i64 = ONE_THIRD_DATE + ONE_THIRD_PERIOD;

#[derive(Debug, Clone)]
pub struct SlowlyChangingDimensionKey {
    business_key: String,
    start_date: i64,
    end_date: i64,
    is_new_business_key: bool,
}

impl SlowlyChangingDimensionKey {
    pub fn new(
        business_key: String,
        start_date: i64,
        end_date: i64,
        is_new_business_key: bool,
    ) -> Self {
        Self {
            business_key,
            start_date,
            end_date,
            is_new_business_key,
        }
    }

    pub fn get_business_key(&self) -> &str {
        &self.business_key
    }

    pub fn get_start_date(&self) -> i64 {
        self.start_date
    }

    pub fn get_end_date(&self) -> i64 {
        self.end_date
    }

    pub fn is_new_business_key(&self) -> bool {
        self.is_new_business_key
    }
}

pub fn compute_scd_key(table: Table, row_number: u64) -> SlowlyChangingDimensionKey {
    assert!(row_number > 0, "row number must be 1-based");
    let modulo = (row_number % 6) as i32;
    let table_number = table.get_ordinal(); // Use Java ordinal, not Rust enum discriminant

    let (business_key, start_date, mut end_date, is_new_key) = match modulo {
        1 => {
            // 1 revision
            let business_key = make_business_key(row_number);
            let start_date = Date::JULIAN_DATA_START_DATE - table_number * 6;
            let end_date = -1;
            (business_key, start_date, end_date, true)
        }
        2 => {
            // 1 of 2 revisions
            let business_key = make_business_key(row_number);
            let start_date = Date::JULIAN_DATA_START_DATE - table_number * 6;
            let end_date = ONE_HALF_DATE - table_number * 6;
            (business_key, start_date, end_date, true)
        }
        3 => {
            // 2 of 2 revisions
            let business_key = make_business_key(row_number - 1);
            let start_date = ONE_HALF_DATE - table_number * 6 + 1;
            let end_date = -1;
            (business_key, start_date, end_date, false)
        }
        4 => {
            // 1 of 3 revisions
            let business_key = make_business_key(row_number);
            let start_date = Date::JULIAN_DATA_START_DATE - table_number * 6;
            let end_date = ONE_THIRD_DATE - table_number * 6;
            (business_key, start_date, end_date, true)
        }
        5 => {
            // 2 of 3 revisions
            let business_key = make_business_key(row_number - 1);
            let start_date = ONE_THIRD_DATE - table_number * 6 + 1;
            let end_date = TWO_THIRDS_DATE - table_number * 6;
            (business_key, start_date, end_date, false)
        }
        0 => {
            // 3 of 3 revisions
            let business_key = make_business_key(row_number - 2);
            let start_date = TWO_THIRDS_DATE - table_number * 6 + 1;
            let end_date = -1;
            (business_key, start_date, end_date, false)
        }
        _ => panic!(
            "Something's wrong. Positive integers % 6 should always be covered by one of the cases"
        ),
    };

    if end_date > Date::JULIAN_DATA_END_DATE {
        end_date = -1;
    }

    SlowlyChangingDimensionKey::new(business_key, start_date, end_date, is_new_key)
}

pub fn get_value_for_slowly_changing_dimension<T>(
    field_change_flag: i32,
    is_new_key: bool,
    old_value: T,
    new_value: T,
) -> T {
    if should_change_dimension(field_change_flag, is_new_key) {
        new_value
    } else {
        old_value
    }
}

pub fn should_change_dimension(flags: i32, is_new_key: bool) -> bool {
    flags % 2 == 0 || is_new_key
}

/// Match surrogate key for SCD tables based on unique ID and julian date.
///
/// This converts a unique ID (which represents a business key) into the appropriate
/// surrogate key (row number) based on the date, accounting for SCD history revisions.
///
/// # Arguments
/// * `unique` - The unique business key ID (1-based)
/// * `julian_date` - The julian date for temporal matching
/// * `table` - The SCD table being referenced (config::Table)
/// * `scaling` - Scaling information for the table
///
/// # Returns
/// The surrogate key (row number) that matches the business key at the given date
pub fn match_surrogate_key(
    unique: i64,
    julian_date: i64,
    table: crate::config::Table,
    scaling: &crate::config::Scaling,
) -> i64 {
    let mut surrogate_key = (unique / 3) * 6;

    match unique % 3 {
        1 => {
            // Only one occurrence of this ID
            surrogate_key += 1;
        }
        2 => {
            // Two revisions of this ID
            surrogate_key += 2;
            if julian_date > ONE_HALF_DATE {
                surrogate_key += 1;
            }
        }
        0 => {
            // Three revisions of this ID
            surrogate_key -= 2;
            if julian_date > ONE_THIRD_DATE {
                surrogate_key += 1;
            }
            if julian_date > TWO_THIRDS_DATE {
                surrogate_key += 1;
            }
        }
        _ => panic!("unique % 3 did not equal 0, 1, or 2"),
    }

    let row_count = i64::try_from(scaling.get_row_count(table)).expect("row count fits in i64");
    if surrogate_key > row_count {
        surrogate_key = row_count;
    }

    surrogate_key
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Scaling;

    #[test]
    fn test_should_change_dimension() {
        assert!(should_change_dimension(0, false)); // even flag
        assert!(!should_change_dimension(1, false)); // odd flag
        assert!(should_change_dimension(1, true)); // new key overrides odd flag
        assert!(should_change_dimension(0, true)); // new key + even flag
    }

    #[test]
    #[should_panic(expected = "row number must be 1-based")]
    fn test_compute_scd_key_rejects_zero_row_number() {
        compute_scd_key(Table::Item, 0);
    }

    #[test]
    fn test_match_surrogate_key_single_revision() {
        let scaling = Scaling::new(1.0);
        // unique % 3 == 1 means single revision
        let surrogate = match_surrogate_key(
            1,
            Date::JULIAN_DATA_START_DATE,
            crate::config::Table::Item,
            &scaling,
        );
        assert_eq!(surrogate, 1); // (1/3)*6 + 1 = 0 + 1 = 1
    }

    #[test]
    fn test_match_surrogate_key_two_revisions() {
        let scaling = Scaling::new(1.0);
        // unique % 3 == 2 means two revisions
        // Before half date: surrogate_key = (unique/3)*6 + 2
        let surrogate = match_surrogate_key(
            2,
            Date::JULIAN_DATA_START_DATE,
            crate::config::Table::Item,
            &scaling,
        );
        assert_eq!(surrogate, 2); // (2/3)*6 + 2 = 0 + 2 = 2

        // After half date: surrogate_key = (unique/3)*6 + 2 + 1
        let surrogate =
            match_surrogate_key(2, ONE_HALF_DATE + 1, crate::config::Table::Item, &scaling);
        assert_eq!(surrogate, 3); // (2/3)*6 + 2 + 1 = 0 + 3 = 3
    }

    #[test]
    fn test_match_surrogate_key_three_revisions() {
        let scaling = Scaling::new(1.0);
        // unique % 3 == 0 means three revisions
        // Before one-third: (unique/3)*6 - 2
        let surrogate = match_surrogate_key(
            3,
            Date::JULIAN_DATA_START_DATE,
            crate::config::Table::Item,
            &scaling,
        );
        assert_eq!(surrogate, 4); // (3/3)*6 - 2 = 6 - 2 = 4

        // Between one-third and two-thirds: (unique/3)*6 - 2 + 1
        let surrogate =
            match_surrogate_key(3, ONE_THIRD_DATE + 1, crate::config::Table::Item, &scaling);
        assert_eq!(surrogate, 5); // (3/3)*6 - 2 + 1 = 5

        // After two-thirds: (unique/3)*6 - 2 + 1 + 1
        let surrogate =
            match_surrogate_key(3, TWO_THIRDS_DATE + 1, crate::config::Table::Item, &scaling);
        assert_eq!(surrogate, 6); // (3/3)*6 - 2 + 2 = 6
    }

    #[test]
    fn test_match_surrogate_key_capped_at_row_count() {
        let scaling = Scaling::new(1.0);
        // For a very large unique ID, surrogate should be capped at row count
        let row_count = i64::try_from(scaling.get_row_count(crate::config::Table::Item))
            .expect("row count fits in i64");
        let large_unique = 100000;
        let surrogate = match_surrogate_key(
            large_unique,
            Date::JULIAN_DATA_START_DATE,
            crate::config::Table::Item,
            &scaling,
        );
        assert_eq!(surrogate, row_count);
    }
}
