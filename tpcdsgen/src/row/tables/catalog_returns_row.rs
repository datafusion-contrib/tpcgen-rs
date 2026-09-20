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

//! Catalog returns row structure

use crate::generator::CatalogReturnsGeneratorColumn;
use crate::row::table_row::DatField;
use crate::types::Pricing;
use std::fmt;

/// Row structure for catalog_returns table
#[derive(Debug, Clone)]
pub struct CatalogReturnsRow {
    pub(crate) null_bit_map: i64,
    pub(crate) cr_returned_date_sk: i64,
    pub(crate) cr_returned_time_sk: i64,
    pub(crate) cr_item_sk: i64,
    pub(crate) cr_refunded_customer_sk: i64,
    pub(crate) cr_refunded_cdemo_sk: i64,
    pub(crate) cr_refunded_hdemo_sk: i64,
    pub(crate) cr_refunded_addr_sk: i64,
    pub(crate) cr_returning_customer_sk: i64,
    pub(crate) cr_returning_cdemo_sk: i64,
    pub(crate) cr_returning_hdemo_sk: i64,
    pub(crate) cr_returning_addr_sk: i64,
    pub(crate) cr_call_center_sk: i64,
    pub(crate) cr_catalog_page_sk: i64,
    pub(crate) cr_ship_mode_sk: i64,
    pub(crate) cr_warehouse_sk: i64,
    pub(crate) cr_reason_sk: i64,
    pub(crate) cr_order_number: i64,
    pub(crate) cr_pricing: Pricing,
}

impl CatalogReturnsRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        null_bit_map: i64,
        cr_returned_date_sk: i64,
        cr_returned_time_sk: i64,
        cr_item_sk: i64,
        cr_refunded_customer_sk: i64,
        cr_refunded_cdemo_sk: i64,
        cr_refunded_hdemo_sk: i64,
        cr_refunded_addr_sk: i64,
        cr_returning_customer_sk: i64,
        cr_returning_cdemo_sk: i64,
        cr_returning_hdemo_sk: i64,
        cr_returning_addr_sk: i64,
        cr_call_center_sk: i64,
        cr_catalog_page_sk: i64,
        cr_ship_mode_sk: i64,
        cr_warehouse_sk: i64,
        cr_reason_sk: i64,
        cr_order_number: i64,
        cr_pricing: Pricing,
    ) -> Self {
        CatalogReturnsRow {
            null_bit_map,
            cr_returned_date_sk,
            cr_returned_time_sk,
            cr_item_sk,
            cr_refunded_customer_sk,
            cr_refunded_cdemo_sk,
            cr_refunded_hdemo_sk,
            cr_refunded_addr_sk,
            cr_returning_customer_sk,
            cr_returning_cdemo_sk,
            cr_returning_hdemo_sk,
            cr_returning_addr_sk,
            cr_call_center_sk,
            cr_catalog_page_sk,
            cr_ship_mode_sk,
            cr_warehouse_sk,
            cr_reason_sk,
            cr_order_number,
            cr_pricing,
        }
    }

    fn is_null(&self, column: &CatalogReturnsGeneratorColumn) -> bool {
        let column_number = column.get_global_column_number();
        let first_column =
            CatalogReturnsGeneratorColumn::CrReturnedDateSk.get_global_column_number();
        let bit_position = column_number - first_column;
        (self.null_bit_map & (1 << bit_position)) != 0
    }

    pub fn null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    pub fn get_cr_returned_date_sk(&self) -> i64 {
        self.cr_returned_date_sk
    }

    pub fn get_cr_returned_time_sk(&self) -> i64 {
        self.cr_returned_time_sk
    }

    pub fn get_cr_item_sk(&self) -> i64 {
        self.cr_item_sk
    }

    pub fn get_cr_refunded_customer_sk(&self) -> i64 {
        self.cr_refunded_customer_sk
    }

    pub fn get_cr_refunded_cdemo_sk(&self) -> i64 {
        self.cr_refunded_cdemo_sk
    }

    pub fn get_cr_refunded_hdemo_sk(&self) -> i64 {
        self.cr_refunded_hdemo_sk
    }

    pub fn get_cr_refunded_addr_sk(&self) -> i64 {
        self.cr_refunded_addr_sk
    }

    pub fn get_cr_returning_customer_sk(&self) -> i64 {
        self.cr_returning_customer_sk
    }

    pub fn get_cr_returning_cdemo_sk(&self) -> i64 {
        self.cr_returning_cdemo_sk
    }

    pub fn get_cr_returning_hdemo_sk(&self) -> i64 {
        self.cr_returning_hdemo_sk
    }

    pub fn get_cr_returning_addr_sk(&self) -> i64 {
        self.cr_returning_addr_sk
    }

    pub fn get_cr_call_center_sk(&self) -> i64 {
        self.cr_call_center_sk
    }

    pub fn get_cr_catalog_page_sk(&self) -> i64 {
        self.cr_catalog_page_sk
    }

    pub fn get_cr_ship_mode_sk(&self) -> i64 {
        self.cr_ship_mode_sk
    }

    pub fn get_cr_warehouse_sk(&self) -> i64 {
        self.cr_warehouse_sk
    }

    pub fn get_cr_reason_sk(&self) -> i64 {
        self.cr_reason_sk
    }

    pub fn get_cr_order_number(&self) -> i64 {
        self.cr_order_number
    }

    pub fn get_cr_pricing(&self) -> &Pricing {
        &self.cr_pricing
    }

    /// DAT field for a surrogate key: NULL when the null bit is set or the
    /// key is negative.
    pub(crate) fn key_field(
        &self,
        value: i64,
        column: &CatalogReturnsGeneratorColumn,
    ) -> DatField<i64> {
        DatField::new(value, self.is_null(column) || value < 0)
    }

    /// DAT field for a regular value: NULL when the null bit is set.
    pub(crate) fn field<T>(&self, value: T, column: &CatalogReturnsGeneratorColumn) -> DatField<T> {
        DatField::new(value, self.is_null(column))
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces one
/// `|`-terminated field per column.
impl fmt::Display for CatalogReturnsRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CatalogReturnsGeneratorColumn::*;

        write!(
            f,
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|",
            self.key_field(self.cr_returned_date_sk, &CrReturnedDateSk),
            self.key_field(self.cr_returned_time_sk, &CrReturnedTimeSk),
            self.key_field(self.cr_item_sk, &CrItemSk),
            self.key_field(self.cr_refunded_customer_sk, &CrRefundedCustomerSk),
            self.key_field(self.cr_refunded_cdemo_sk, &CrRefundedCdemoSk),
            self.key_field(self.cr_refunded_hdemo_sk, &CrRefundedHdemoSk),
            self.key_field(self.cr_refunded_addr_sk, &CrRefundedAddrSk),
            self.key_field(self.cr_returning_customer_sk, &CrReturningCustomerSk),
            self.key_field(self.cr_returning_cdemo_sk, &CrReturningCdemoSk),
            self.key_field(self.cr_returning_hdemo_sk, &CrReturningHdemoSk),
            self.key_field(self.cr_returning_addr_sk, &CrReturningAddrSk),
            self.key_field(self.cr_call_center_sk, &CrCallCenterSk),
            self.key_field(self.cr_catalog_page_sk, &CrCatalogPageSk),
            self.key_field(self.cr_ship_mode_sk, &CrShipModeSk),
            self.key_field(self.cr_warehouse_sk, &CrWarehouseSk),
            self.key_field(self.cr_reason_sk, &CrReasonSk),
            self.field(self.cr_order_number, &CrOrderNumber),
            self.field(self.cr_pricing.get_quantity(), &CrPricingQuantity),
            self.field(self.cr_pricing.get_net_paid(), &CrPricingNetPaid),
            self.field(self.cr_pricing.get_ext_tax(), &CrPricingExtTax),
            self.field(
                self.cr_pricing.get_net_paid_including_tax(),
                &CrPricingNetPaidIncTax
            ),
            self.field(self.cr_pricing.get_fee(), &CrPricingFee),
            self.field(self.cr_pricing.get_ext_ship_cost(), &CrPricingExtShipCost),
            self.field(self.cr_pricing.get_refunded_cash(), &CrPricingRefundedCash),
            self.field(
                self.cr_pricing.get_reversed_charge(),
                &CrPricingReversedCharge
            ),
            self.field(self.cr_pricing.get_store_credit(), &CrPricingStoreCredit),
            self.field(self.cr_pricing.get_net_loss(), &CrPricingNetLoss),
        )
    }
}

use crate::generator::GeneratorColumn;
