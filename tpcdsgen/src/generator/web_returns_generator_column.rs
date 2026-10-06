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

//! Web returns generator column definitions

use crate::column::Table;
use crate::generator::GeneratorColumn;

/// Enum representing all generator columns for the web_returns table
/// Global column numbers 383-408
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WebReturnsGeneratorColumn {
    WrReturnedDateSk,        // 383
    WrReturnedTimeSk,        // 384
    WrItemSk,                // 385
    WrRefundedCustomerSk,    // 386
    WrRefundedCdemoSk,       // 387
    WrRefundedHdemoSk,       // 388
    WrRefundedAddrSk,        // 389
    WrReturningCustomerSk,   // 390
    WrReturningCdemoSk,      // 391
    WrReturningHdemoSk,      // 392
    WrReturningAddrSk,       // 393
    WrWebPageSk,             // 394
    WrReasonSk,              // 395
    WrOrderNumber,           // 396
    WrPricingQuantity,       // 397
    WrPricingNetPaid,        // 398
    WrPricingExtTax,         // 399
    WrPricingNetPaidIncTax,  // 400
    WrPricingFee,            // 401
    WrPricingExtShipCost,    // 402
    WrPricingRefundedCash,   // 403
    WrPricingReversedCharge, // 404
    WrPricingStoreCredit,    // 405
    WrPricingNetLoss,        // 406
    WrPricing,               // 407
    WrNulls,                 // 408
}

impl GeneratorColumn for WebReturnsGeneratorColumn {
    fn get_table(&self) -> Table {
        Table::WebReturns
    }

    fn get_global_column_number(&self) -> i32 {
        match self {
            WebReturnsGeneratorColumn::WrReturnedDateSk => 383,
            WebReturnsGeneratorColumn::WrReturnedTimeSk => 384,
            WebReturnsGeneratorColumn::WrItemSk => 385,
            WebReturnsGeneratorColumn::WrRefundedCustomerSk => 386,
            WebReturnsGeneratorColumn::WrRefundedCdemoSk => 387,
            WebReturnsGeneratorColumn::WrRefundedHdemoSk => 388,
            WebReturnsGeneratorColumn::WrRefundedAddrSk => 389,
            WebReturnsGeneratorColumn::WrReturningCustomerSk => 390,
            WebReturnsGeneratorColumn::WrReturningCdemoSk => 391,
            WebReturnsGeneratorColumn::WrReturningHdemoSk => 392,
            WebReturnsGeneratorColumn::WrReturningAddrSk => 393,
            WebReturnsGeneratorColumn::WrWebPageSk => 394,
            WebReturnsGeneratorColumn::WrReasonSk => 395,
            WebReturnsGeneratorColumn::WrOrderNumber => 396,
            WebReturnsGeneratorColumn::WrPricingQuantity => 397,
            WebReturnsGeneratorColumn::WrPricingNetPaid => 398,
            WebReturnsGeneratorColumn::WrPricingExtTax => 399,
            WebReturnsGeneratorColumn::WrPricingNetPaidIncTax => 400,
            WebReturnsGeneratorColumn::WrPricingFee => 401,
            WebReturnsGeneratorColumn::WrPricingExtShipCost => 402,
            WebReturnsGeneratorColumn::WrPricingRefundedCash => 403,
            WebReturnsGeneratorColumn::WrPricingReversedCharge => 404,
            WebReturnsGeneratorColumn::WrPricingStoreCredit => 405,
            WebReturnsGeneratorColumn::WrPricingNetLoss => 406,
            WebReturnsGeneratorColumn::WrPricing => 407,
            WebReturnsGeneratorColumn::WrNulls => 408,
        }
    }

    fn get_seeds_per_row(&self) -> u32 {
        match self {
            WebReturnsGeneratorColumn::WrReturnedDateSk => 32,
            WebReturnsGeneratorColumn::WrReturnedTimeSk => 32,
            WebReturnsGeneratorColumn::WrItemSk => 16,
            WebReturnsGeneratorColumn::WrRefundedCustomerSk => 16,
            WebReturnsGeneratorColumn::WrRefundedCdemoSk => 16,
            WebReturnsGeneratorColumn::WrRefundedHdemoSk => 16,
            WebReturnsGeneratorColumn::WrRefundedAddrSk => 16,
            WebReturnsGeneratorColumn::WrReturningCustomerSk => 16,
            WebReturnsGeneratorColumn::WrReturningCdemoSk => 16,
            WebReturnsGeneratorColumn::WrReturningHdemoSk => 16,
            WebReturnsGeneratorColumn::WrReturningAddrSk => 16,
            WebReturnsGeneratorColumn::WrWebPageSk => 16,
            WebReturnsGeneratorColumn::WrReasonSk => 16,
            WebReturnsGeneratorColumn::WrOrderNumber => 0,
            WebReturnsGeneratorColumn::WrPricingQuantity => 0,
            WebReturnsGeneratorColumn::WrPricingNetPaid => 0,
            WebReturnsGeneratorColumn::WrPricingExtTax => 0,
            WebReturnsGeneratorColumn::WrPricingNetPaidIncTax => 0,
            WebReturnsGeneratorColumn::WrPricingFee => 0,
            WebReturnsGeneratorColumn::WrPricingExtShipCost => 0,
            WebReturnsGeneratorColumn::WrPricingRefundedCash => 0,
            WebReturnsGeneratorColumn::WrPricingReversedCharge => 0,
            WebReturnsGeneratorColumn::WrPricingStoreCredit => 0,
            WebReturnsGeneratorColumn::WrPricingNetLoss => 0,
            WebReturnsGeneratorColumn::WrPricing => 80,
            WebReturnsGeneratorColumn::WrNulls => 32,
        }
    }
}

impl WebReturnsGeneratorColumn {
    /// Returns all variants in order
    pub fn all_variants() -> &'static [WebReturnsGeneratorColumn] {
        use WebReturnsGeneratorColumn::*;
        &[
            WrReturnedDateSk,
            WrReturnedTimeSk,
            WrItemSk,
            WrRefundedCustomerSk,
            WrRefundedCdemoSk,
            WrRefundedHdemoSk,
            WrRefundedAddrSk,
            WrReturningCustomerSk,
            WrReturningCdemoSk,
            WrReturningHdemoSk,
            WrReturningAddrSk,
            WrWebPageSk,
            WrReasonSk,
            WrOrderNumber,
            WrPricingQuantity,
            WrPricingNetPaid,
            WrPricingExtTax,
            WrPricingNetPaidIncTax,
            WrPricingFee,
            WrPricingExtShipCost,
            WrPricingRefundedCash,
            WrPricingReversedCharge,
            WrPricingStoreCredit,
            WrPricingNetLoss,
            WrPricing,
            WrNulls,
        ]
    }
}
