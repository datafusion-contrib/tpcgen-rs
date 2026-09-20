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

//! Catalog sales row structure

use crate::generator::CatalogSalesGeneratorColumn;
use crate::row::table_row::DatField;
use crate::types::Pricing;
use std::fmt;

/// Row structure for catalog_sales table
#[derive(Debug, Clone)]
pub struct CatalogSalesRow {
    pub(crate) null_bit_map: i64,
    pub(crate) cs_sold_date_sk: i64,
    pub(crate) cs_sold_time_sk: i64,
    pub(crate) cs_ship_date_sk: i64,
    pub(crate) cs_bill_customer_sk: i64,
    pub(crate) cs_bill_cdemo_sk: i64,
    pub(crate) cs_bill_hdemo_sk: i64,
    pub(crate) cs_bill_addr_sk: i64,
    pub(crate) cs_ship_customer_sk: i64,
    pub(crate) cs_ship_cdemo_sk: i64,
    pub(crate) cs_ship_hdemo_sk: i64,
    pub(crate) cs_ship_addr_sk: i64,
    pub(crate) cs_call_center_sk: i64,
    pub(crate) cs_catalog_page_sk: i64,
    pub(crate) cs_ship_mode_sk: i64,
    pub(crate) cs_warehouse_sk: i64,
    pub(crate) cs_sold_item_sk: i64,
    pub(crate) cs_promo_sk: i64,
    pub(crate) cs_order_number: i64,
    pub(crate) cs_pricing: Pricing,
}

impl CatalogSalesRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        null_bit_map: i64,
        cs_sold_date_sk: i64,
        cs_sold_time_sk: i64,
        cs_ship_date_sk: i64,
        cs_bill_customer_sk: i64,
        cs_bill_cdemo_sk: i64,
        cs_bill_hdemo_sk: i64,
        cs_bill_addr_sk: i64,
        cs_ship_customer_sk: i64,
        cs_ship_cdemo_sk: i64,
        cs_ship_hdemo_sk: i64,
        cs_ship_addr_sk: i64,
        cs_call_center_sk: i64,
        cs_catalog_page_sk: i64,
        cs_ship_mode_sk: i64,
        cs_warehouse_sk: i64,
        cs_sold_item_sk: i64,
        cs_promo_sk: i64,
        cs_order_number: i64,
        cs_pricing: Pricing,
    ) -> Self {
        CatalogSalesRow {
            null_bit_map,
            cs_sold_date_sk,
            cs_sold_time_sk,
            cs_ship_date_sk,
            cs_bill_customer_sk,
            cs_bill_cdemo_sk,
            cs_bill_hdemo_sk,
            cs_bill_addr_sk,
            cs_ship_customer_sk,
            cs_ship_cdemo_sk,
            cs_ship_hdemo_sk,
            cs_ship_addr_sk,
            cs_call_center_sk,
            cs_catalog_page_sk,
            cs_ship_mode_sk,
            cs_warehouse_sk,
            cs_sold_item_sk,
            cs_promo_sk,
            cs_order_number,
            cs_pricing,
        }
    }

    // Getters for fields needed by CatalogReturnsRowGenerator
    pub fn get_cs_ship_date_sk(&self) -> i64 {
        self.cs_ship_date_sk
    }

    pub fn get_cs_sold_item_sk(&self) -> i64 {
        self.cs_sold_item_sk
    }

    pub fn get_cs_bill_customer_sk(&self) -> i64 {
        self.cs_bill_customer_sk
    }

    pub fn get_cs_bill_cdemo_sk(&self) -> i64 {
        self.cs_bill_cdemo_sk
    }

    pub fn get_cs_bill_hdemo_sk(&self) -> i64 {
        self.cs_bill_hdemo_sk
    }

    pub fn get_cs_bill_addr_sk(&self) -> i64 {
        self.cs_bill_addr_sk
    }

    pub fn get_cs_ship_customer_sk(&self) -> i64 {
        self.cs_ship_customer_sk
    }

    pub fn get_cs_ship_cdemo_sk(&self) -> i64 {
        self.cs_ship_cdemo_sk
    }

    pub fn get_cs_ship_addr_sk(&self) -> i64 {
        self.cs_ship_addr_sk
    }

    pub fn get_cs_call_center_sk(&self) -> i64 {
        self.cs_call_center_sk
    }

    pub fn get_cs_catalog_page_sk(&self) -> i64 {
        self.cs_catalog_page_sk
    }

    pub fn get_cs_order_number(&self) -> i64 {
        self.cs_order_number
    }

    pub fn get_cs_pricing(&self) -> &Pricing {
        &self.cs_pricing
    }

    fn is_null(&self, column: &CatalogSalesGeneratorColumn) -> bool {
        let column_number = column.get_global_column_number();
        let first_column = CatalogSalesGeneratorColumn::CsSoldDateSk.get_global_column_number();
        let bit_position = column_number - first_column;
        (self.null_bit_map & (1 << bit_position)) != 0
    }

    pub fn null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    pub fn get_cs_sold_date_sk(&self) -> i64 {
        self.cs_sold_date_sk
    }

    pub fn get_cs_sold_time_sk(&self) -> i64 {
        self.cs_sold_time_sk
    }

    pub fn get_cs_ship_hdemo_sk(&self) -> i64 {
        self.cs_ship_hdemo_sk
    }

    pub fn get_cs_ship_mode_sk(&self) -> i64 {
        self.cs_ship_mode_sk
    }

    pub fn get_cs_warehouse_sk(&self) -> i64 {
        self.cs_warehouse_sk
    }

    pub fn get_cs_promo_sk(&self) -> i64 {
        self.cs_promo_sk
    }

    /// DAT field for a surrogate key: NULL when the null bit is set or the
    /// key is negative.
    pub(crate) fn key_field(
        &self,
        value: i64,
        column: &CatalogSalesGeneratorColumn,
    ) -> DatField<i64> {
        DatField::new(value, self.is_null(column) || value < 0)
    }

    /// DAT field for a regular value: NULL when the null bit is set.
    pub(crate) fn field<T>(&self, value: T, column: &CatalogSalesGeneratorColumn) -> DatField<T> {
        DatField::new(value, self.is_null(column))
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces one
/// `|`-terminated field per column.
impl fmt::Display for CatalogSalesRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CatalogSalesGeneratorColumn::*;

        write!(
            f,
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|",
            self.key_field(self.cs_sold_date_sk, &CsSoldDateSk),
            self.key_field(self.cs_sold_time_sk, &CsSoldTimeSk),
            self.key_field(self.cs_ship_date_sk, &CsShipDateSk),
            self.key_field(self.cs_bill_customer_sk, &CsBillCustomerSk),
            self.key_field(self.cs_bill_cdemo_sk, &CsBillCdemoSk),
            self.key_field(self.cs_bill_hdemo_sk, &CsBillHdemoSk),
            self.key_field(self.cs_bill_addr_sk, &CsBillAddrSk),
            self.key_field(self.cs_ship_customer_sk, &CsShipCustomerSk),
            self.key_field(self.cs_ship_cdemo_sk, &CsShipCdemoSk),
            self.key_field(self.cs_ship_hdemo_sk, &CsShipHdemoSk),
            self.key_field(self.cs_ship_addr_sk, &CsShipAddrSk),
            self.key_field(self.cs_call_center_sk, &CsCallCenterSk),
            self.key_field(self.cs_catalog_page_sk, &CsCatalogPageSk),
            self.key_field(self.cs_ship_mode_sk, &CsShipModeSk),
            self.field(self.cs_warehouse_sk, &CsWarehouseSk),
            self.key_field(self.cs_sold_item_sk, &CsSoldItemSk),
            self.key_field(self.cs_promo_sk, &CsPromoSk),
            self.field(self.cs_order_number, &CsOrderNumber),
            self.field(self.cs_pricing.get_quantity(), &CsPricingQuantity),
            self.field(
                self.cs_pricing.get_wholesale_cost(),
                &CsPricingWholesaleCost
            ),
            self.field(self.cs_pricing.get_list_price(), &CsPricingListPrice),
            self.field(self.cs_pricing.get_sales_price(), &CsPricingSalesPrice),
            self.field(
                self.cs_pricing.get_ext_discount_amount(),
                &CsPricingExtDiscountAmount
            ),
            self.field(
                self.cs_pricing.get_ext_sales_price(),
                &CsPricingExtSalesPrice
            ),
            self.field(
                self.cs_pricing.get_ext_wholesale_cost(),
                &CsPricingExtWholesaleCost
            ),
            self.field(
                self.cs_pricing.get_ext_list_price(),
                &CsPricingExtListPrice
            ),
            self.field(self.cs_pricing.get_ext_tax(), &CsPricingExtTax),
            self.field(self.cs_pricing.get_coupon_amount(), &CsPricingCouponAmt),
            self.field(self.cs_pricing.get_ext_ship_cost(), &CsPricingExtShipCost),
            self.field(self.cs_pricing.get_net_paid(), &CsPricingNetPaid),
            self.field(
                self.cs_pricing.get_net_paid_including_tax(),
                &CsPricingNetPaidIncTax
            ),
            self.field(
                self.cs_pricing.get_net_paid_including_shipping(),
                &CsPricingNetPaidIncShip
            ),
            self.field(
                self.cs_pricing.get_net_paid_including_shipping_and_tax(),
                &CsPricingNetPaidIncShipTax
            ),
            self.field(self.cs_pricing.get_net_profit(), &CsPricingNetProfit),
        )
    }
}

use crate::generator::GeneratorColumn;
