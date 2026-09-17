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

//! Customer address row generator (CustomerAddressRowGenerator)

use crate::business_key_generator::make_business_key;
use crate::config::Session;
use crate::distribution::location_types_distribution::{
    LocationTypeWeights, LocationTypesDistribution,
};
use crate::error::Result;
use crate::generator::CustomerAddressGeneratorColumn;
use crate::nulls::create_null_bit_map;
use crate::row::{AbstractRowGenerator, CustomerAddressRow, RowGenerator, RowGeneratorResult};
use crate::table::Table;
use crate::types::Address;

/// Row generator for the CUSTOMER_ADDRESS table (CustomerAddressRowGenerator)
pub struct CustomerAddressRowGenerator {
    abstract_generator: AbstractRowGenerator,
}

impl Default for CustomerAddressRowGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomerAddressRowGenerator {
    /// Create a new CustomerAddressRowGenerator
    pub fn new() -> Self {
        Self {
            abstract_generator: AbstractRowGenerator::new(Table::CustomerAddress),
        }
    }

    /// Generate a CustomerAddressRow with realistic data following Java implementation
    fn generate_customer_address_row(
        &mut self,
        row_number: u64,
        session: &Session,
    ) -> Result<CustomerAddressRow> {
        let row_number_i64 = i64::try_from(row_number).expect("row number fits in i64");

        // Create null bit map (createNullBitMap call)
        let nulls_stream = self
            .abstract_generator
            .get_random_number_stream(&CustomerAddressGeneratorColumn::CaNulls);
        let null_bit_map = create_null_bit_map(Table::CustomerAddress, nulls_stream);

        let ca_addr_sk = row_number_i64;
        let ca_addr_id = make_business_key(row_number);

        // Generate address
        let scaling = session.get_scaling();
        let address_stream = self
            .abstract_generator
            .get_random_number_stream(&CustomerAddressGeneratorColumn::CaAddress);
        let ca_address =
            Address::make_address_for_column(Table::CustomerAddress, address_stream, scaling)?;

        // Generate location type using UNIFORM weights (matches Java)
        let location_type_stream = self
            .abstract_generator
            .get_random_number_stream(&CustomerAddressGeneratorColumn::CaLocationType);
        let ca_location_type = LocationTypesDistribution::pick_random_location_type(
            LocationTypeWeights::Uniform,
            location_type_stream,
        )?;

        Ok(CustomerAddressRow::new(
            null_bit_map,
            ca_addr_sk,
            ca_addr_id,
            ca_address,
            ca_location_type,
        ))
    }
}

impl RowGenerator for CustomerAddressRowGenerator {
    fn generate_row_and_child_rows(
        &mut self,
        row_number: u64,
        session: &Session,
        _parent_row_generator: Option<&mut dyn RowGenerator>,
        _child_row_generator: Option<&mut dyn RowGenerator>,
    ) -> Result<RowGeneratorResult> {
        let row = self.generate_customer_address_row(row_number, session)?;
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
