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

use crate::row::table_row::NullLiteralField;
use std::fmt;

/// Household Demographics row data structure (HouseholdDemographicsRow)
/// Contains all fields for the HOUSEHOLD_DEMOGRAPHICS table in TPC-DS
#[derive(Debug, Clone, PartialEq)]
pub struct HouseholdDemographicsRow {
    pub(crate) hd_demo_sk: i64,
    pub(crate) hd_income_band_sk: i64,
    pub(crate) hd_buy_potential: String,
    pub(crate) hd_dep_count: i32,
    pub(crate) hd_vehicle_count: i32,
    null_bit_map: i64,
}

impl HouseholdDemographicsRow {
    /// Create a new builder for HouseholdDemographicsRow
    pub fn builder() -> HouseholdDemographicsRowBuilder {
        HouseholdDemographicsRowBuilder::new()
    }

    // Getter methods (matching Java implementation)
    pub fn get_hd_demo_sk(&self) -> i64 {
        self.hd_demo_sk
    }

    pub fn get_hd_income_band_sk(&self) -> i64 {
        self.hd_income_band_sk
    }

    pub fn get_hd_buy_potential(&self) -> &str {
        &self.hd_buy_potential
    }

    pub fn get_hd_dep_count(&self) -> i32 {
        self.hd_dep_count
    }

    pub fn get_hd_vehicle_count(&self) -> i32 {
        self.hd_vehicle_count
    }

    pub fn get_null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    pub fn null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    /// Check if a field should be null based on the null bitmap
    pub(crate) fn is_null(&self, column_position: i32) -> bool {
        (self.null_bit_map & (1 << column_position)) != 0
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator (no newline); NULL values/numerics print the literal `NULL`
/// (a Java quirk preserved for byte-for-byte compatibility).
impl fmt::Display for HouseholdDemographicsRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}|{}|",
            NullLiteralField::new(self.hd_demo_sk, self.is_null(0)),
            NullLiteralField::new(self.hd_income_band_sk, self.is_null(1)),
            NullLiteralField::new(&self.hd_buy_potential, self.is_null(2)),
            NullLiteralField::new(self.hd_dep_count, self.is_null(3)),
            NullLiteralField::new(self.hd_vehicle_count, self.is_null(4)),
        )
    }
}

/// Builder for HouseholdDemographicsRow (HouseholdDemographicsRow.Builder)
#[derive(Debug, Default)]
pub struct HouseholdDemographicsRowBuilder {
    hd_demo_sk: Option<i64>,
    hd_income_band_sk: Option<i64>,
    hd_buy_potential: Option<String>,
    hd_dep_count: Option<i32>,
    hd_vehicle_count: Option<i32>,
    null_bit_map: Option<i64>,
}

impl HouseholdDemographicsRowBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    // Setter methods (matching Java builder pattern)
    pub fn set_hd_demo_sk(mut self, value: i64) -> Self {
        self.hd_demo_sk = Some(value);
        self
    }

    pub fn set_hd_income_band_sk(mut self, value: i64) -> Self {
        self.hd_income_band_sk = Some(value);
        self
    }

    pub fn set_hd_buy_potential(mut self, value: String) -> Self {
        self.hd_buy_potential = Some(value);
        self
    }

    pub fn set_hd_dep_count(mut self, value: i32) -> Self {
        self.hd_dep_count = Some(value);
        self
    }

    pub fn set_hd_vehicle_count(mut self, value: i32) -> Self {
        self.hd_vehicle_count = Some(value);
        self
    }

    pub fn set_null_bit_map(mut self, value: i64) -> Self {
        self.null_bit_map = Some(value);
        self
    }

    /// Build the HouseholdDemographicsRow
    pub fn build(self) -> HouseholdDemographicsRow {
        HouseholdDemographicsRow {
            hd_demo_sk: self.hd_demo_sk.unwrap_or(0),
            hd_income_band_sk: self.hd_income_band_sk.unwrap_or(0),
            hd_buy_potential: self.hd_buy_potential.unwrap_or_default(),
            hd_dep_count: self.hd_dep_count.unwrap_or(0),
            hd_vehicle_count: self.hd_vehicle_count.unwrap_or(0),
            null_bit_map: self.null_bit_map.unwrap_or(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::row::dat_values;

    #[test]
    fn test_household_demographics_row_builder() {
        let row = HouseholdDemographicsRow::builder()
            .set_hd_demo_sk(1)
            .set_hd_income_band_sk(5)
            .set_hd_buy_potential("1001-5000".to_string())
            .set_hd_dep_count(3)
            .set_hd_vehicle_count(2)
            .set_null_bit_map(0)
            .build();

        // Test getters
        assert_eq!(row.get_hd_demo_sk(), 1);
        assert_eq!(row.get_hd_income_band_sk(), 5);
        assert_eq!(row.get_hd_buy_potential(), "1001-5000");
        assert_eq!(row.get_hd_dep_count(), 3);
        assert_eq!(row.get_hd_vehicle_count(), 2);
        assert_eq!(row.get_null_bit_map(), 0);
    }

    #[test]
    fn test_household_demographics_row_table_row() {
        let row = HouseholdDemographicsRow::builder()
            .set_hd_demo_sk(1)
            .set_hd_income_band_sk(5)
            .set_hd_buy_potential("1001-5000".to_string())
            .set_hd_dep_count(3)
            .set_hd_vehicle_count(2)
            .set_null_bit_map(0)
            .build();

        let values = dat_values(&row);
        assert_eq!(values.len(), 5); // 5 columns total
        assert_eq!(values[0], "1"); // hd_demo_sk
        assert_eq!(values[1], "5"); // hd_income_band_sk
        assert_eq!(values[2], "1001-5000"); // hd_buy_potential
        assert_eq!(values[3], "3"); // hd_dep_count
        assert_eq!(values[4], "2"); // hd_vehicle_count
    }

    #[test]
    fn test_household_demographics_row_clone_and_equality() {
        let row1 = HouseholdDemographicsRow::builder()
            .set_hd_demo_sk(42)
            .set_hd_buy_potential("501-1000".to_string())
            .set_hd_dep_count(1)
            .build();

        let row2 = row1.clone();
        assert_eq!(row1, row2);
        assert_eq!(row1.get_hd_demo_sk(), row2.get_hd_demo_sk());
        assert_eq!(row1.get_hd_buy_potential(), row2.get_hd_buy_potential());
    }

    #[test]
    fn test_builder_chaining() {
        // Test that builder methods can be chained in any order
        let row = HouseholdDemographicsRow::builder()
            .set_hd_vehicle_count(1)
            .set_hd_demo_sk(100)
            .set_hd_dep_count(0)
            .set_hd_income_band_sk(3)
            .set_hd_buy_potential("Unknown".to_string())
            .build();

        assert_eq!(row.get_hd_demo_sk(), 100);
        assert_eq!(row.get_hd_income_band_sk(), 3);
        assert_eq!(row.get_hd_buy_potential(), "Unknown");
        assert_eq!(row.get_hd_dep_count(), 0);
        assert_eq!(row.get_hd_vehicle_count(), 1);
    }

    #[test]
    fn test_null_handling() {
        // Test null bitmap handling - set bit 2 (hd_buy_potential)
        let row = HouseholdDemographicsRow::builder()
            .set_hd_demo_sk(1)
            .set_hd_income_band_sk(5)
            .set_hd_buy_potential("1001-5000".to_string())
            .set_hd_dep_count(3)
            .set_hd_vehicle_count(2)
            .set_null_bit_map(1 << 2) // Make hd_buy_potential null
            .build();

        let values = dat_values(&row);
        assert_eq!(values[0], "1"); // hd_demo_sk not null
        assert_eq!(values[1], "5"); // hd_income_band_sk not null
        assert_eq!(values[2], "NULL"); // hd_buy_potential is null
        assert_eq!(values[3], "3"); // hd_dep_count not null
        assert_eq!(values[4], "2"); // hd_vehicle_count not null
    }
}
