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

//! Web sales row generator

use crate::config::Session;
use crate::error::Result;
use crate::generator::WebSalesGeneratorColumn;
use crate::join_key_utils::generate_join_key;
use crate::nulls::create_null_bit_map;
use crate::permutations::{get_permutation_entry, make_permutation};
use crate::random::RandomValueGenerator;
use crate::row::web_returns_row_generator::WebReturnsRowGenerator;
use crate::row::web_sales_row::WebSalesRow;
use crate::row::{AbstractRowGenerator, GeneratedRow, RowGenerator, RowGeneratorResult};
use crate::slowly_changing_dimension_utils::match_surrogate_key;
use crate::table::Table;
use crate::types::{generate_pricing_for_sales_table, get_web_sales_pricing_limits};

/// Percentage for determining if order ships to different customer (gift)
/// Note: In Java, condition is `randomInt > GIFT_PERCENTAGE`, meaning ~92% are gifts
pub const GIFT_PERCENTAGE: i32 = 7;
/// Percentage of orders that get returned
pub const RETURN_PERCENTAGE: i32 = 10;

/// Order information shared across line items in the same order
struct OrderInfo {
    ws_sold_date_sk: i64,
    ws_sold_time_sk: i64,
    ws_bill_customer_sk: i64,
    ws_bill_cdemo_sk: i64,
    ws_bill_hdemo_sk: i64,
    ws_bill_addr_sk: i64,
    ws_ship_customer_sk: i64,
    ws_ship_cdemo_sk: i64,
    ws_ship_hdemo_sk: i64,
    ws_ship_addr_sk: i64,
    ws_order_number: i64,
}

impl OrderInfo {
    fn default() -> Self {
        OrderInfo {
            ws_sold_date_sk: 0,
            ws_sold_time_sk: 0,
            ws_bill_customer_sk: 0,
            ws_bill_cdemo_sk: 0,
            ws_bill_hdemo_sk: 0,
            ws_bill_addr_sk: 0,
            ws_ship_customer_sk: 0,
            ws_ship_cdemo_sk: 0,
            ws_ship_hdemo_sk: 0,
            ws_ship_addr_sk: 0,
            ws_order_number: 0,
        }
    }
}

pub struct WebSalesRowGenerator {
    abstract_generator: AbstractRowGenerator,
    item_permutation: Option<Vec<i32>>,
    remaining_line_items: i32,
    order_info: OrderInfo,
    item_index: i32,
    web_returns_generator: WebReturnsRowGenerator,
}

impl WebSalesRowGenerator {
    pub fn new() -> Self {
        WebSalesRowGenerator {
            abstract_generator: AbstractRowGenerator::new(Table::WebSales),
            item_permutation: None,
            remaining_line_items: 0,
            order_info: OrderInfo::default(),
            item_index: 0,
            web_returns_generator: WebReturnsRowGenerator::new(),
        }
    }

    fn generate_order_info(&mut self, row_number: u64, session: &Session) -> Result<OrderInfo> {
        use WebSalesGeneratorColumn::*;

        let row_number_i64 = i64::try_from(row_number).expect("row number fits in i64");

        let scaling = session.get_scaling();

        // Web sales uses generate_join_key for date (not date-based iteration like catalog_sales)
        let stream = self
            .abstract_generator
            .get_random_number_stream(&WsSoldDateSk);
        let ws_sold_date_sk = generate_join_key(
            &WsSoldDateSk,
            stream,
            crate::config::Table::DateDim,
            1,
            scaling,
        )?;

        let stream = self
            .abstract_generator
            .get_random_number_stream(&WsSoldTimeSk);
        let ws_sold_time_sk = generate_join_key(
            &WsSoldTimeSk,
            stream,
            crate::config::Table::TimeDim,
            1,
            scaling,
        )?;

        let stream = self
            .abstract_generator
            .get_random_number_stream(&WsBillCustomerSk);
        let ws_bill_customer_sk = generate_join_key(
            &WsBillCustomerSk,
            stream,
            crate::config::Table::Customer,
            1,
            scaling,
        )?;

        let stream = self
            .abstract_generator
            .get_random_number_stream(&WsBillCdemoSk);
        let ws_bill_cdemo_sk = generate_join_key(
            &WsBillCdemoSk,
            stream,
            crate::config::Table::CustomerDemographics,
            1,
            scaling,
        )?;

        let stream = self
            .abstract_generator
            .get_random_number_stream(&WsBillHdemoSk);
        let ws_bill_hdemo_sk = generate_join_key(
            &WsBillHdemoSk,
            stream,
            crate::config::Table::HouseholdDemographics,
            1,
            scaling,
        )?;

        let stream = self
            .abstract_generator
            .get_random_number_stream(&WsBillAddrSk);
        let ws_bill_addr_sk = generate_join_key(
            &WsBillAddrSk,
            stream,
            crate::config::Table::CustomerAddress,
            1,
            scaling,
        )?;

        // Usually the billing info and shipping info are the same.
        // If it's a "gift", they'll be different.
        // Note: Java uses `randomInt > GIFT_PERCENTAGE` which means ~92% get different ship info
        let mut ws_ship_customer_sk = ws_bill_customer_sk;
        let mut ws_ship_cdemo_sk = ws_bill_cdemo_sk;
        let mut ws_ship_hdemo_sk = ws_bill_hdemo_sk;
        let mut ws_ship_addr_sk = ws_bill_addr_sk;

        let stream = self
            .abstract_generator
            .get_random_number_stream(&WsShipCustomerSk);
        let random_int = RandomValueGenerator::generate_uniform_random_int(0, 99, stream);

        // Java: if (randomInt > GIFT_PERCENTAGE)
        if random_int > GIFT_PERCENTAGE {
            let stream = self
                .abstract_generator
                .get_random_number_stream(&WsShipCustomerSk);
            ws_ship_customer_sk = generate_join_key(
                &WsShipCustomerSk,
                stream,
                crate::config::Table::Customer,
                2,
                scaling,
            )?;

            let stream = self
                .abstract_generator
                .get_random_number_stream(&WsShipCdemoSk);
            ws_ship_cdemo_sk = generate_join_key(
                &WsShipCdemoSk,
                stream,
                crate::config::Table::CustomerDemographics,
                2,
                scaling,
            )?;

            let stream = self
                .abstract_generator
                .get_random_number_stream(&WsShipHdemoSk);
            ws_ship_hdemo_sk = generate_join_key(
                &WsShipHdemoSk,
                stream,
                crate::config::Table::HouseholdDemographics,
                2,
                scaling,
            )?;

            let stream = self
                .abstract_generator
                .get_random_number_stream(&WsShipAddrSk);
            ws_ship_addr_sk = generate_join_key(
                &WsShipAddrSk,
                stream,
                crate::config::Table::CustomerAddress,
                2,
                scaling,
            )?;
        }

        let ws_order_number = row_number_i64;

        Ok(OrderInfo {
            ws_sold_date_sk,
            ws_sold_time_sk,
            ws_bill_customer_sk,
            ws_bill_cdemo_sk,
            ws_bill_hdemo_sk,
            ws_bill_addr_sk,
            ws_ship_customer_sk,
            ws_ship_cdemo_sk,
            ws_ship_hdemo_sk,
            ws_ship_addr_sk,
            ws_order_number,
        })
    }

    fn is_last_row_in_order(&self) -> bool {
        self.remaining_line_items == 0
    }
}

impl Default for WebSalesRowGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl RowGenerator for WebSalesRowGenerator {
    fn generate_row_and_child_rows(
        &mut self,
        row_number: u64,
        session: &Session,
        _parent_row_generator: Option<&mut dyn RowGenerator>,
        _child_row_generator: Option<&mut dyn RowGenerator>,
    ) -> Result<RowGeneratorResult> {
        use WebSalesGeneratorColumn::*;

        let scaling = session.get_scaling();
        let item_count = scaling.get_id_count(crate::config::Table::Item) as usize;

        // Initialize item permutation if needed
        if self.item_permutation.is_none() {
            let stream = self
                .abstract_generator
                .get_random_number_stream(&WsPermutation);
            self.item_permutation = Some(make_permutation(item_count, stream));
        }

        // Start a new order if we've finished the previous one
        if self.remaining_line_items == 0 {
            self.order_info = self.generate_order_info(row_number, session)?;

            let stream = self.abstract_generator.get_random_number_stream(&WsItemSk);
            self.item_index =
                RandomValueGenerator::generate_uniform_random_int(1, item_count as i32, stream);

            // WebSales has 8-16 line items per order (vs 4-14 for CatalogSales)
            let stream = self
                .abstract_generator
                .get_random_number_stream(&WsOrderNumber);
            self.remaining_line_items =
                RandomValueGenerator::generate_uniform_random_int(8, 16, stream);
        }

        // Generate null bit map
        let stream = self.abstract_generator.get_random_number_stream(&WsNulls);
        let null_bit_map = create_null_bit_map(Table::WebSales, stream);

        // Orders are shipped some number of days after they are ordered (1-120 days)
        let stream = self
            .abstract_generator
            .get_random_number_stream(&WsShipDateSk);
        let ship_lag = RandomValueGenerator::generate_uniform_random_int(1, 120, stream);
        let ws_ship_date_sk = self.order_info.ws_sold_date_sk + ship_lag as i64;

        // Items need to be unique within an order
        self.item_index += 1;
        if self.item_index > item_count as i32 {
            self.item_index = 1;
        }

        // Get item from permutation and match surrogate key for SCD
        let permutation = self.item_permutation.as_ref().unwrap();
        let item_key = get_permutation_entry(permutation, self.item_index);
        let ws_item_sk = match_surrogate_key(
            item_key as i64,
            self.order_info.ws_sold_date_sk,
            crate::config::Table::Item,
            scaling,
        );

        // The web page needs to be valid for the sale date
        let stream = self
            .abstract_generator
            .get_random_number_stream(&WsWebPageSk);
        let ws_web_page_sk = generate_join_key(
            &WsWebPageSk,
            stream,
            crate::config::Table::WebPage,
            self.order_info.ws_sold_date_sk,
            scaling,
        )?;

        let stream = self
            .abstract_generator
            .get_random_number_stream(&WsWebSiteSk);
        let ws_web_site_sk = generate_join_key(
            &WsWebSiteSk,
            stream,
            crate::config::Table::WebSite,
            self.order_info.ws_sold_date_sk,
            scaling,
        )?;

        // Generate ship mode
        let stream = self
            .abstract_generator
            .get_random_number_stream(&WsShipModeSk);
        let ws_ship_mode_sk = generate_join_key(
            &WsShipModeSk,
            stream,
            crate::config::Table::ShipMode,
            1,
            scaling,
        )?;

        // Generate warehouse
        let stream = self
            .abstract_generator
            .get_random_number_stream(&WsWarehouseSk);
        let ws_warehouse_sk = generate_join_key(
            &WsWarehouseSk,
            stream,
            crate::config::Table::Warehouse,
            1,
            scaling,
        )?;

        // Generate promo sk
        let stream = self.abstract_generator.get_random_number_stream(&WsPromoSk);
        let ws_promo_sk = generate_join_key(
            &WsPromoSk,
            stream,
            crate::config::Table::Promotion,
            1,
            scaling,
        )?;

        // Generate pricing
        let stream = self.abstract_generator.get_random_number_stream(&WsPricing);
        let ws_pricing = generate_pricing_for_sales_table(&get_web_sales_pricing_limits(), stream);

        let web_sales_row = WebSalesRow::new(
            null_bit_map,
            self.order_info.ws_sold_date_sk,
            self.order_info.ws_sold_time_sk,
            ws_ship_date_sk,
            ws_item_sk,
            self.order_info.ws_bill_customer_sk,
            self.order_info.ws_bill_cdemo_sk,
            self.order_info.ws_bill_hdemo_sk,
            self.order_info.ws_bill_addr_sk,
            self.order_info.ws_ship_customer_sk,
            self.order_info.ws_ship_cdemo_sk,
            self.order_info.ws_ship_hdemo_sk,
            self.order_info.ws_ship_addr_sk,
            ws_web_page_sk,
            ws_web_site_sk,
            ws_ship_mode_sk,
            ws_warehouse_sk,
            ws_promo_sk,
            self.order_info.ws_order_number,
            ws_pricing,
        );

        // Check if this sale gets returned (10% return rate)
        // We check and generate the return BEFORE moving the sales row to avoid cloning
        let stream = self
            .abstract_generator
            .get_random_number_stream(&WrIsReturned);
        let random_int = RandomValueGenerator::generate_uniform_random_int(0, 99, stream);

        // Generate return row if applicable (using reference before we move sales_row)
        let return_row = if random_int < RETURN_PERCENTAGE {
            Some(
                self.web_returns_generator
                    .generate_row(session, &web_sales_row)?,
            )
        } else {
            None
        };

        // Now move (not clone) the sales row into the result
        let mut generated_rows: Vec<GeneratedRow> = Vec::with_capacity(2);
        generated_rows.push(web_sales_row.into());

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
        self.web_returns_generator.consume_remaining_seeds_for_row();
    }

    fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64) {
        self.abstract_generator
            .skip_rows_until_starting_row_number(starting_row_number);
        self.web_returns_generator
            .skip_rows_until_starting_row_number(starting_row_number);
    }
}
