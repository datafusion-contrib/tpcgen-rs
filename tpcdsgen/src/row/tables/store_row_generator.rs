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

//! Store row generator (Slowly Changing Dimension)

use crate::config::Session;
use crate::distribution::{CallCenterDistributions, FirstNamesWeights, NamesDistributions};
use crate::error::Result;
use crate::generator::StoreGeneratorColumn;
use crate::nulls::create_null_bit_map;
use crate::random::RandomValueGenerator;
use crate::row::store_row::StoreRow;
use crate::row::{AbstractRowGenerator, RowGenerator, RowGeneratorResult};
use crate::slowly_changing_dimension_utils::{
    compute_scd_key, get_value_for_slowly_changing_dimension,
};
use crate::table::Table;
use crate::types::{Address, Date, Decimal};

const ROW_SIZE_S_MARKET_DESC: i32 = 100;
fn store_min_tax_percentage() -> Decimal {
    Decimal::new(0, 2).unwrap()
}
fn store_max_tax_percentage() -> Decimal {
    Decimal::new(11, 2).unwrap()
}
const STORE_MIN_DAYS_OPEN: i32 = 5;
const STORE_MAX_DAYS_OPEN: i32 = 500;
const STORE_CLOSED_PCT: i32 = 30;
const STORE_DESC_MIN: i32 = 15;

pub struct StoreRowGenerator {
    abstract_generator: AbstractRowGenerator,
    previous_row: Option<StoreRow>,
}

impl StoreRowGenerator {
    pub fn new() -> Self {
        StoreRowGenerator {
            abstract_generator: AbstractRowGenerator::new(Table::Store),
            previous_row: None,
        }
    }

    fn generate_store_row(&mut self, row_number: u64, session: &Session) -> Result<StoreRow> {
        use StoreGeneratorColumn::*;

        let row_number_i64 = i64::try_from(row_number).expect("row number fits in i64");

        // Generate null bit map first
        let stream = self
            .abstract_generator
            .get_random_number_stream(&WStoreNulls);
        let null_bit_map = create_null_bit_map(Table::Store, stream);

        let store_sk = row_number_i64;

        // Compute SCD key using S_STORE table
        let scd_key = compute_scd_key(Table::SStore, row_number);
        let store_id = scd_key.get_business_key().to_string();
        let rec_start_date_id = scd_key.get_start_date();
        let rec_end_date_id = scd_key.get_end_date();
        let is_new_business_key = scd_key.is_new_business_key();

        // Get field change flags
        let stream = self.abstract_generator.get_random_number_stream(&WStoreScd);
        let mut field_change_flags = stream.next_random() as i32;

        // Generate closed date
        let stream = self
            .abstract_generator
            .get_random_number_stream(&WStoreClosedDateId);
        let percentage = RandomValueGenerator::generate_uniform_random_int(1, 100, stream);
        let stream = self
            .abstract_generator
            .get_random_number_stream(&WStoreClosedDateId);
        let days_open = RandomValueGenerator::generate_uniform_random_int(
            STORE_MIN_DAYS_OPEN,
            STORE_MAX_DAYS_OPEN,
            stream,
        );
        let mut closed_date_id: i64 = if percentage < STORE_CLOSED_PCT {
            Date::JULIAN_DATE_MINIMUM as i64 + days_open as i64
        } else {
            -1
        };
        if let Some(ref prev_row) = self.previous_row {
            closed_date_id = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                prev_row.get_closed_date_id(),
                closed_date_id,
            );
        }
        field_change_flags >>= 1;

        // Generate store name
        let mut store_name = RandomValueGenerator::generate_word(
            row_number_i64,
            5,
            crate::distribution::get_syllables_distribution(),
        );
        if let Some(ref prev_row) = self.previous_row {
            store_name = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                prev_row.get_store_name().to_string(),
                store_name,
            );
        }
        field_change_flags >>= 1;

        // Generate employees
        let stream = self
            .abstract_generator
            .get_random_number_stream(&WStoreEmployees);
        let mut employees = RandomValueGenerator::generate_uniform_random_int(200, 300, stream);
        if let Some(ref prev_row) = self.previous_row {
            employees = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                prev_row.get_employees(),
                employees,
            );
        }
        field_change_flags >>= 1;

        // Generate floor space
        let stream = self
            .abstract_generator
            .get_random_number_stream(&WStoreFloorSpace);
        let mut floor_space =
            RandomValueGenerator::generate_uniform_random_int(5000000, 10000000, stream);
        if let Some(ref prev_row) = self.previous_row {
            floor_space = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                prev_row.get_floor_space(),
                floor_space,
            );
        }
        field_change_flags >>= 1;

        // Generate hours
        let stream = self
            .abstract_generator
            .get_random_number_stream(&WStoreHours);
        let hours = CallCenterDistributions::pick_random_call_center_hours(stream)?.to_string();
        field_change_flags >>= 1;

        // Generate store manager
        let weights = if session.is_sexist() {
            FirstNamesWeights::MaleFrequency
        } else {
            FirstNamesWeights::GeneralFrequency
        };
        let stream = self
            .abstract_generator
            .get_random_number_stream(&WStoreManager);
        let first_name = NamesDistributions::pick_random_first_name(weights, stream)?;
        let stream = self
            .abstract_generator
            .get_random_number_stream(&WStoreManager);
        let last_name = NamesDistributions::pick_random_last_name(stream)?;
        let mut store_manager = format!("{} {}", first_name, last_name);
        if let Some(ref prev_row) = self.previous_row {
            store_manager = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                prev_row.get_store_manager().to_string(),
                store_manager,
            );
        }
        field_change_flags >>= 1;

        // Generate market ID
        let stream = self
            .abstract_generator
            .get_random_number_stream(&WStoreMarketId);
        let mut market_id = RandomValueGenerator::generate_uniform_random_int(1, 10, stream);
        if let Some(ref prev_row) = self.previous_row {
            market_id = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                prev_row.get_market_id(),
                market_id,
            );
        }
        field_change_flags >>= 1;

        // Generate tax percentage
        let stream = self
            .abstract_generator
            .get_random_number_stream(&WStoreTaxPercentage);
        let mut d_tax_percentage = RandomValueGenerator::generate_uniform_random_decimal(
            store_min_tax_percentage(),
            store_max_tax_percentage(),
            stream,
        );
        if let Some(ref prev_row) = self.previous_row {
            d_tax_percentage = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                prev_row.get_d_tax_percentage(),
                d_tax_percentage,
            );
        }
        field_change_flags >>= 1;

        // Geography class is always "Unknown"
        let geography_class = "Unknown".to_string();
        field_change_flags >>= 1;

        // Generate market description
        let stream = self
            .abstract_generator
            .get_random_number_stream(&WStoreMarketDesc);
        let mut market_desc = RandomValueGenerator::generate_random_text(
            STORE_DESC_MIN,
            ROW_SIZE_S_MARKET_DESC,
            stream,
        );
        if let Some(ref prev_row) = self.previous_row {
            market_desc = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                prev_row.get_market_desc().to_string(),
                market_desc,
            );
        }
        field_change_flags >>= 1;

        // Generate market manager
        let stream = self
            .abstract_generator
            .get_random_number_stream(&WStoreMarketManager);
        let first_name = NamesDistributions::pick_random_first_name(weights, stream)?;
        let stream = self
            .abstract_generator
            .get_random_number_stream(&WStoreMarketManager);
        let last_name = NamesDistributions::pick_random_last_name(stream)?;
        let mut market_manager = format!("{} {}", first_name, last_name);
        if let Some(ref prev_row) = self.previous_row {
            market_manager = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                prev_row.get_market_manager().to_string(),
                market_manager,
            );
        }
        field_change_flags >>= 1;

        // Division and company are constant "Unknown" and 1
        let division_name = "Unknown".to_string();
        let division_id: i64 = 1;
        field_change_flags >>= 1; // divisionId
        field_change_flags >>= 1; // divisionName

        let company_name = "Unknown".to_string();
        let company_id: i64 = 1;
        field_change_flags >>= 1; // companyId
        field_change_flags >>= 1; // companyName

        // Generate address - many fields don't get updated due to C bug
        let stream = self
            .abstract_generator
            .get_random_number_stream(&WStoreAddress);
        let mut address =
            Address::make_address_for_column(Table::Store, stream, session.get_scaling())?;
        field_change_flags >>= 1; // city
        field_change_flags >>= 1; // county

        let mut gmt_offset = address.get_gmt_offset();
        if let Some(ref prev_row) = self.previous_row {
            gmt_offset = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                prev_row.get_address().get_gmt_offset(),
                gmt_offset,
            );
        }
        field_change_flags >>= 1;

        field_change_flags >>= 1; // state
        field_change_flags >>= 1; // streetType
        field_change_flags >>= 1; // streetName1
        field_change_flags >>= 1; // streetName2

        let mut street_number = address.get_street_number();
        if let Some(ref prev_row) = self.previous_row {
            street_number = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                prev_row.get_address().get_street_number(),
                street_number,
            );
        }
        field_change_flags >>= 1;

        let mut zip = address.get_zip();
        if let Some(ref prev_row) = self.previous_row {
            zip = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                prev_row.get_address().get_zip(),
                zip,
            );
        }

        // Create new address with updated SCD fields
        address = Address::new(
            address.get_suite_number().to_string(),
            street_number,
            address.get_street_name1().to_string(),
            address.get_street_name2().to_string(),
            address.get_street_type().to_string(),
            address.get_city().to_string(),
            address.get_county().map(|s| s.to_string()),
            address.get_state().to_string(),
            address.get_country().to_string(),
            zip,
            gmt_offset,
        )?;

        let row = StoreRow::new(
            null_bit_map,
            store_sk,
            store_id,
            rec_start_date_id,
            rec_end_date_id,
            closed_date_id,
            store_name,
            employees,
            floor_space,
            hours,
            store_manager,
            market_id,
            d_tax_percentage,
            geography_class,
            market_desc,
            market_manager,
            division_id,
            division_name,
            company_id,
            company_name,
            address,
        );

        Ok(row)
    }
}

impl Default for StoreRowGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl RowGenerator for StoreRowGenerator {
    fn generate_row_and_child_rows(
        &mut self,
        row_number: u64,
        session: &Session,
        _parent_row_generator: Option<&mut dyn RowGenerator>,
        _child_row_generator: Option<&mut dyn RowGenerator>,
    ) -> Result<RowGeneratorResult> {
        let row = self.generate_store_row(row_number, session)?;
        // Store for SCD logic on next row
        self.previous_row = Some(row.clone());
        Ok(RowGeneratorResult::new(row))
    }

    fn consume_remaining_seeds_for_row(&mut self) {
        self.abstract_generator.consume_remaining_seeds_for_row();
    }

    fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64) {
        self.abstract_generator
            .skip_rows_until_starting_row_number(starting_row_number);
    }
}
