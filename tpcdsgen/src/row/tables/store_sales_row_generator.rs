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
use crate::join_key_utils::{generate_join_key, skip_join_key};
use crate::nulls::{create_null_bit_map, skip_null_bit_map};
use crate::permutations::{get_permutation_entry, make_permutation};
use crate::random::RandomValueGenerator;
use crate::row::store_sales_row::StoreSalesRow;
use crate::row::{AbstractRowGenerator, LineItem};
use crate::slowly_changing_dimension_utils::match_surrogate_key;
use crate::table::Table;
use crate::types::{
    generate_pricing_for_sales_table, get_store_sales_pricing_limits, skip_pricing_for_sales_table,
};

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

/// Generates `store_sales` rows, several line items per source row (ticket).
///
/// [`StoreReturnsRowGenerator`] replays the same line items through the
/// `pub(crate)` stepping methods and keeps the returned ones.
///
/// [`StoreReturnsRowGenerator`]: crate::row::StoreReturnsRowGenerator
pub struct StoreSalesRowGenerator {
    abstract_generator: AbstractRowGenerator,
    item_permutation: Option<Vec<i32>>,
    remaining_line_items: i32,
    order_info: OrderInfo,
    item_index: i32,
    session: Session,
    current_row: u64,
    row_count: u64,
}

impl StoreSalesRowGenerator {
    /// Generate source rows `1..=row_count`.
    pub fn new(session: Session, row_count: u64) -> Self {
        StoreSalesRowGenerator {
            abstract_generator: AbstractRowGenerator::new(Table::StoreSales),
            item_permutation: None,
            remaining_line_items: 0,
            order_info: OrderInfo::default(),
            item_index: 0,
            session,
            current_row: 1,
            row_count,
        }
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

    pub(crate) fn session(&self) -> &Session {
        &self.session
    }

    /// Advance the random streams [`Self::generate_sales_row`] would have
    /// consumed, without calculating the row.
    ///
    /// Used by the returns generator for line items that are not returned.
    pub(crate) fn skip_item_sales_draws(&mut self) {
        use StoreSalesGeneratorColumn::*;

        let stream = self.abstract_generator.get_random_number_stream(&SsNulls);
        skip_null_bit_map(stream);

        let stream = self
            .abstract_generator
            .get_random_number_stream(&SsSoldPromoSk);
        skip_join_key(crate::config::Table::Promotion, stream);

        let stream = self.abstract_generator.get_random_number_stream(&SsPricing);
        skip_pricing_for_sales_table(stream);
    }

    /// Generate the sales row for the current line item.
    pub(crate) fn generate_sales_row(&mut self, ss_sold_item_sk: i64) -> Result<StoreSalesRow> {
        use StoreSalesGeneratorColumn::*;

        let scaling = self.session.get_scaling();

        let stream = self.abstract_generator.get_random_number_stream(&SsNulls);
        let null_bit_map = create_null_bit_map(Table::StoreSales, stream);

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

        let stream = self.abstract_generator.get_random_number_stream(&SsPricing);
        let ss_pricing =
            generate_pricing_for_sales_table(&get_store_sales_pricing_limits(), stream);

        Ok(StoreSalesRow::new(
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
        ))
    }

    fn generate_order_info(&mut self, row_number: u64) -> Result<OrderInfo> {
        use StoreSalesGeneratorColumn::*;

        let row_number_i64 = i64::try_from(row_number).expect("row number fits in i64");

        let scaling = self.session.get_scaling();

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

    /// Advance to the next line item, starting a new ticket when the
    /// previous one is complete.
    ///
    /// Returns `None` once every source row has been generated.
    pub(crate) fn next_line_item(&mut self) -> Result<Option<LineItem>> {
        use StoreSalesGeneratorColumn::*;

        if self.current_row > self.row_count {
            return Ok(None);
        }

        let item_count = self
            .session
            .get_scaling()
            .get_id_count(crate::config::Table::Item) as usize;

        // Initialize item permutation if needed
        if self.item_permutation.is_none() {
            let stream = self
                .abstract_generator
                .get_random_number_stream(&SsPermutation);
            self.item_permutation = Some(make_permutation(item_count, stream));
        }

        // Start a new order if we've finished the previous one
        if self.remaining_line_items == 0 {
            self.order_info = self.generate_order_info(self.current_row)?;

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

        // Items need to be unique within an order
        // Use a sequence within the permutation
        self.item_index += 1;
        if self.item_index > item_count as i32 {
            self.item_index = 1;
        }

        // Get item from permutation and match surrogate key for SCD
        let permutation = self.item_permutation.as_ref().unwrap();
        let item_key = get_permutation_entry(permutation, self.item_index);
        let item_sk = match_surrogate_key(
            item_key as i64,
            self.order_info.ss_sold_date_sk,
            crate::config::Table::Item,
            self.session.get_scaling(),
        );

        // Row is returned if random_int < SR_RETURN_PCT
        let stream = self
            .abstract_generator
            .get_random_number_stream(&SrIsReturned);
        let random_int = RandomValueGenerator::generate_uniform_random_int(0, 99, stream);

        Ok(Some(LineItem {
            item_sk,
            is_returned: random_int < SR_RETURN_PCT,
        }))
    }

    /// Finish the current line item, after its sales row was generated or
    /// skipped.
    ///
    /// Returns true when it was the last line item of its ticket: the
    /// ticket's remaining seeds have been consumed and generation moves to
    /// the next source row.
    pub(crate) fn finish_line_item(&mut self) -> bool {
        self.remaining_line_items -= 1;
        let last_in_order = self.remaining_line_items == 0;
        if last_in_order {
            self.abstract_generator.consume_remaining_seeds_for_row();
            self.current_row += 1;
        }
        last_in_order
    }
}

impl Iterator for StoreSalesRowGenerator {
    type Item = StoreSalesRow;

    fn next(&mut self) -> Option<StoreSalesRow> {
        let item = self.next_line_item().expect("row gen")?;
        let row = self.generate_sales_row(item.item_sk).expect("row gen");
        self.finish_line_item();
        Some(row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Session, SessionBuilder};
    use crate::row::dat_values;
    use crate::row::StoreReturnsRowGenerator;

    #[test]
    fn test_store_sales_row_generator_creation() {
        let generator = StoreSalesRowGenerator::new(Session::default(), 1);
        assert!(generator.item_permutation.is_none());
        assert_eq!(generator.remaining_line_items, 0);
    }

    #[test]
    fn test_store_sales_row_generation() {
        let mut generator = StoreSalesRowGenerator::new(Session::default(), 1);
        let row = generator.next().expect("first line item");
        assert_eq!(dat_values(&row).len(), 23);
    }

    #[test]
    fn test_store_sales_order_grouping() {
        let mut generator = StoreSalesRowGenerator::new(Session::default(), 1);
        // A ticket has at least 8 line items, so the first two rows share
        // its ticket number
        let ticket1 = dat_values(&generator.next().unwrap())[9].clone();
        let ticket2 = dat_values(&generator.next().unwrap())[9].clone();
        assert_eq!(ticket1, ticket2);
    }

    /// Splitting a table into source row ranges must produce exactly the same
    /// rows as generating it in one pass.
    macro_rules! assert_source_row_ranges_concatenate {
        ($generator:ty) => {{
            let session = SessionBuilder::new()
                .with_scale_factor(0.01)
                .build()
                .expect("session");
            let source_rows = session
                .get_scaling()
                .get_row_count(crate::config::Table::StoreSales);
            assert!(source_rows > 100, "need enough rows to split");
            let dat = |start: u64, end: u64| -> Vec<String> {
                let mut rows = <$generator>::new(session.clone(), source_rows);
                rows.set_source_row_range(start, end);
                rows.map(|row| row.to_string()).collect()
            };

            let whole = dat(1, source_rows);
            let mut chunked = dat(1, source_rows / 2);
            chunked.extend(dat(source_rows / 2 + 1, source_rows));

            assert!(!whole.is_empty());
            assert_eq!(whole, chunked);
        }};
    }

    #[test]
    fn store_sales_splits_into_source_row_ranges() {
        assert_source_row_ranges_concatenate!(StoreSalesRowGenerator);
    }

    #[test]
    fn store_returns_splits_into_source_row_ranges() {
        assert_source_row_ranges_concatenate!(StoreReturnsRowGenerator);
    }
}
