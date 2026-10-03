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

use crate::config::Session;
use crate::distribution::DemographicsDistributions;
use crate::error::Result;
use crate::generator::HouseholdDemographicsGeneratorColumn;
use crate::random::RandomValueGenerator;
use crate::row::{AbstractRowGenerator, HouseholdDemographicsRow};
use crate::table::Table;

/// Row generator for the HOUSEHOLD_DEMOGRAPHICS table (HouseholdDemographicsRowGenerator)
pub struct HouseholdDemographicsRowGenerator {
    abstract_generator: AbstractRowGenerator,
    current_row: u64,
    row_count: u64,
}

impl HouseholdDemographicsRowGenerator {
    /// Generate source rows `1..=row_count`.
    pub fn new(_session: Session, row_count: u64) -> Self {
        Self {
            abstract_generator: AbstractRowGenerator::new(Table::HouseholdDemographics),
            current_row: 1,
            row_count,
        }
    }

    /// Generate a HouseholdDemographicsRow with realistic data following Java implementation
    fn generate_household_demographics_row(
        &mut self,
        row_number: u64,
    ) -> Result<HouseholdDemographicsRow> {
        let row_number_i64 = i64::try_from(row_number).expect("row number fits in i64");

        // Create null bit map (createNullBitMap call)
        let nulls_stream = self
            .abstract_generator
            .get_random_number_stream(&HouseholdDemographicsGeneratorColumn::HdNulls);
        let threshold = RandomValueGenerator::generate_uniform_random_int(0, 9999, nulls_stream);
        let bit_map =
            RandomValueGenerator::generate_uniform_random_key(1, i32::MAX as i64, nulls_stream);

        // Calculate null_bit_map based on threshold and table's not-null bitmap (Nulls.createNullBitMap)
        let null_bit_map = if threshold < Table::HouseholdDemographics.get_null_basis_points() {
            bit_map & !Table::HouseholdDemographics.get_not_null_bit_map()
        } else {
            0
        };

        // Generate household demographics using index-based cartesian product (algorithm from Java)
        let hd_demo_sk = row_number_i64;
        let mut index = hd_demo_sk;

        // Get income band id using modulo
        let hd_income_band_sk =
            (index % DemographicsDistributions::get_income_band_size() as i64) + 1;
        index /= DemographicsDistributions::get_income_band_size() as i64;

        // Get buy potential and divide index
        let hd_buy_potential =
            DemographicsDistributions::get_buy_potential_for_index_mod_size(index);
        index /= DemographicsDistributions::get_buy_potential_size() as i64;

        // Get dependent count and divide index
        let hd_dep_count = DemographicsDistributions::get_dep_count_for_index_mod_size(index);
        index /= DemographicsDistributions::get_dep_count_size() as i64;

        // Get vehicle count (no division needed, last in sequence)
        let hd_vehicle_count =
            DemographicsDistributions::get_vehicle_count_for_index_mod_size(index);

        Ok(HouseholdDemographicsRow::builder()
            .set_hd_demo_sk(hd_demo_sk)
            .set_hd_income_band_sk(hd_income_band_sk)
            .set_hd_buy_potential(hd_buy_potential.to_string())
            .set_hd_dep_count(hd_dep_count)
            .set_hd_vehicle_count(hd_vehicle_count)
            .set_null_bit_map(null_bit_map)
            .build())
    }

    /// Start generating at `starting_row_number` (1-based), fast forwarding
    /// the random number streams to that row.
    pub fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64) {
        self.abstract_generator
            .skip_rows_until_starting_row_number(starting_row_number);
        self.current_row = starting_row_number;
    }

    /// Restrict generation to source rows
    /// `starting_row_number..=ending_row_number` (1-based, inclusive).
    ///
    /// The ending row number is clamped to the table's row count.
    pub fn set_source_row_range(&mut self, starting_row_number: u64, ending_row_number: u64) {
        self.skip_rows_until_starting_row_number(starting_row_number);
        self.row_count = self.row_count.min(ending_row_number);
    }
}

impl Iterator for HouseholdDemographicsRowGenerator {
    type Item = HouseholdDemographicsRow;

    fn next(&mut self) -> Option<HouseholdDemographicsRow> {
        if self.current_row > self.row_count {
            return None;
        }
        let row = self
            .generate_household_demographics_row(self.current_row)
            .expect("row gen");
        self.abstract_generator.consume_remaining_seeds_for_row();
        self.current_row += 1;
        Some(row)
    }
}
