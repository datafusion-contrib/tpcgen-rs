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

//! Web sales row definition

use crate::generator::{GeneratorColumn, WebSalesGeneratorColumn};
use crate::row::table_row::DatField;
use crate::types::Pricing;
use std::fmt;

/// Row structure for web_sales table
#[derive(Debug, Clone)]
pub struct WebSalesRow {
    null_bit_map: i64,
    pub(crate) ws_sold_date_sk: i64,
    pub(crate) ws_sold_time_sk: i64,
    pub(crate) ws_ship_date_sk: i64,
    pub(crate) ws_item_sk: i64,
    pub(crate) ws_bill_customer_sk: i64,
    pub(crate) ws_bill_cdemo_sk: i64,
    pub(crate) ws_bill_hdemo_sk: i64,
    pub(crate) ws_bill_addr_sk: i64,
    pub(crate) ws_ship_customer_sk: i64,
    pub(crate) ws_ship_cdemo_sk: i64,
    pub(crate) ws_ship_hdemo_sk: i64,
    pub(crate) ws_ship_addr_sk: i64,
    pub(crate) ws_web_page_sk: i64,
    pub(crate) ws_web_site_sk: i64,
    pub(crate) ws_ship_mode_sk: i64,
    pub(crate) ws_warehouse_sk: i64,
    pub(crate) ws_promo_sk: i64,
    pub(crate) ws_order_number: i64,
    pub(crate) ws_pricing: Pricing,
}

impl WebSalesRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        null_bit_map: i64,
        ws_sold_date_sk: i64,
        ws_sold_time_sk: i64,
        ws_ship_date_sk: i64,
        ws_item_sk: i64,
        ws_bill_customer_sk: i64,
        ws_bill_cdemo_sk: i64,
        ws_bill_hdemo_sk: i64,
        ws_bill_addr_sk: i64,
        ws_ship_customer_sk: i64,
        ws_ship_cdemo_sk: i64,
        ws_ship_hdemo_sk: i64,
        ws_ship_addr_sk: i64,
        ws_web_page_sk: i64,
        ws_web_site_sk: i64,
        ws_ship_mode_sk: i64,
        ws_warehouse_sk: i64,
        ws_promo_sk: i64,
        ws_order_number: i64,
        ws_pricing: Pricing,
    ) -> Self {
        WebSalesRow {
            null_bit_map,
            ws_sold_date_sk,
            ws_sold_time_sk,
            ws_ship_date_sk,
            ws_item_sk,
            ws_bill_customer_sk,
            ws_bill_cdemo_sk,
            ws_bill_hdemo_sk,
            ws_bill_addr_sk,
            ws_ship_customer_sk,
            ws_ship_cdemo_sk,
            ws_ship_hdemo_sk,
            ws_ship_addr_sk,
            ws_web_page_sk,
            ws_web_site_sk,
            ws_ship_mode_sk,
            ws_warehouse_sk,
            ws_promo_sk,
            ws_order_number,
            ws_pricing,
        }
    }

    pub fn get_ws_item_sk(&self) -> i64 {
        self.ws_item_sk
    }

    pub fn get_ws_order_number(&self) -> i64 {
        self.ws_order_number
    }

    pub fn get_ws_web_page_sk(&self) -> i64 {
        self.ws_web_page_sk
    }

    pub fn get_ws_ship_date_sk(&self) -> i64 {
        self.ws_ship_date_sk
    }

    pub fn get_ws_ship_customer_sk(&self) -> i64 {
        self.ws_ship_customer_sk
    }

    pub fn get_ws_ship_cdemo_sk(&self) -> i64 {
        self.ws_ship_cdemo_sk
    }

    pub fn get_ws_ship_hdemo_sk(&self) -> i64 {
        self.ws_ship_hdemo_sk
    }

    pub fn get_ws_ship_addr_sk(&self) -> i64 {
        self.ws_ship_addr_sk
    }

    pub fn get_ws_pricing(&self) -> &Pricing {
        &self.ws_pricing
    }

    fn is_null(&self, column: WebSalesGeneratorColumn) -> bool {
        let bit_position = column.get_global_column_number()
            - WebSalesGeneratorColumn::WsSoldDateSk.get_global_column_number();
        (self.null_bit_map & (1 << bit_position)) != 0
    }

    pub fn null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    pub fn get_ws_sold_date_sk(&self) -> i64 {
        self.ws_sold_date_sk
    }

    pub fn get_ws_sold_time_sk(&self) -> i64 {
        self.ws_sold_time_sk
    }

    pub fn get_ws_bill_customer_sk(&self) -> i64 {
        self.ws_bill_customer_sk
    }

    pub fn get_ws_bill_cdemo_sk(&self) -> i64 {
        self.ws_bill_cdemo_sk
    }

    pub fn get_ws_bill_hdemo_sk(&self) -> i64 {
        self.ws_bill_hdemo_sk
    }

    pub fn get_ws_bill_addr_sk(&self) -> i64 {
        self.ws_bill_addr_sk
    }

    pub fn get_ws_web_site_sk(&self) -> i64 {
        self.ws_web_site_sk
    }

    pub fn get_ws_ship_mode_sk(&self) -> i64 {
        self.ws_ship_mode_sk
    }

    pub fn get_ws_warehouse_sk(&self) -> i64 {
        self.ws_warehouse_sk
    }

    pub fn get_ws_promo_sk(&self) -> i64 {
        self.ws_promo_sk
    }
}

/// DAT field helper: NULL is driven purely by the null bit (web rows
/// apply no key sentinel check).
impl WebSalesRow {
    pub(crate) fn field<T>(&self, value: T, column: WebSalesGeneratorColumn) -> DatField<T> {
        DatField::new(value, self.is_null(column))
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces one
/// `|`-terminated field per column.
impl fmt::Display for WebSalesRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use WebSalesGeneratorColumn::*;

        write!(
            f,
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|",
            self.field(self.ws_sold_date_sk, WsSoldDateSk),
            self.field(self.ws_sold_time_sk, WsSoldTimeSk),
            self.field(self.ws_ship_date_sk, WsShipDateSk),
            self.field(self.ws_item_sk, WsItemSk),
            self.field(self.ws_bill_customer_sk, WsBillCustomerSk),
            self.field(self.ws_bill_cdemo_sk, WsBillCdemoSk),
            self.field(self.ws_bill_hdemo_sk, WsBillHdemoSk),
            self.field(self.ws_bill_addr_sk, WsBillAddrSk),
            self.field(self.ws_ship_customer_sk, WsShipCustomerSk),
            self.field(self.ws_ship_cdemo_sk, WsShipCdemoSk),
            self.field(self.ws_ship_hdemo_sk, WsShipHdemoSk),
            self.field(self.ws_ship_addr_sk, WsShipAddrSk),
            self.field(self.ws_web_page_sk, WsWebPageSk),
            self.field(self.ws_web_site_sk, WsWebSiteSk),
            self.field(self.ws_ship_mode_sk, WsShipModeSk),
            self.field(self.ws_warehouse_sk, WsWarehouseSk),
            self.field(self.ws_promo_sk, WsPromoSk),
            self.field(self.ws_order_number, WsOrderNumber),
            self.field(self.ws_pricing.get_quantity(), WsPricingQuantity),
            self.field(self.ws_pricing.get_wholesale_cost(), WsPricingWholesaleCost),
            self.field(self.ws_pricing.get_list_price(), WsPricingListPrice),
            self.field(self.ws_pricing.get_sales_price(), WsPricingSalesPrice),
            self.field(self.ws_pricing.get_ext_discount_amount(), WsPricingExtDiscountAmt),
            self.field(self.ws_pricing.get_ext_sales_price(), WsPricingExtSalesPrice),
            self.field(self.ws_pricing.get_ext_wholesale_cost(), WsPricingExtWholesaleCost),
            self.field(self.ws_pricing.get_ext_list_price(), WsPricingExtListPrice),
            self.field(self.ws_pricing.get_ext_tax(), WsPricingExtTax),
            self.field(self.ws_pricing.get_coupon_amount(), WsPricingCouponAmt),
            self.field(self.ws_pricing.get_ext_ship_cost(), WsPricingExtShipCost),
            self.field(self.ws_pricing.get_net_paid(), WsPricingNetPaid),
            self.field(self.ws_pricing.get_net_paid_including_tax(), WsPricingNetPaidIncTax),
            self.field(self.ws_pricing.get_net_paid_including_shipping(), WsPricingNetPaidIncShip),
            self.field(self.ws_pricing.get_net_paid_including_shipping_and_tax(), WsPricingNetPaidIncShipTax),
            self.field(self.ws_pricing.get_net_profit(), WsPricingNetProfit),
        )
    }
}
