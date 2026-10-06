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

//! Store returns generator column definitions

use crate::column::Table;
use crate::generator::GeneratorColumn;

/// Enum representing all generator columns for the store_returns table
/// Global column numbers 292-313
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StoreReturnsGeneratorColumn {
    SrReturnedDateSk,        // 292
    SrReturnedTimeSk,        // 293
    SrItemSk,                // 294
    SrCustomerSk,            // 295
    SrCdemoSk,               // 296
    SrHdemoSk,               // 297
    SrAddrSk,                // 298
    SrStoreSk,               // 299
    SrReasonSk,              // 300
    SrTicketNumber,          // 301
    SrPricingQuantity,       // 302
    SrPricingNetPaid,        // 303
    SrPricingExtTax,         // 304
    SrPricingNetPaidIncTax,  // 305
    SrPricingFee,            // 306
    SrPricingExtShipCost,    // 307
    SrPricingRefundedCash,   // 308
    SrPricingReversedCharge, // 309
    SrPricingStoreCredit,    // 310
    SrPricingNetLoss,        // 311
    SrPricing,               // 312
    SrNulls,                 // 313
}

impl GeneratorColumn for StoreReturnsGeneratorColumn {
    fn get_table(&self) -> Table {
        Table::StoreReturns
    }

    fn get_global_column_number(&self) -> i32 {
        match self {
            StoreReturnsGeneratorColumn::SrReturnedDateSk => 292,
            StoreReturnsGeneratorColumn::SrReturnedTimeSk => 293,
            StoreReturnsGeneratorColumn::SrItemSk => 294,
            StoreReturnsGeneratorColumn::SrCustomerSk => 295,
            StoreReturnsGeneratorColumn::SrCdemoSk => 296,
            StoreReturnsGeneratorColumn::SrHdemoSk => 297,
            StoreReturnsGeneratorColumn::SrAddrSk => 298,
            StoreReturnsGeneratorColumn::SrStoreSk => 299,
            StoreReturnsGeneratorColumn::SrReasonSk => 300,
            StoreReturnsGeneratorColumn::SrTicketNumber => 301,
            StoreReturnsGeneratorColumn::SrPricingQuantity => 302,
            StoreReturnsGeneratorColumn::SrPricingNetPaid => 303,
            StoreReturnsGeneratorColumn::SrPricingExtTax => 304,
            StoreReturnsGeneratorColumn::SrPricingNetPaidIncTax => 305,
            StoreReturnsGeneratorColumn::SrPricingFee => 306,
            StoreReturnsGeneratorColumn::SrPricingExtShipCost => 307,
            StoreReturnsGeneratorColumn::SrPricingRefundedCash => 308,
            StoreReturnsGeneratorColumn::SrPricingReversedCharge => 309,
            StoreReturnsGeneratorColumn::SrPricingStoreCredit => 310,
            StoreReturnsGeneratorColumn::SrPricingNetLoss => 311,
            StoreReturnsGeneratorColumn::SrPricing => 312,
            StoreReturnsGeneratorColumn::SrNulls => 313,
        }
    }

    fn get_seeds_per_row(&self) -> u32 {
        match self {
            StoreReturnsGeneratorColumn::SrReturnedDateSk => 32,
            StoreReturnsGeneratorColumn::SrReturnedTimeSk => 32,
            StoreReturnsGeneratorColumn::SrItemSk => 16,
            StoreReturnsGeneratorColumn::SrCustomerSk => 16,
            StoreReturnsGeneratorColumn::SrCdemoSk => 16,
            StoreReturnsGeneratorColumn::SrHdemoSk => 16,
            StoreReturnsGeneratorColumn::SrAddrSk => 16,
            StoreReturnsGeneratorColumn::SrStoreSk => 16,
            StoreReturnsGeneratorColumn::SrReasonSk => 16,
            StoreReturnsGeneratorColumn::SrTicketNumber => 16,
            StoreReturnsGeneratorColumn::SrPricingQuantity => 0,
            StoreReturnsGeneratorColumn::SrPricingNetPaid => 0,
            StoreReturnsGeneratorColumn::SrPricingExtTax => 0,
            StoreReturnsGeneratorColumn::SrPricingNetPaidIncTax => 0,
            StoreReturnsGeneratorColumn::SrPricingFee => 0,
            StoreReturnsGeneratorColumn::SrPricingExtShipCost => 0,
            StoreReturnsGeneratorColumn::SrPricingRefundedCash => 0,
            StoreReturnsGeneratorColumn::SrPricingReversedCharge => 0,
            StoreReturnsGeneratorColumn::SrPricingStoreCredit => 0,
            StoreReturnsGeneratorColumn::SrPricingNetLoss => 0,
            StoreReturnsGeneratorColumn::SrPricing => 80,
            StoreReturnsGeneratorColumn::SrNulls => 32,
        }
    }
}

impl StoreReturnsGeneratorColumn {
    /// Returns all variants in order
    pub fn all_variants() -> &'static [StoreReturnsGeneratorColumn] {
        use StoreReturnsGeneratorColumn::*;
        &[
            SrReturnedDateSk,
            SrReturnedTimeSk,
            SrItemSk,
            SrCustomerSk,
            SrCdemoSk,
            SrHdemoSk,
            SrAddrSk,
            SrStoreSk,
            SrReasonSk,
            SrTicketNumber,
            SrPricingQuantity,
            SrPricingNetPaid,
            SrPricingExtTax,
            SrPricingNetPaidIncTax,
            SrPricingFee,
            SrPricingExtShipCost,
            SrPricingRefundedCash,
            SrPricingReversedCharge,
            SrPricingStoreCredit,
            SrPricingNetLoss,
            SrPricing,
            SrNulls,
        ]
    }
}
