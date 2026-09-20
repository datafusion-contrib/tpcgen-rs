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

//! Web returns row definition

use crate::generator::{GeneratorColumn, WebReturnsGeneratorColumn};
use crate::row::table_row::DatField;
use crate::types::Pricing;
use std::fmt;

/// Row structure for web_returns table
#[derive(Debug, Clone)]
pub struct WebReturnsRow {
    null_bit_map: i64,
    pub(crate) wr_returned_date_sk: i64,
    pub(crate) wr_returned_time_sk: i64,
    pub(crate) wr_item_sk: i64,
    pub(crate) wr_refunded_customer_sk: i64,
    pub(crate) wr_refunded_cdemo_sk: i64,
    pub(crate) wr_refunded_hdemo_sk: i64,
    pub(crate) wr_refunded_addr_sk: i64,
    pub(crate) wr_returning_customer_sk: i64,
    pub(crate) wr_returning_cdemo_sk: i64,
    pub(crate) wr_returning_hdemo_sk: i64,
    pub(crate) wr_returning_addr_sk: i64,
    pub(crate) wr_web_page_sk: i64,
    pub(crate) wr_reason_sk: i64,
    pub(crate) wr_order_number: i64,
    pub(crate) wr_pricing: Pricing,
}

impl WebReturnsRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        null_bit_map: i64,
        wr_returned_date_sk: i64,
        wr_returned_time_sk: i64,
        wr_item_sk: i64,
        wr_refunded_customer_sk: i64,
        wr_refunded_cdemo_sk: i64,
        wr_refunded_hdemo_sk: i64,
        wr_refunded_addr_sk: i64,
        wr_returning_customer_sk: i64,
        wr_returning_cdemo_sk: i64,
        wr_returning_hdemo_sk: i64,
        wr_returning_addr_sk: i64,
        wr_web_page_sk: i64,
        wr_reason_sk: i64,
        wr_order_number: i64,
        wr_pricing: Pricing,
    ) -> Self {
        WebReturnsRow {
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
        }
    }

    fn is_null(&self, column: WebReturnsGeneratorColumn) -> bool {
        let bit_position = column.get_global_column_number()
            - WebReturnsGeneratorColumn::WrReturnedDateSk.get_global_column_number();
        (self.null_bit_map & (1 << bit_position)) != 0
    }

    pub fn null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    pub fn get_wr_returned_date_sk(&self) -> i64 {
        self.wr_returned_date_sk
    }

    pub fn get_wr_returned_time_sk(&self) -> i64 {
        self.wr_returned_time_sk
    }

    pub fn get_wr_item_sk(&self) -> i64 {
        self.wr_item_sk
    }

    pub fn get_wr_refunded_customer_sk(&self) -> i64 {
        self.wr_refunded_customer_sk
    }

    pub fn get_wr_refunded_cdemo_sk(&self) -> i64 {
        self.wr_refunded_cdemo_sk
    }

    pub fn get_wr_refunded_hdemo_sk(&self) -> i64 {
        self.wr_refunded_hdemo_sk
    }

    pub fn get_wr_refunded_addr_sk(&self) -> i64 {
        self.wr_refunded_addr_sk
    }

    pub fn get_wr_returning_customer_sk(&self) -> i64 {
        self.wr_returning_customer_sk
    }

    pub fn get_wr_returning_cdemo_sk(&self) -> i64 {
        self.wr_returning_cdemo_sk
    }

    pub fn get_wr_returning_hdemo_sk(&self) -> i64 {
        self.wr_returning_hdemo_sk
    }

    pub fn get_wr_returning_addr_sk(&self) -> i64 {
        self.wr_returning_addr_sk
    }

    pub fn get_wr_web_page_sk(&self) -> i64 {
        self.wr_web_page_sk
    }

    pub fn get_wr_reason_sk(&self) -> i64 {
        self.wr_reason_sk
    }

    pub fn get_wr_order_number(&self) -> i64 {
        self.wr_order_number
    }

    pub fn get_wr_pricing(&self) -> &Pricing {
        &self.wr_pricing
    }
}

/// DAT field helper: NULL is driven purely by the null bit (web rows
/// apply no key sentinel check).
impl WebReturnsRow {
    pub(crate) fn field<T>(&self, value: T, column: WebReturnsGeneratorColumn) -> DatField<T> {
        DatField::new(value, self.is_null(column))
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces one
/// `|`-terminated field per column.
impl fmt::Display for WebReturnsRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use WebReturnsGeneratorColumn::*;

        write!(
            f,
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|",
            self.field(self.wr_returned_date_sk, WrReturnedDateSk),
            self.field(self.wr_returned_time_sk, WrReturnedTimeSk),
            self.field(self.wr_item_sk, WrItemSk),
            self.field(self.wr_refunded_customer_sk, WrRefundedCustomerSk),
            self.field(self.wr_refunded_cdemo_sk, WrRefundedCdemoSk),
            self.field(self.wr_refunded_hdemo_sk, WrRefundedHdemoSk),
            self.field(self.wr_refunded_addr_sk, WrRefundedAddrSk),
            self.field(self.wr_returning_customer_sk, WrReturningCustomerSk),
            self.field(self.wr_returning_cdemo_sk, WrReturningCdemoSk),
            self.field(self.wr_returning_hdemo_sk, WrReturningHdemoSk),
            self.field(self.wr_returning_addr_sk, WrReturningAddrSk),
            self.field(self.wr_web_page_sk, WrWebPageSk),
            self.field(self.wr_reason_sk, WrReasonSk),
            self.field(self.wr_order_number, WrOrderNumber),
            self.field(self.wr_pricing.get_quantity(), WrPricingQuantity),
            self.field(self.wr_pricing.get_net_paid(), WrPricingNetPaid),
            self.field(self.wr_pricing.get_ext_tax(), WrPricingExtTax),
            self.field(
                self.wr_pricing.get_net_paid_including_tax(),
                WrPricingNetPaidIncTax
            ),
            self.field(self.wr_pricing.get_fee(), WrPricingFee),
            self.field(self.wr_pricing.get_ext_ship_cost(), WrPricingExtShipCost),
            self.field(self.wr_pricing.get_refunded_cash(), WrPricingRefundedCash),
            self.field(
                self.wr_pricing.get_reversed_charge(),
                WrPricingReversedCharge
            ),
            self.field(self.wr_pricing.get_store_credit(), WrPricingStoreCredit),
            self.field(self.wr_pricing.get_net_loss(), WrPricingNetLoss),
        )
    }
}
