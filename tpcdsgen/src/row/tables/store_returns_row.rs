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

//! Store returns row data structure

use crate::generator::{GeneratorColumn, StoreReturnsGeneratorColumn};
use crate::row::table_row::DatField;
use crate::types::Pricing;
use std::fmt;

/// Row data structure for the store_returns table
#[derive(Debug, Clone)]
pub struct StoreReturnsRow {
    null_bit_map: i64,
    pub(crate) sr_returned_date_sk: i64,
    pub(crate) sr_returned_time_sk: i64,
    pub(crate) sr_item_sk: i64,
    pub(crate) sr_customer_sk: i64,
    pub(crate) sr_cdemo_sk: i64,
    pub(crate) sr_hdemo_sk: i64,
    pub(crate) sr_addr_sk: i64,
    pub(crate) sr_store_sk: i64,
    pub(crate) sr_reason_sk: i64,
    pub(crate) sr_ticket_number: i64,
    pub(crate) sr_pricing: Pricing,
}

impl StoreReturnsRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        null_bit_map: i64,
        sr_returned_date_sk: i64,
        sr_returned_time_sk: i64,
        sr_item_sk: i64,
        sr_customer_sk: i64,
        sr_cdemo_sk: i64,
        sr_hdemo_sk: i64,
        sr_addr_sk: i64,
        sr_store_sk: i64,
        sr_reason_sk: i64,
        sr_ticket_number: i64,
        sr_pricing: Pricing,
    ) -> Self {
        StoreReturnsRow {
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
        }
    }

    pub(crate) fn is_null_at(&self, column: StoreReturnsGeneratorColumn) -> bool {
        let bit_position = column.get_global_column_number()
            - StoreReturnsGeneratorColumn::SrReturnedDateSk.get_global_column_number();
        (self.null_bit_map & (1 << bit_position)) != 0
    }

    pub fn null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    pub fn get_sr_returned_date_sk(&self) -> i64 {
        self.sr_returned_date_sk
    }

    pub fn get_sr_returned_time_sk(&self) -> i64 {
        self.sr_returned_time_sk
    }

    pub fn get_sr_item_sk(&self) -> i64 {
        self.sr_item_sk
    }

    pub fn get_sr_customer_sk(&self) -> i64 {
        self.sr_customer_sk
    }

    pub fn get_sr_cdemo_sk(&self) -> i64 {
        self.sr_cdemo_sk
    }

    pub fn get_sr_hdemo_sk(&self) -> i64 {
        self.sr_hdemo_sk
    }

    pub fn get_sr_addr_sk(&self) -> i64 {
        self.sr_addr_sk
    }

    pub fn get_sr_store_sk(&self) -> i64 {
        self.sr_store_sk
    }

    pub fn get_sr_reason_sk(&self) -> i64 {
        self.sr_reason_sk
    }

    pub fn get_sr_ticket_number(&self) -> i64 {
        self.sr_ticket_number
    }

    pub fn get_sr_pricing(&self) -> &Pricing {
        &self.sr_pricing
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces one
/// `|`-terminated field per column.
impl fmt::Display for StoreReturnsRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use StoreReturnsGeneratorColumn::*;

        write!(
            f,
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|",
            DatField::key(self.sr_returned_date_sk, self.is_null_at(SrReturnedDateSk)),
            DatField::key(self.sr_returned_time_sk, self.is_null_at(SrReturnedTimeSk)),
            DatField::key(self.sr_item_sk, self.is_null_at(SrItemSk)),
            DatField::key(self.sr_customer_sk, self.is_null_at(SrCustomerSk)),
            DatField::key(self.sr_cdemo_sk, self.is_null_at(SrCdemoSk)),
            DatField::key(self.sr_hdemo_sk, self.is_null_at(SrHdemoSk)),
            DatField::key(self.sr_addr_sk, self.is_null_at(SrAddrSk)),
            DatField::key(self.sr_store_sk, self.is_null_at(SrStoreSk)),
            DatField::key(self.sr_reason_sk, self.is_null_at(SrReasonSk)),
            DatField::key(self.sr_ticket_number, self.is_null_at(SrTicketNumber)),
            DatField::new(
                self.sr_pricing.get_quantity(),
                self.is_null_at(SrPricingQuantity)
            ),
            DatField::new(
                self.sr_pricing.get_net_paid(),
                self.is_null_at(SrPricingNetPaid)
            ),
            DatField::new(
                self.sr_pricing.get_ext_tax(),
                self.is_null_at(SrPricingExtTax)
            ),
            DatField::new(
                self.sr_pricing.get_net_paid_including_tax(),
                self.is_null_at(SrPricingNetPaidIncTax)
            ),
            DatField::new(self.sr_pricing.get_fee(), self.is_null_at(SrPricingFee)),
            DatField::new(
                self.sr_pricing.get_ext_ship_cost(),
                self.is_null_at(SrPricingExtShipCost)
            ),
            DatField::new(
                self.sr_pricing.get_refunded_cash(),
                self.is_null_at(SrPricingRefundedCash)
            ),
            DatField::new(
                self.sr_pricing.get_reversed_charge(),
                self.is_null_at(SrPricingReversedCharge)
            ),
            DatField::new(
                self.sr_pricing.get_store_credit(),
                self.is_null_at(SrPricingStoreCredit)
            ),
            DatField::new(
                self.sr_pricing.get_net_loss(),
                self.is_null_at(SrPricingNetLoss)
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::row::dat_values;
    use crate::types::Decimal;

    fn create_test_pricing() -> Pricing {
        Pricing::new(
            Decimal::new(1000, 2).unwrap(), // wholesale_cost: 10.00
            Decimal::new(1500, 2).unwrap(), // list_price: 15.00
            Decimal::new(1200, 2).unwrap(), // sales_price: 12.00
            5,                              // quantity
            Decimal::new(300, 2).unwrap(),  // ext_discount_amount: 3.00
            Decimal::new(6000, 2).unwrap(), // ext_sales_price: 60.00
            Decimal::new(5000, 2).unwrap(), // ext_wholesale_cost: 50.00
            Decimal::new(7500, 2).unwrap(), // ext_list_price: 75.00
            Decimal::new(8, 2).unwrap(),    // tax_percent: 0.08
            Decimal::new(480, 2).unwrap(),  // ext_tax: 4.80
            Decimal::new(100, 2).unwrap(),  // coupon_amount: 1.00
            Decimal::new(200, 2).unwrap(),  // ship_cost: 2.00
            Decimal::new(1000, 2).unwrap(), // ext_ship_cost: 10.00
            Decimal::new(5900, 2).unwrap(), // net_paid: 59.00
            Decimal::new(6380, 2).unwrap(), // net_paid_including_tax: 63.80
            Decimal::new(6900, 2).unwrap(), // net_paid_including_shipping: 69.00
            Decimal::new(7380, 2).unwrap(), // net_paid_including_shipping_and_tax: 73.80
            Decimal::new(900, 2).unwrap(),  // net_profit: 9.00
            Decimal::new(2000, 2).unwrap(), // refunded_cash: 20.00
            Decimal::new(1000, 2).unwrap(), // reversed_charge: 10.00
            Decimal::new(2900, 2).unwrap(), // store_credit: 29.00
            Decimal::new(500, 2).unwrap(),  // fee: 5.00
            Decimal::new(1580, 2).unwrap(), // net_loss: 15.80
        )
    }

    #[test]
    fn test_store_returns_row_creation() {
        let pricing = create_test_pricing();
        let row = StoreReturnsRow::new(
            0,       // null_bit_map
            2451545, // sr_returned_date_sk
            36000,   // sr_returned_time_sk
            1,       // sr_item_sk
            100,     // sr_customer_sk
            200,     // sr_cdemo_sk
            300,     // sr_hdemo_sk
            400,     // sr_addr_sk
            500,     // sr_store_sk
            600,     // sr_reason_sk
            1,       // sr_ticket_number
            pricing,
        );

        let values = dat_values(&row);
        assert_eq!(values.len(), 20);
        assert_eq!(values[0], "2451545"); // sr_returned_date_sk
        assert_eq!(values[9], "1"); // sr_ticket_number
    }

    #[test]
    fn test_store_returns_row_null_handling() {
        let pricing = create_test_pricing();
        // Set bit for sr_returned_time_sk (position 1)
        let row = StoreReturnsRow::new(
            0b10, // null_bit_map - second bit set
            2451545, 36000, 1, 100, 200, 300, 400, 500, 600, 1, pricing,
        );

        let values = dat_values(&row);
        assert_eq!(values[0], "2451545"); // not null
        assert_eq!(values[1], ""); // null (bit 1 set)
    }
}
