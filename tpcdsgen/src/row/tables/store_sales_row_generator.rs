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

//! Store sales row generator

use crate::config::Session;
use crate::error::Result;
use crate::generator::StoreSalesGeneratorColumn;
use crate::join_key_utils::generate_join_key;
use crate::nulls::create_null_bit_map;
use crate::permutations::{get_permutation_entry, make_permutation};
use crate::random::RandomValueGenerator;
use crate::row::store_returns_row_generator::StoreReturnsRowGenerator;
use crate::row::store_sales_row::StoreSalesRow;
use crate::row::{AbstractRowGenerator, GeneratedRow, RowGenerator, RowGeneratorResult};
use crate::slowly_changing_dimension_utils::match_surrogate_key;
use crate::table::Table;
use crate::types::{generate_pricing_for_sales_table, get_store_sales_pricing_limits};

/// Percentage of sales that get returned
const SR_RETURN_PCT: i32 = 10;

/// Order information shared across line items in the same order
struct OrderInfo {
    ss_sold_store_sk: i64,
    ss_sold_time_sk: i64,
    ss_sold_date_sk: i64,
    ss_sold_customer_sk: i64,
    ss_sold_cdemo_sk: i64,
    ss_sold_hdemo_sk: i64,
    ss_sold_addr_sk: i64,
    ss_ticket_number: i64,
}

impl OrderInfo {
    #[allow(clippy::too_many_arguments)]
    fn new(
        ss_sold_store_sk: i64,
        ss_sold_time_sk: i64,
        ss_sold_date_sk: i64,
        ss_sold_customer_sk: i64,
        ss_sold_cdemo_sk: i64,
        ss_sold_hdemo_sk: i64,
        ss_sold_addr_sk: i64,
        ss_ticket_number: i64,
    ) -> Self {
        OrderInfo {
            ss_sold_store_sk,
            ss_sold_time_sk,
            ss_sold_date_sk,
            ss_sold_customer_sk,
            ss_sold_cdemo_sk,
            ss_sold_hdemo_sk,
            ss_sold_addr_sk,
            ss_ticket_number,
        }
    }

    fn default() -> Self {
        OrderInfo {
            ss_sold_store_sk: 0,
            ss_sold_time_sk: 0,
            ss_sold_date_sk: 0,
            ss_sold_customer_sk: 0,
            ss_sold_cdemo_sk: 0,
            ss_sold_hdemo_sk: 0,
            ss_sold_addr_sk: 0,
            ss_ticket_number: 0,
        }
    }
}

pub struct StoreSalesRowGenerator {
    abstract_generator: AbstractRowGenerator,
    item_permutation: Option<Vec<i32>>,
    remaining_line_items: i32,
    order_info: OrderInfo,
    item_index: i32,
    store_returns_generator: StoreReturnsRowGenerator,
}

impl StoreSalesRowGenerator {
    pub fn new() -> Self {
        StoreSalesRowGenerator {
            abstract_generator: AbstractRowGenerator::new(Table::StoreSales),
            item_permutation: None,
            remaining_line_items: 0,
            order_info: OrderInfo::default(),
            item_index: 0,
            store_returns_generator: StoreReturnsRowGenerator::new(),
        }
    }

    fn generate_order_info(&mut self, row_number: u64, session: &Session) -> Result<OrderInfo> {
        use StoreSalesGeneratorColumn::*;

        let row_number_i64 = i64::try_from(row_number).expect("row number fits in i64");

        let scaling = session.get_scaling();

        let stream = self
            .abstract_generator
            .get_random_number_stream(&SsSoldStoreSk);
        let ss_sold_store_sk = generate_join_key(
            &SsSoldStoreSk,
            stream,
            crate::config::Table::Store,
            1,
            scaling,
        )?;

        let stream = self
            .abstract_generator
            .get_random_number_stream(&SsSoldTimeSk);
        let ss_sold_time_sk = generate_join_key(
            &SsSoldTimeSk,
            stream,
            crate::config::Table::TimeDim,
            1,
            scaling,
        )?;

        let stream = self
            .abstract_generator
            .get_random_number_stream(&SsSoldDateSk);
        let ss_sold_date_sk = generate_join_key(
            &SsSoldDateSk,
            stream,
            crate::config::Table::DateDim,
            1,
            scaling,
        )?;

        let stream = self
            .abstract_generator
            .get_random_number_stream(&SsSoldCustomerSk);
        let ss_sold_customer_sk = generate_join_key(
            &SsSoldCustomerSk,
            stream,
            crate::config::Table::Customer,
            1,
            scaling,
        )?;

        let stream = self
            .abstract_generator
            .get_random_number_stream(&SsSoldCdemoSk);
        let ss_sold_cdemo_sk = generate_join_key(
            &SsSoldCdemoSk,
            stream,
            crate::config::Table::CustomerDemographics,
            1,
            scaling,
        )?;

        let stream = self
            .abstract_generator
            .get_random_number_stream(&SsSoldHdemoSk);
        let ss_sold_hdemo_sk = generate_join_key(
            &SsSoldHdemoSk,
            stream,
            crate::config::Table::HouseholdDemographics,
            1,
            scaling,
        )?;

        let stream = self
            .abstract_generator
            .get_random_number_stream(&SsSoldAddrSk);
        let ss_sold_addr_sk = generate_join_key(
            &SsSoldAddrSk,
            stream,
            crate::config::Table::CustomerAddress,
            1,
            scaling,
        )?;

        let ss_ticket_number = row_number_i64;

        Ok(OrderInfo::new(
            ss_sold_store_sk,
            ss_sold_time_sk,
            ss_sold_date_sk,
            ss_sold_customer_sk,
            ss_sold_cdemo_sk,
            ss_sold_hdemo_sk,
            ss_sold_addr_sk,
            ss_ticket_number,
        ))
    }

    fn is_last_row_in_order(&self) -> bool {
        self.remaining_line_items == 0
    }
}

impl Default for StoreSalesRowGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl RowGenerator for StoreSalesRowGenerator {
    fn generate_row_and_child_rows(
        &mut self,
        row_number: u64,
        session: &Session,
        _parent_row_generator: Option<&mut dyn RowGenerator>,
        _child_row_generator: Option<&mut dyn RowGenerator>,
    ) -> Result<RowGeneratorResult> {
        use StoreSalesGeneratorColumn::*;

        let scaling = session.get_scaling();
        let item_count = scaling.get_id_count(crate::config::Table::Item) as usize;

        // Initialize item permutation if needed
        if self.item_permutation.is_none() {
            let stream = self
                .abstract_generator
                .get_random_number_stream(&SsPermutation);
            self.item_permutation = Some(make_permutation(item_count, stream));
        }

        // Start a new order if we've finished the previous one
        if self.remaining_line_items == 0 {
            self.order_info = self.generate_order_info(row_number, session)?;

            let stream = self
                .abstract_generator
                .get_random_number_stream(&SsTicketNumber);
            self.remaining_line_items =
                RandomValueGenerator::generate_uniform_random_int(8, 16, stream);

            let stream = self
                .abstract_generator
                .get_random_number_stream(&SsSoldItemSk);
            self.item_index =
                RandomValueGenerator::generate_uniform_random_int(1, item_count as i32, stream);
        }

        // Generate null bit map
        let stream = self.abstract_generator.get_random_number_stream(&SsNulls);
        let null_bit_map = create_null_bit_map(Table::StoreSales, stream);

        // Items need to be unique within an order
        // Use a sequence within the permutation
        self.item_index += 1;
        if self.item_index > item_count as i32 {
            self.item_index = 1;
        }

        // Get item from permutation and match surrogate key for SCD
        let permutation = self.item_permutation.as_ref().unwrap();
        let item_key = get_permutation_entry(permutation, self.item_index);
        let ss_sold_item_sk = match_surrogate_key(
            item_key as i64,
            self.order_info.ss_sold_date_sk,
            crate::config::Table::Item,
            scaling,
        );

        // Generate promo sk
        let stream = self
            .abstract_generator
            .get_random_number_stream(&SsSoldPromoSk);
        let ss_sold_promo_sk = generate_join_key(
            &SsSoldPromoSk,
            stream,
            crate::config::Table::Promotion,
            1,
            scaling,
        )?;

        // Generate pricing
        let stream = self.abstract_generator.get_random_number_stream(&SsPricing);
        let ss_pricing =
            generate_pricing_for_sales_table(&get_store_sales_pricing_limits(), stream);

        let store_sales_row = StoreSalesRow::new(
            null_bit_map,
            self.order_info.ss_sold_date_sk,
            self.order_info.ss_sold_time_sk,
            ss_sold_item_sk,
            self.order_info.ss_sold_customer_sk,
            self.order_info.ss_sold_cdemo_sk,
            self.order_info.ss_sold_hdemo_sk,
            self.order_info.ss_sold_addr_sk,
            self.order_info.ss_sold_store_sk,
            ss_sold_promo_sk,
            self.order_info.ss_ticket_number,
            ss_pricing,
        );

        // Check if this sale gets returned (10% return rate)
        // We check and generate the return BEFORE moving the sales row to avoid cloning
        let stream = self
            .abstract_generator
            .get_random_number_stream(&SrIsReturned);
        let random_int = RandomValueGenerator::generate_uniform_random_int(0, 99, stream);

        // Generate return row if applicable (using reference before we move sales_row)
        // Note: In Java's --table store_sales mode, returns are NOT generated.
        // This code generates returns (like Java's --table store_returns mode).
        // The consume_remaining_seeds_for_row() is called separately in the binary.
        let return_row = if random_int < SR_RETURN_PCT {
            Some(
                self.store_returns_generator
                    .generate_row(session, &store_sales_row)?,
            )
        } else {
            None
        };

        // Now move (not clone) the sales row into the result
        let mut generated_rows: Vec<GeneratedRow> = Vec::with_capacity(2);
        generated_rows.push(store_sales_row.into());

        if let Some(ret_row) = return_row {
            generated_rows.push(ret_row);
        }

        self.remaining_line_items -= 1;

        Ok(RowGeneratorResult::new_with_multiple(
            generated_rows,
            self.is_last_row_in_order(),
        ))
    }

    fn consume_remaining_seeds_for_row(&mut self) {
        self.abstract_generator.consume_remaining_seeds_for_row();
        self.store_returns_generator
            .consume_remaining_seeds_for_row();
    }

    fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64) {
        self.abstract_generator
            .skip_rows_until_starting_row_number(starting_row_number);
        self.store_returns_generator
            .skip_rows_until_starting_row_number(starting_row_number);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Session;
    use crate::row::dat_values;

    #[test]
    fn test_store_sales_row_generator_creation() {
        let generator = StoreSalesRowGenerator::new();
        assert!(generator.item_permutation.is_none());
        assert_eq!(generator.remaining_line_items, 0);
    }

    #[test]
    fn test_store_sales_row_generation() {
        let mut generator = StoreSalesRowGenerator::new();
        let session = Session::default();

        let result = generator
            .generate_row_and_child_rows(1, &session, None, None)
            .unwrap();

        // Should have at least one row (the store_sales row)
        assert!(!result.get_rows().is_empty());

        // First row should have 23 columns
        let first_row = &result.get_rows()[0];
        assert_eq!(dat_values(&first_row).len(), 23);
    }

    #[test]
    fn test_store_sales_order_grouping() {
        let mut generator = StoreSalesRowGenerator::new();
        let session = Session::default();

        // Generate first row (starts new order)
        let result1 = generator
            .generate_row_and_child_rows(1, &session, None, None)
            .unwrap();
        let values1 = dat_values(&result1.get_rows()[0]);
        let ticket1 = &values1[9]; // ss_ticket_number

        // Generate second row (should be in same order)
        let result2 = generator
            .generate_row_and_child_rows(2, &session, None, None)
            .unwrap();
        let values2 = dat_values(&result2.get_rows()[0]);
        let ticket2 = &values2[9];

        // Same ticket number means same order
        assert_eq!(ticket1, ticket2);
    }
}
