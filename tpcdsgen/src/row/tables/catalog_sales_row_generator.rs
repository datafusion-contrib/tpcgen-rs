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

//! Catalog sales row generator

use crate::config::Session;
use crate::error::Result;
use crate::generator::CatalogSalesGeneratorColumn;
use crate::join_key_utils::generate_join_key;
use crate::nulls::create_null_bit_map;
use crate::permutations::{get_permutation_entry, make_permutation};
use crate::random::RandomValueGenerator;
use crate::row::catalog_returns_row_generator::{CatalogReturnsRowGenerator, RETURN_PERCENT};
use crate::row::catalog_sales_row::CatalogSalesRow;
use crate::row::{AbstractRowGenerator, GeneratedRow, RowGenerator, RowGeneratorResult};
use crate::slowly_changing_dimension_utils::match_surrogate_key;
use crate::table::Table;
use crate::types::{generate_pricing_for_sales_table, get_catalog_sales_pricing_limits, Date};

/// Minimum days from order to ship
pub const CS_MIN_SHIP_DELAY: i32 = 2;
/// Maximum days from order to ship
pub const CS_MAX_SHIP_DELAY: i32 = 90;
/// Percentage of orders that are gifts (ship to different customer)
const GIFT_PERCENTAGE: i32 = 10;

/// Order information shared across line items in the same order
struct OrderInfo {
    cs_sold_date_sk: i64,
    cs_sold_time_sk: i64,
    cs_call_center_sk: i64,
    cs_bill_customer_sk: i64,
    cs_bill_cdemo_sk: i64,
    cs_bill_hdemo_sk: i64,
    cs_bill_addr_sk: i64,
    cs_ship_customer_sk: i64,
    cs_ship_cdemo_sk: i64,
    cs_ship_hdemo_sk: i64,
    cs_ship_addr_sk: i64,
    cs_order_number: i64,
}

impl OrderInfo {
    fn default() -> Self {
        OrderInfo {
            cs_sold_date_sk: 0,
            cs_sold_time_sk: 0,
            cs_call_center_sk: 0,
            cs_bill_customer_sk: 0,
            cs_bill_cdemo_sk: 0,
            cs_bill_hdemo_sk: 0,
            cs_bill_addr_sk: 0,
            cs_ship_customer_sk: 0,
            cs_ship_cdemo_sk: 0,
            cs_ship_hdemo_sk: 0,
            cs_ship_addr_sk: 0,
            cs_order_number: 0,
        }
    }
}

pub struct CatalogSalesRowGenerator {
    abstract_generator: AbstractRowGenerator,
    item_permutation: Option<Vec<i32>>,
    julian_date: i64,
    next_date_index: u64,
    remaining_line_items: i32,
    order_info: OrderInfo,
    ticket_item_base: i32,
    catalog_returns_generator: CatalogReturnsRowGenerator,
}

impl CatalogSalesRowGenerator {
    pub fn new() -> Self {
        CatalogSalesRowGenerator {
            abstract_generator: AbstractRowGenerator::new(Table::CatalogSales),
            item_permutation: None,
            julian_date: Date::JULIAN_DATA_START_DATE,
            next_date_index: 0,
            remaining_line_items: 0,
            order_info: OrderInfo::default(),
            ticket_item_base: 0,
            catalog_returns_generator: CatalogReturnsRowGenerator::new(),
        }
    }

    fn generate_order_info(&mut self, row_number: u64, session: &Session) -> Result<OrderInfo> {
        use CatalogSalesGeneratorColumn::*;

        let row_number_i64 = i64::try_from(row_number).expect("row number fits in i64");

        let scaling = session.get_scaling();

        // Move to a new date if the row number is ahead of the nextDateIndex
        while row_number > self.next_date_index {
            self.julian_date += 1;
            self.next_date_index += scaling
                .get_row_count_for_date(crate::config::Table::CatalogSales, self.julian_date);
        }

        let cs_sold_date_sk = self.julian_date;

        // cs_sold_time_sk uses cs_call_center_sk from previous order (like Java)
        let stream = self
            .abstract_generator
            .get_random_number_stream(&CsSoldTimeSk);
        let cs_sold_time_sk = generate_join_key(
            &CsSoldTimeSk,
            stream,
            crate::config::Table::TimeDim,
            self.order_info.cs_call_center_sk,
            scaling,
        )?;

        let stream = self
            .abstract_generator
            .get_random_number_stream(&CsCallCenterSk);
        let cs_call_center_sk = if cs_sold_date_sk == -1 {
            -1
        } else {
            generate_join_key(
                &CsCallCenterSk,
                stream,
                crate::config::Table::CallCenter,
                cs_sold_date_sk,
                scaling,
            )?
        };

        let stream = self
            .abstract_generator
            .get_random_number_stream(&CsBillCustomerSk);
        let cs_bill_customer_sk = generate_join_key(
            &CsBillCustomerSk,
            stream,
            crate::config::Table::Customer,
            1,
            scaling,
        )?;

        let stream = self
            .abstract_generator
            .get_random_number_stream(&CsBillCdemoSk);
        let cs_bill_cdemo_sk = generate_join_key(
            &CsBillCdemoSk,
            stream,
            crate::config::Table::CustomerDemographics,
            1,
            scaling,
        )?;

        let stream = self
            .abstract_generator
            .get_random_number_stream(&CsBillHdemoSk);
        let cs_bill_hdemo_sk = generate_join_key(
            &CsBillHdemoSk,
            stream,
            crate::config::Table::HouseholdDemographics,
            1,
            scaling,
        )?;

        let stream = self
            .abstract_generator
            .get_random_number_stream(&CsBillAddrSk);
        let cs_bill_addr_sk = generate_join_key(
            &CsBillAddrSk,
            stream,
            crate::config::Table::CustomerAddress,
            1,
            scaling,
        )?;

        // Most orders are for the ordering customer, some are gifts (10%)
        let stream = self
            .abstract_generator
            .get_random_number_stream(&CsShipCustomerSk);
        let gift_percentage = RandomValueGenerator::generate_uniform_random_int(0, 99, stream);

        let (cs_ship_customer_sk, cs_ship_cdemo_sk, cs_ship_hdemo_sk, cs_ship_addr_sk) =
            if gift_percentage <= GIFT_PERCENTAGE {
                // Gift order - ship to different customer
                let stream = self
                    .abstract_generator
                    .get_random_number_stream(&CsShipCustomerSk);
                let ship_customer = generate_join_key(
                    &CsShipCustomerSk,
                    stream,
                    crate::config::Table::Customer,
                    2,
                    scaling,
                )?;

                let stream = self
                    .abstract_generator
                    .get_random_number_stream(&CsShipCdemoSk);
                let ship_cdemo = generate_join_key(
                    &CsShipCdemoSk,
                    stream,
                    crate::config::Table::CustomerDemographics,
                    2,
                    scaling,
                )?;

                let stream = self
                    .abstract_generator
                    .get_random_number_stream(&CsShipHdemoSk);
                let ship_hdemo = generate_join_key(
                    &CsShipHdemoSk,
                    stream,
                    crate::config::Table::HouseholdDemographics,
                    2,
                    scaling,
                )?;

                let stream = self
                    .abstract_generator
                    .get_random_number_stream(&CsShipAddrSk);
                let ship_addr = generate_join_key(
                    &CsShipAddrSk,
                    stream,
                    crate::config::Table::CustomerAddress,
                    2,
                    scaling,
                )?;

                (ship_customer, ship_cdemo, ship_hdemo, ship_addr)
            } else {
                // Same as bill customer
                (
                    cs_bill_customer_sk,
                    cs_bill_cdemo_sk,
                    cs_bill_hdemo_sk,
                    cs_bill_addr_sk,
                )
            };

        let cs_order_number = row_number_i64;

        Ok(OrderInfo {
            cs_sold_date_sk,
            cs_sold_time_sk,
            cs_call_center_sk,
            cs_bill_customer_sk,
            cs_bill_cdemo_sk,
            cs_bill_hdemo_sk,
            cs_bill_addr_sk,
            cs_ship_customer_sk,
            cs_ship_cdemo_sk,
            cs_ship_hdemo_sk,
            cs_ship_addr_sk,
            cs_order_number,
        })
    }

    fn is_last_row_in_order(&self) -> bool {
        self.remaining_line_items == 0
    }
}

impl Default for CatalogSalesRowGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl RowGenerator for CatalogSalesRowGenerator {
    fn generate_row_and_child_rows(
        &mut self,
        row_number: u64,
        session: &Session,
        _parent_row_generator: Option<&mut dyn RowGenerator>,
        _child_row_generator: Option<&mut dyn RowGenerator>,
    ) -> Result<RowGeneratorResult> {
        use CatalogSalesGeneratorColumn::*;

        let scaling = session.get_scaling();
        let item_count = scaling.get_id_count(crate::config::Table::Item) as usize;

        // Initialize item permutation and date tracking if needed
        if self.item_permutation.is_none() {
            let stream = self.abstract_generator.get_random_number_stream(&CsPermute);
            self.item_permutation = Some(make_permutation(item_count, stream));

            // Initialize date tracking
            self.julian_date = Date::JULIAN_DATA_START_DATE;
            self.next_date_index = scaling
                .get_row_count_for_date(crate::config::Table::CatalogSales, self.julian_date)
                + 1;
        }

        // Start a new order if we've finished the previous one
        if self.remaining_line_items == 0 {
            self.order_info = self.generate_order_info(row_number, session)?;

            let stream = self
                .abstract_generator
                .get_random_number_stream(&CsSoldItemSk);
            self.ticket_item_base =
                RandomValueGenerator::generate_uniform_random_int(1, item_count as i32, stream);

            let stream = self
                .abstract_generator
                .get_random_number_stream(&CsOrderNumber);
            self.remaining_line_items =
                RandomValueGenerator::generate_uniform_random_int(4, 14, stream);
        }

        // Generate null bit map
        let stream = self.abstract_generator.get_random_number_stream(&CsNulls);
        let null_bit_map = create_null_bit_map(Table::CatalogSales, stream);

        // Orders are shipped some number of days after they are ordered
        let stream = self
            .abstract_generator
            .get_random_number_stream(&CsShipDateSk);
        let shipping_lag = RandomValueGenerator::generate_uniform_random_int(
            CS_MIN_SHIP_DELAY,
            CS_MAX_SHIP_DELAY,
            stream,
        );
        let cs_ship_date_sk = if self.order_info.cs_sold_date_sk == -1 {
            -1
        } else {
            self.order_info.cs_sold_date_sk + shipping_lag as i64
        };

        // Items need to be unique within an order
        // Use a sequence within the permutation
        self.ticket_item_base += 1;
        if self.ticket_item_base > item_count as i32 {
            self.ticket_item_base = 1;
        }

        // Get item from permutation and match surrogate key for SCD
        let permutation = self.item_permutation.as_ref().unwrap();
        let item_key = get_permutation_entry(permutation, self.ticket_item_base);
        let cs_sold_item_sk = match_surrogate_key(
            item_key as i64,
            self.order_info.cs_sold_date_sk,
            crate::config::Table::Item,
            scaling,
        );

        // Catalog page needs to be from a catalog active at the time of the sale
        let stream = self
            .abstract_generator
            .get_random_number_stream(&CsCatalogPageSk);
        let cs_catalog_page_sk = if self.order_info.cs_sold_date_sk == -1 {
            -1
        } else {
            generate_join_key(
                &CsCatalogPageSk,
                stream,
                crate::config::Table::CatalogPage,
                self.order_info.cs_sold_date_sk,
                scaling,
            )?
        };

        // Generate ship mode
        let stream = self
            .abstract_generator
            .get_random_number_stream(&CsShipModeSk);
        let cs_ship_mode_sk = generate_join_key(
            &CsShipModeSk,
            stream,
            crate::config::Table::ShipMode,
            1,
            scaling,
        )?;

        // Generate warehouse
        let stream = self
            .abstract_generator
            .get_random_number_stream(&CsWarehouseSk);
        let cs_warehouse_sk = generate_join_key(
            &CsWarehouseSk,
            stream,
            crate::config::Table::Warehouse,
            1,
            scaling,
        )?;

        // Generate promo sk
        let stream = self.abstract_generator.get_random_number_stream(&CsPromoSk);
        let cs_promo_sk = generate_join_key(
            &CsPromoSk,
            stream,
            crate::config::Table::Promotion,
            1,
            scaling,
        )?;

        // Generate pricing
        let stream = self.abstract_generator.get_random_number_stream(&CsPricing);
        let cs_pricing =
            generate_pricing_for_sales_table(&get_catalog_sales_pricing_limits(), stream);

        let catalog_sales_row = CatalogSalesRow::new(
            null_bit_map,
            self.order_info.cs_sold_date_sk,
            self.order_info.cs_sold_time_sk,
            cs_ship_date_sk,
            self.order_info.cs_bill_customer_sk,
            self.order_info.cs_bill_cdemo_sk,
            self.order_info.cs_bill_hdemo_sk,
            self.order_info.cs_bill_addr_sk,
            self.order_info.cs_ship_customer_sk,
            self.order_info.cs_ship_cdemo_sk,
            self.order_info.cs_ship_hdemo_sk,
            self.order_info.cs_ship_addr_sk,
            self.order_info.cs_call_center_sk,
            cs_catalog_page_sk,
            cs_ship_mode_sk,
            cs_warehouse_sk,
            cs_sold_item_sk,
            cs_promo_sk,
            self.order_info.cs_order_number,
            cs_pricing,
        );

        // Check if this sale gets returned (10% return rate)
        // We check and generate the return BEFORE moving the sales row to avoid cloning
        let stream = self
            .abstract_generator
            .get_random_number_stream(&CrIsReturned);
        let random_int = RandomValueGenerator::generate_uniform_random_int(0, 99, stream);

        // Generate return row if applicable (using reference before we move sales_row)
        let return_row = if random_int < RETURN_PERCENT {
            Some(
                self.catalog_returns_generator
                    .generate_row(session, &catalog_sales_row)?,
            )
        } else {
            None
        };

        // Now move (not clone) the sales row into the result
        let mut generated_rows: Vec<GeneratedRow> = Vec::with_capacity(2);
        generated_rows.push(catalog_sales_row.into());

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
        self.catalog_returns_generator
            .consume_remaining_seeds_for_row();
    }

    fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64) {
        self.abstract_generator
            .skip_rows_until_starting_row_number(starting_row_number);
        self.catalog_returns_generator
            .skip_rows_until_starting_row_number(starting_row_number);
    }
}
