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

//! Web returns row generator

use crate::config::Session;
use crate::error::Result;
use crate::generator::WebReturnsGeneratorColumn;
use crate::join_key_utils::generate_join_key;
use crate::nulls::create_null_bit_map;
use crate::random::RandomValueGenerator;
use crate::row::web_returns_row::WebReturnsRow;
use crate::row::web_sales_row::WebSalesRow;
use crate::row::web_sales_row_generator::{WebSalesRowGenerator, GIFT_PERCENTAGE};
use crate::row::AbstractRowGenerator;
use crate::table::Table;
use crate::types::generate_pricing_for_returns_table;

/// Generates `web_returns` rows: the returned line items of `web_sales`.
///
/// Replays the `web_sales` line items through a [`WebSalesRowGenerator`],
/// generating the sales row only for the ~10% of line items that are
/// returned.
pub struct WebReturnsRowGenerator {
    sales: WebSalesRowGenerator,
    abstract_generator: AbstractRowGenerator,
}

impl WebReturnsRowGenerator {
    /// Generate the returns of `web_sales` source rows `1..=row_count`.
    pub fn new(session: Session, row_count: u64) -> Self {
        WebReturnsRowGenerator {
            sales: WebSalesRowGenerator::new(session, row_count),
            abstract_generator: AbstractRowGenerator::new(Table::WebReturns),
        }
    }

    /// Start generating at source row `starting_row_number` (1-based), fast
    /// forwarding the random number streams to that row.
    pub fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64) {
        self.sales
            .skip_rows_until_starting_row_number(starting_row_number);
        self.abstract_generator
            .skip_rows_until_starting_row_number(starting_row_number);
    }

    /// Restrict generation to source rows
    /// `starting_row_number..=ending_row_number` (1-based, inclusive).
    ///
    /// The ending row number is clamped to the table's row count.
    pub fn set_source_row_range(&mut self, starting_row_number: u64, ending_row_number: u64) {
        self.sales
            .set_source_row_range(starting_row_number, ending_row_number);
        self.abstract_generator
            .skip_rows_until_starting_row_number(starting_row_number);
    }

    /// Generate the return row for `sales_row`.
    fn generate_row(&mut self, sales_row: &WebSalesRow) -> Result<WebReturnsRow> {
        use WebReturnsGeneratorColumn::*;

        let scaling = self.sales.session().get_scaling();

        // Generate null bit map
        let stream = self.abstract_generator.get_random_number_stream(&WrNulls);
        let null_bit_map = create_null_bit_map(Table::WebReturns, stream);

        // Fields taken from the original sale
        let wr_item_sk = sales_row.get_ws_item_sk();
        let wr_order_number = sales_row.get_ws_order_number();
        let wr_web_page_sk = sales_row.get_ws_web_page_sk();

        // Remaining fields are specific to this return
        let stream = self
            .abstract_generator
            .get_random_number_stream(&WrReturnedDateSk);
        let wr_returned_date_sk = generate_join_key(
            &WrReturnedDateSk,
            stream,
            crate::config::Table::DateDim,
            sales_row.get_ws_ship_date_sk(),
            scaling,
        )?;

        let stream = self
            .abstract_generator
            .get_random_number_stream(&WrReturnedTimeSk);
        let wr_returned_time_sk = generate_join_key(
            &WrReturnedTimeSk,
            stream,
            crate::config::Table::TimeDim,
            1,
            scaling,
        )?;

        // Items are usually returned to the people they were shipped to, but sometimes not
        // Generate new values first
        let stream = self
            .abstract_generator
            .get_random_number_stream(&WrRefundedCustomerSk);
        let mut wr_refunded_customer_sk = generate_join_key(
            &WrRefundedCustomerSk,
            stream,
            crate::config::Table::Customer,
            1,
            scaling,
        )?;

        let stream = self
            .abstract_generator
            .get_random_number_stream(&WrRefundedCdemoSk);
        let mut wr_refunded_cdemo_sk = generate_join_key(
            &WrRefundedCdemoSk,
            stream,
            crate::config::Table::CustomerDemographics,
            1,
            scaling,
        )?;

        let stream = self
            .abstract_generator
            .get_random_number_stream(&WrRefundedHdemoSk);
        let mut wr_refunded_hdemo_sk = generate_join_key(
            &WrRefundedHdemoSk,
            stream,
            crate::config::Table::HouseholdDemographics,
            1,
            scaling,
        )?;

        let stream = self
            .abstract_generator
            .get_random_number_stream(&WrRefundedAddrSk);
        let mut wr_refunded_addr_sk = generate_join_key(
            &WrRefundedAddrSk,
            stream,
            crate::config::Table::CustomerAddress,
            1,
            scaling,
        )?;

        // If below GIFT_PERCENTAGE, use ship info from sales row instead
        let stream = self
            .abstract_generator
            .get_random_number_stream(&WrReturningCustomerSk);
        let random_int = RandomValueGenerator::generate_uniform_random_int(0, 99, stream);
        if random_int < GIFT_PERCENTAGE {
            wr_refunded_customer_sk = sales_row.get_ws_ship_customer_sk();
            wr_refunded_cdemo_sk = sales_row.get_ws_ship_cdemo_sk();
            wr_refunded_hdemo_sk = sales_row.get_ws_ship_hdemo_sk();
            wr_refunded_addr_sk = sales_row.get_ws_ship_addr_sk();
        }

        // Returning customer is same as refunded customer
        let wr_returning_customer_sk = wr_refunded_customer_sk;
        let wr_returning_cdemo_sk = wr_refunded_cdemo_sk;
        let wr_returning_hdemo_sk = wr_refunded_hdemo_sk;
        let wr_returning_addr_sk = wr_refunded_addr_sk;

        let stream = self
            .abstract_generator
            .get_random_number_stream(&WrReasonSk);
        let wr_reason_sk = generate_join_key(
            &WrReasonSk,
            stream,
            crate::config::Table::Reason,
            1,
            scaling,
        )?;

        // Generate pricing for returns
        let stream = self.abstract_generator.get_random_number_stream(&WrPricing);
        let quantity = RandomValueGenerator::generate_uniform_random_int(
            1,
            sales_row.get_ws_pricing().get_quantity(),
            stream,
        );

        let stream = self.abstract_generator.get_random_number_stream(&WrPricing);
        let wr_pricing =
            generate_pricing_for_returns_table(stream, quantity, sales_row.get_ws_pricing());

        Ok(WebReturnsRow::new(
            null_bit_map,
            wr_returned_date_sk,
            wr_returned_time_sk,
            wr_item_sk,
            wr_refunded_customer_sk,
            wr_refunded_cdemo_sk,
            wr_refunded_hdemo_sk,
            wr_refunded_addr_sk,
            wr_returning_customer_sk,
            wr_returning_cdemo_sk,
            wr_returning_hdemo_sk,
            wr_returning_addr_sk,
            wr_web_page_sk,
            wr_reason_sk,
            wr_order_number,
            wr_pricing,
        ))
    }
}

impl Iterator for WebReturnsRowGenerator {
    type Item = WebReturnsRow;

    fn next(&mut self) -> Option<WebReturnsRow> {
        loop {
            let item = self.sales.next_line_item().expect("row gen")?;
            let row = if item.is_returned {
                let sales_row = self
                    .sales
                    .generate_sales_row(item.item_sk)
                    .expect("row gen");
                Some(self.generate_row(&sales_row).expect("row gen"))
            } else {
                self.sales.skip_item_sales_draws();
                None
            };
            if self.sales.finish_line_item() {
                self.abstract_generator.consume_remaining_seeds_for_row();
            }
            if row.is_some() {
                return row;
            }
        }
    }
}
