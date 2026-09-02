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

//! Store returns row generator

use crate::config::Session;
use crate::error::Result;
use crate::generator::StoreReturnsGeneratorColumn;
use crate::join_key_utils::generate_join_key;
use crate::nulls::create_null_bit_map;
use crate::random::RandomValueGenerator;
use crate::row::store_returns_row::StoreReturnsRow;
use crate::row::store_sales_row::StoreSalesRow;
use crate::row::{AbstractRowGenerator, GeneratedRow, RowGenerator, RowGeneratorResult};
use crate::table::Table;
use crate::types::generate_pricing_for_returns_table;

/// Percentage of returns where the same customer returns the item
const SR_SAME_CUSTOMER: i32 = 80;

pub struct StoreReturnsRowGenerator {
    abstract_generator: AbstractRowGenerator,
}

impl StoreReturnsRowGenerator {
    pub fn new() -> Self {
        StoreReturnsRowGenerator {
            abstract_generator: AbstractRowGenerator::new(Table::StoreReturns),
        }
    }

    /// Generate a return row from a sales row
    /// This is called by StoreSalesRowGenerator when a sale is returned
    pub fn generate_row(
        &mut self,
        session: &Session,
        sales_row: &StoreSalesRow,
    ) -> Result<GeneratedRow> {
        use StoreReturnsGeneratorColumn::*;

        let scaling = session.get_scaling();

        // Generate null bit map
        let stream = self.abstract_generator.get_random_number_stream(&SrNulls);
        let null_bit_map = create_null_bit_map(Table::StoreReturns, stream);

        // Some of the information in the return is taken from the original sale
        let sr_ticket_number = sales_row.get_ss_ticket_number();
        let sr_item_sk = sales_row.get_ss_sold_item_sk();

        // Some fields are conditionally taken from the sale (80% same customer)
        let stream = self
            .abstract_generator
            .get_random_number_stream(&SrCustomerSk);
        let mut sr_customer_sk = generate_join_key(
            &SrCustomerSk,
            stream,
            crate::config::Table::Customer,
            1,
            scaling,
        )?;
        let stream = self
            .abstract_generator
            .get_random_number_stream(&SrTicketNumber);
        let random_int = RandomValueGenerator::generate_uniform_random_int(1, 100, stream);
        if random_int < SR_SAME_CUSTOMER {
            sr_customer_sk = sales_row.get_ss_sold_customer_sk();
        }

        // The rest of the columns are generated for this specific return
        let stream = self
            .abstract_generator
            .get_random_number_stream(&SrReturnedDateSk);
        let sr_returned_date_sk = generate_join_key(
            &SrReturnedDateSk,
            stream,
            crate::config::Table::DateDim,
            sales_row.get_ss_sold_date_sk(),
            scaling,
        )?;

        // Return time is between 8am and 5pm (8*3600-1 to 17*3600-1 seconds)
        let stream = self
            .abstract_generator
            .get_random_number_stream(&SrReturnedTimeSk);
        let sr_returned_time_sk =
            RandomValueGenerator::generate_uniform_random_int(8 * 3600 - 1, 17 * 3600 - 1, stream)
                as i64;

        let stream = self.abstract_generator.get_random_number_stream(&SrCdemoSk);
        let sr_cdemo_sk = generate_join_key(
            &SrCdemoSk,
            stream,
            crate::config::Table::CustomerDemographics,
            1,
            scaling,
        )?;

        let stream = self.abstract_generator.get_random_number_stream(&SrHdemoSk);
        let sr_hdemo_sk = generate_join_key(
            &SrHdemoSk,
            stream,
            crate::config::Table::HouseholdDemographics,
            1,
            scaling,
        )?;

        let stream = self.abstract_generator.get_random_number_stream(&SrAddrSk);
        let sr_addr_sk = generate_join_key(
            &SrAddrSk,
            stream,
            crate::config::Table::CustomerAddress,
            1,
            scaling,
        )?;

        let stream = self.abstract_generator.get_random_number_stream(&SrStoreSk);
        let sr_store_sk =
            generate_join_key(&SrStoreSk, stream, crate::config::Table::Store, 1, scaling)?;

        let stream = self
            .abstract_generator
            .get_random_number_stream(&SrReasonSk);
        let sr_reason_sk = generate_join_key(
            &SrReasonSk,
            stream,
            crate::config::Table::Reason,
            1,
            scaling,
        )?;

        // Generate return quantity (1 to original sale quantity)
        let sales_pricing = sales_row.get_ss_pricing();
        let stream = self.abstract_generator.get_random_number_stream(&SrPricing);
        let quantity = RandomValueGenerator::generate_uniform_random_int(
            1,
            sales_pricing.get_quantity(),
            stream,
        );

        // Generate return pricing
        let stream = self.abstract_generator.get_random_number_stream(&SrPricing);
        let sr_pricing = generate_pricing_for_returns_table(stream, quantity, sales_pricing);

        Ok(StoreReturnsRow::new(
            null_bit_map,
            sr_returned_date_sk,
            sr_returned_time_sk,
            sr_item_sk,
            sr_customer_sk,
            sr_cdemo_sk,
            sr_hdemo_sk,
            sr_addr_sk,
            sr_store_sk,
            sr_reason_sk,
            sr_ticket_number,
            sr_pricing,
        )
        .into())
    }
}

impl Default for StoreReturnsRowGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl RowGenerator for StoreReturnsRowGenerator {
    fn generate_row_and_child_rows(
        &mut self,
        _row_number: u64,
        _session: &Session,
        _parent_row_generator: Option<&mut dyn RowGenerator>,
        _child_row_generator: Option<&mut dyn RowGenerator>,
    ) -> Result<RowGeneratorResult> {
        // The store_returns table is a child of the store_sales table because you can only
        // return things that have already been purchased. This method should only get called
        // if we are generating the store_returns table in isolation.
        // Otherwise store_returns is generated during the generation of the store_sales table
        // via the generate_row method above.
        //
        // For now, we panic if called directly - the proper way is to generate through
        // store_sales which calls our generate_row method.
        panic!("StoreReturnsRowGenerator::generate_row_and_child_rows should not be called directly. Use StoreSalesRowGenerator to generate both sales and returns.");
    }

    fn consume_remaining_seeds_for_row(&mut self) {
        self.abstract_generator.consume_remaining_seeds_for_row();
    }

    fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64) {
        self.abstract_generator
            .skip_rows_until_starting_row_number(starting_row_number);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_returns_row_generator_creation() {
        let _generator = StoreReturnsRowGenerator::new();
        // Just test that it creates successfully
    }
}
