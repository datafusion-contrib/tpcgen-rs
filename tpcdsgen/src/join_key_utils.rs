/*
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! Join key generation utilities for TPC-DS foreign key relationships.
//!
//! This module provides functionality to generate foreign keys (join keys) between
//! TPC-DS tables, respecting the benchmark's referential integrity requirements.
use crate::config::{Scaling, Table};
use crate::distribution::calendar_distribution::{CalendarDistribution, CalendarWeights};
use crate::distribution::catalog_page_distributions::CatalogPageTypesDistribution;
use crate::distribution::hours_distribution::{HoursDistribution, HoursWeights};
use crate::error::{Result, TpcdsError};
use crate::generator::GeneratorColumn;
use crate::pseudo_table_scaling_infos::PseudoTableScalingInfos;
use crate::random::{RandomNumberStream, RandomValueGenerator};
use crate::slowly_changing_dimension_utils;
use crate::types::Date;

#[allow(dead_code)]
const WEB_PAGES_PER_SITE: i32 = 123;
#[allow(dead_code)]
const WEB_DATE_STAGGER: i64 = 17;
const CS_MIN_SHIP_DELAY: i32 = 2;
const CS_MAX_SHIP_DELAY: i32 = 90;
const CATALOGS_PER_YEAR: i32 = 18;

/// Generates a join key (foreign key) from one table/column to another table.
///
/// This is the main entry point for generating foreign keys between TPC-DS tables.
/// It routes to specialized key generators based on the target table type.
///
/// # Arguments
///
/// * `from_column` - The column generating the join key
/// * `random_number_stream` - Random number stream for generation
/// * `to_table` - The target table being referenced
/// * `join_count` - Context-dependent value (often a date or row number)
/// * `scaling` - Scaling information for the dataset
///
/// # Returns
///
/// The generated join key value, or -1 if no valid key can be generated
pub fn generate_join_key(
    from_column: &dyn GeneratorColumn,
    random_number_stream: &mut dyn RandomNumberStream,
    to_table: Table,
    join_count: i64,
    scaling: &Scaling,
) -> Result<i64> {
    // NOTE: to_table is config::Table (for CLI), from_column.get_table() returns
    // column::Table (now same as table::Table). Different enums for different purposes.

    match to_table {
        Table::CatalogPage => {
            generate_catalog_page_join_key(random_number_stream, join_count, scaling)
        }
        Table::DateDim => {
            let year = RandomValueGenerator::generate_uniform_random_int(
                Date::DATE_MINIMUM.year(),
                Date::DATE_MAXIMUM.year(),
                random_number_stream,
            );
            generate_date_join_key(random_number_stream, from_column, join_count, year, scaling)
        }
        Table::TimeDim => generate_time_join_key(from_column, random_number_stream),
        _ => {
            if to_table.keeps_history() {
                generate_scd_join_key(to_table, random_number_stream, join_count, scaling)
            } else {
                Ok(RandomValueGenerator::generate_uniform_random_key(
                    1,
                    i64::try_from(scaling.get_row_count(to_table)).expect("row count fits in i64"),
                    random_number_stream,
                ))
            }
        }
    }
}

/// Generates a join key to the catalog_page table.
///
/// Calculates which catalog page based on the date and catalog type (monthly, bi-annual, quarterly).
/// Each catalog type has a different frequency within the year.
///
/// Based on JoinKeyUtils.java:generateCatalogPageJoinKey
fn generate_catalog_page_join_key(
    random_number_stream: &mut dyn RandomNumberStream,
    julian_date: i64,
    scaling: &Scaling,
) -> Result<i64> {
    let pages_per_catalog = ((scaling.get_row_count(Table::CatalogPage) / CATALOGS_PER_YEAR as u64)
        / (Date::DATE_MAXIMUM.year() - Date::DATE_MINIMUM.year() + 2) as u64)
        as i32;

    let catalog_type =
        CatalogPageTypesDistribution::pick_random_catalog_page_type(random_number_stream)?;
    let page = RandomValueGenerator::generate_uniform_random_int(
        1,
        pages_per_catalog,
        random_number_stream,
    );

    let offset_from_start = (julian_date - Date::JULIAN_DATA_START_DATE - 1) as i32;
    let mut count = (offset_from_start / 365) * CATALOGS_PER_YEAR;
    let offset = offset_from_start % 365;

    // Adjust count based on catalog type frequency
    match catalog_type.as_str() {
        "bi-annual" => {
            if offset > 183 {
                count += 1;
            }
        }
        "quarterly" => {
            count += offset / 91;
        }
        "monthly" => {
            count += offset / 31;
        }
        _ => {
            return Err(TpcdsError::new(&format!(
                "Invalid catalog_page_type: {}",
                catalog_type
            )));
        }
    }

    Ok((count * pages_per_catalog + page) as i64)
}

/// Generates a join key to the date_dim table.
///
/// Different table types use different date selection strategies:
/// - Sales tables use SALES or SALES_LEAP_YEAR weights
/// - Returns tables use date returns logic (with lag from sale date)
/// - Web-related tables use web join key logic
/// - Other tables use UNIFORM or UNIFORM_LEAP_YEAR weights
///
/// Based on JoinKeyUtils.java:generateDateJoinKey (lines 109-142)
fn generate_date_join_key(
    random_number_stream: &mut dyn RandomNumberStream,
    from_column: &dyn GeneratorColumn,
    join_count: i64,
    year: i32,
    scaling: &Scaling,
) -> Result<i64> {
    use crate::column::Table as ColumnTable;

    let from_table = from_column.get_table();

    // Check if this is a WEB_SITE or WEB_PAGE table
    if from_table == ColumnTable::WebPage || from_table == ColumnTable::WebSite {
        return generate_web_join_key(from_column, random_number_stream, join_count, scaling);
    }

    // Returns tables: date is computed as sale_date + lag
    // Based on JoinKeyUtils.java:generateDateReturnsJoinKey (lines 192-211)
    if from_table == ColumnTable::StoreReturns
        || from_table == ColumnTable::CatalogReturns
        || from_table == ColumnTable::WebReturns
    {
        return generate_date_returns_join_key(from_table, random_number_stream, join_count);
    }

    // Sales tables: use SALES weights
    // Default: use SALES weights (most common case)
    let weights = if Date::is_leap_year(year) {
        CalendarWeights::SalesLeapYear
    } else {
        CalendarWeights::Sales
    };

    let day_number = CalendarDistribution::pick_random_day_of_year(weights, random_number_stream)?;
    let result = Date::to_julian_days(&Date::new(year, 1, 1)) as i64 + day_number as i64;
    Ok(if result > Date::JULIAN_TODAYS_DATE as i64 {
        -1
    } else {
        result
    })
}

/// Generates a date join key for returns tables.
///
/// Returns have a lag between the sale date and return date.
/// The lag is calculated as: random(min*2, max*2) days after the sale.
///
/// Based on JoinKeyUtils.java:generateDateReturnsJoinKey (lines 192-211)
fn generate_date_returns_join_key(
    from_table: crate::column::Table,
    random_number_stream: &mut dyn RandomNumberStream,
    join_count: i64, // This is the sale date (julian days)
) -> Result<i64> {
    use crate::column::Table as ColumnTable;

    let (min, max) = match from_table {
        ColumnTable::StoreReturns | ColumnTable::CatalogReturns => {
            (CS_MIN_SHIP_DELAY, CS_MAX_SHIP_DELAY)
        }
        ColumnTable::WebReturns => (1, 120), // Web returns have 1-120 day ship lag
        _ => {
            return Err(TpcdsError::new(&format!(
                "Invalid table for date returns join: {:?}",
                from_table
            )))
        }
    };

    let lag =
        RandomValueGenerator::generate_uniform_random_int(min * 2, max * 2, random_number_stream);
    Ok(join_count + lag as i64)
}

/// Generates a join key to the time_dim table.
///
/// Different table types use different hour selection strategies:
/// - Store sales/returns use STORE weights (typical store hours)
/// - Catalog/web sales/returns use CATALOG_AND_WEB weights (24/7 operations)
/// - Other tables use UNIFORM weights
///
/// Returns seconds since midnight (0 to 86399).
///
/// Based on JoinKeyUtils.java:generateTimeJoinKey (lines 213-235)
fn generate_time_join_key(
    from_column: &dyn GeneratorColumn,
    random_number_stream: &mut dyn RandomNumberStream,
) -> Result<i64> {
    use crate::column::Table as ColumnTable;

    let from_table = from_column.get_table();

    let weights = match from_table {
        ColumnTable::StoreSales | ColumnTable::StoreReturns => HoursWeights::Store,
        ColumnTable::CatalogSales
        | ColumnTable::CatalogReturns
        | ColumnTable::WebSales
        | ColumnTable::WebReturns => HoursWeights::CatalogAndWeb,
        _ => HoursWeights::Uniform,
    };

    let hour = HoursDistribution::pick_random_hour(weights, random_number_stream)?;
    let seconds = RandomValueGenerator::generate_uniform_random_int(0, 3599, random_number_stream);

    Ok((hour as i64 * 3600) + seconds as i64)
}

/// Generates a join key to a slowly changing dimension (SCD) table.
///
/// SCD tables keep history, so the join key must match the appropriate
/// version of the dimension based on the effective date.
fn generate_scd_join_key(
    to_table: Table,
    random_number_stream: &mut dyn RandomNumberStream,
    julian_date: i64,
    scaling: &Scaling,
) -> Result<i64> {
    // Can't have a revision in the future
    if julian_date > Date::JULIAN_DATA_END_DATE {
        return Ok(-1);
    }

    let id_count = i64::try_from(scaling.get_id_count(to_table)).expect("ID count fits in i64");
    let unique_key =
        RandomValueGenerator::generate_uniform_random_key(1, id_count, random_number_stream);

    // Match the surrogate key based on the julian date for SCD tables
    let key = slowly_changing_dimension_utils::match_surrogate_key(
        unique_key,
        julian_date,
        to_table,
        scaling,
    );

    Ok(
        if key > i64::try_from(scaling.get_row_count(to_table)).expect("row count fits in i64") {
            -1
        } else {
            key
        },
    )
}

/// Generates a join key for web-related tables (web_site, web_page).
///
/// Web tables have complex date logic involving site creation, open, and close dates.
/// Based on JoinKeyUtils.java:generateWebJoinKey (lines 144-175)
fn generate_web_join_key(
    from_column: &dyn GeneratorColumn,
    random_number_stream: &mut dyn RandomNumberStream,
    join_key: i64,
    scaling: &Scaling,
) -> Result<i64> {
    let global_column_number = from_column.get_global_column_number();

    // WP_CREATION_DATE_SK (global column 371)
    if global_column_number == 371 {
        // Page creation has to happen outside of the page window, to assure a constant number of pages,
        // so it occurs in the gap between site creation and the site's actual activity. For sites that are replaced
        // in the time span of the data set, this will depend on whether they are the first version or the second
        let site = (join_key / WEB_PAGES_PER_SITE as i64 + 1) as i32;
        let web_site_duration = get_web_site_duration(scaling);
        let min_result = Date::JULIAN_DATE_MINIMUM as i64
            - ((site as i64 * WEB_DATE_STAGGER) % web_site_duration / 2);
        return Ok(RandomValueGenerator::generate_uniform_random_int(
            min_result as i32,
            Date::JULIAN_DATE_MINIMUM,
            random_number_stream,
        ) as i64);
    }

    // WEB_OPEN_DATE for WebPage (global column 340) or WebSite (global column 452)
    if global_column_number == 340 || global_column_number == 452 {
        let web_site_duration = get_web_site_duration(scaling);
        return Ok(Date::JULIAN_DATE_MINIMUM as i64
            - ((join_key * WEB_DATE_STAGGER) % web_site_duration / 2));
    }

    // WEB_CLOSE_DATE for WebPage (global column 341) or WebSite (global column 453)
    if global_column_number == 341 || global_column_number == 453 {
        let web_site_duration = get_web_site_duration(scaling);
        let mut result = Date::JULIAN_DATE_MINIMUM as i64
            - ((join_key * WEB_DATE_STAGGER) % web_site_duration / 2);
        result += -web_site_duration; // the -1 here and below are due to undefined values in the C code

        // the site is completely replaced, and this is the first site
        if is_replaced(join_key) && !is_replacement(join_key) {
            // the close date of the first site needs to align on a revision boundary
            result -= -web_site_duration / 2;
        }
        return Ok(result);
    }

    Err(TpcdsError::new(&format!(
        "Invalid column for web join: global column {}",
        global_column_number
    )))
}

/// Calculates the duration of a web site based on concurrent sites.
/// Based on JoinKeyUtils.java:getWebSiteDuration (lines 177-180)
fn get_web_site_duration(scaling: &Scaling) -> i64 {
    let concurrent_web_sites = PseudoTableScalingInfos::get_concurrent_web_sites();
    let row_count = concurrent_web_sites
        .get_row_count_for_scale(scaling.get_scale())
        .expect("Failed to get row count for concurrent web sites");

    (Date::JULIAN_DATE_MAXIMUM as i64 - Date::JULIAN_DATE_MINIMUM as i64) * row_count
}

/// Checks if a web site is replaced (has an even join key).
/// Based on JoinKeyUtils.java:isReplaced (lines 182-185)
fn is_replaced(join_key: i64) -> bool {
    (join_key % 2) == 0
}

/// Checks if a web site is a replacement (odd division by 2).
/// Based on JoinKeyUtils.java:isReplacement (lines 187-190)
fn is_replacement(join_key: i64) -> bool {
    (join_key / 2 % 2) != 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::RandomNumberStreamImpl;

    #[test]
    fn test_is_replaced() {
        assert!(is_replaced(0));
        assert!(is_replaced(2));
        assert!(is_replaced(4));
        assert!(!is_replaced(1));
        assert!(!is_replaced(3));
    }

    #[test]
    fn test_is_replacement() {
        assert!(!is_replacement(0)); // 0/2=0, 0%2=0
        assert!(!is_replacement(1)); // 1/2=0, 0%2=0
        assert!(is_replacement(2)); // 2/2=1, 1%2=1
        assert!(is_replacement(3)); // 3/2=1, 1%2=1
        assert!(!is_replacement(4)); // 4/2=2, 2%2=0
    }

    #[test]
    fn test_generate_time_join_key() {
        use crate::generator::StoreSalesGeneratorColumn;
        let mut stream = RandomNumberStreamImpl::new(1).unwrap();
        let column = StoreSalesGeneratorColumn::SsSoldTimeSk;
        let result = generate_time_join_key(&column, &mut stream).unwrap();

        // Time keys should be in range [0, 86400) seconds in a day
        assert!(
            (0..86400).contains(&result),
            "Time key should be valid seconds in day"
        );
    }

    #[test]
    fn test_generate_time_join_key_deterministic() {
        use crate::generator::StoreSalesGeneratorColumn;
        let mut stream1 = RandomNumberStreamImpl::new(1).unwrap();
        let mut stream2 = RandomNumberStreamImpl::new(1).unwrap();
        let column = StoreSalesGeneratorColumn::SsSoldTimeSk;

        let result1 = generate_time_join_key(&column, &mut stream1).unwrap();
        let result2 = generate_time_join_key(&column, &mut stream2).unwrap();

        assert_eq!(result1, result2, "Same seed should produce same time key");
    }

    #[test]
    fn test_catalog_page_join_key() {
        let mut stream = RandomNumberStreamImpl::new(1).unwrap();
        let scaling = Scaling::new(1.0);

        // Catalog page join key is now implemented (CatalogPageTypesDistribution ported)
        let result = generate_catalog_page_join_key(&mut stream, 2451545, &scaling);
        assert!(result.is_ok(), "Catalog page join should work now");

        let key = result.unwrap();
        assert!(key > 0, "Key should be positive");
    }

    // NOTE: Test disabled until column::Table vs config::Table is resolved
    // #[test]
    // fn test_generate_date_returns_join_key() {
    //     let mut stream = RandomNumberStreamImpl::new(1).unwrap();
    //     let sale_date = Date::to_julian_days(&Date::new(2003, 1, 1)) as i64;
    //
    //     let return_date = _generate_date_returns_join_key(
    //         Table::StoreReturns,
    //         &mut stream,
    //         sale_date,
    //     )
    //     .unwrap();
    //
    //     // Return should be after sale
    //     assert!(return_date > sale_date, "Return date should be after sale date");
    //
    //     // Lag should be within expected range
    //     let lag = return_date - sale_date;
    //     assert!(lag >= (CS_MIN_SHIP_DELAY * 2) as i64 && lag <= (CS_MAX_SHIP_DELAY * 2) as i64);
    // }
}
