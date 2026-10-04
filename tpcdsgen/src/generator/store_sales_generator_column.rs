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

//! Store sales generator column definitions

use crate::column::Table;
use crate::generator::GeneratorColumn;

/// Enum representing all generator columns for the store_sales table
/// Global column numbers 314-339
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StoreSalesGeneratorColumn {
    SsSoldDateSk,              // 314
    SsSoldTimeSk,              // 315
    SsSoldItemSk,              // 316
    SsSoldCustomerSk,          // 317
    SsSoldCdemoSk,             // 318
    SsSoldHdemoSk,             // 319
    SsSoldAddrSk,              // 320
    SsSoldStoreSk,             // 321
    SsSoldPromoSk,             // 322
    SsTicketNumber,            // 323
    SsPricingQuantity,         // 324
    SsPricingWholesaleCost,    // 325
    SsPricingListPrice,        // 326
    SsPricingSalesPrice,       // 327
    SsPricingCouponAmt,        // 328
    SsPricingExtSalesPrice,    // 329
    SsPricingExtWholesaleCost, // 330
    SsPricingExtListPrice,     // 331
    SsPricingExtTax,           // 332
    SsPricingNetPaid,          // 333
    SsPricingNetPaidIncTax,    // 334
    SsPricingNetProfit,        // 335
    SrIsReturned,              // 336
    SsPricing,                 // 337
    SsNulls,                   // 338
    SsPermutation,             // 339
}

impl GeneratorColumn for StoreSalesGeneratorColumn {
    fn get_table(&self) -> Table {
        Table::StoreSales
    }

    fn get_global_column_number(&self) -> i32 {
        match self {
            StoreSalesGeneratorColumn::SsSoldDateSk => 314,
            StoreSalesGeneratorColumn::SsSoldTimeSk => 315,
            StoreSalesGeneratorColumn::SsSoldItemSk => 316,
            StoreSalesGeneratorColumn::SsSoldCustomerSk => 317,
            StoreSalesGeneratorColumn::SsSoldCdemoSk => 318,
            StoreSalesGeneratorColumn::SsSoldHdemoSk => 319,
            StoreSalesGeneratorColumn::SsSoldAddrSk => 320,
            StoreSalesGeneratorColumn::SsSoldStoreSk => 321,
            StoreSalesGeneratorColumn::SsSoldPromoSk => 322,
            StoreSalesGeneratorColumn::SsTicketNumber => 323,
            StoreSalesGeneratorColumn::SsPricingQuantity => 324,
            StoreSalesGeneratorColumn::SsPricingWholesaleCost => 325,
            StoreSalesGeneratorColumn::SsPricingListPrice => 326,
            StoreSalesGeneratorColumn::SsPricingSalesPrice => 327,
            StoreSalesGeneratorColumn::SsPricingCouponAmt => 328,
            StoreSalesGeneratorColumn::SsPricingExtSalesPrice => 329,
            StoreSalesGeneratorColumn::SsPricingExtWholesaleCost => 330,
            StoreSalesGeneratorColumn::SsPricingExtListPrice => 331,
            StoreSalesGeneratorColumn::SsPricingExtTax => 332,
            StoreSalesGeneratorColumn::SsPricingNetPaid => 333,
            StoreSalesGeneratorColumn::SsPricingNetPaidIncTax => 334,
            StoreSalesGeneratorColumn::SsPricingNetProfit => 335,
            StoreSalesGeneratorColumn::SrIsReturned => 336,
            StoreSalesGeneratorColumn::SsPricing => 337,
            StoreSalesGeneratorColumn::SsNulls => 338,
            StoreSalesGeneratorColumn::SsPermutation => 339,
        }
    }

    fn get_seeds_per_row(&self) -> u32 {
        match self {
            StoreSalesGeneratorColumn::SsSoldDateSk => 2,
            StoreSalesGeneratorColumn::SsSoldTimeSk => 2,
            StoreSalesGeneratorColumn::SsSoldItemSk => 1,
            StoreSalesGeneratorColumn::SsSoldCustomerSk => 1,
            StoreSalesGeneratorColumn::SsSoldCdemoSk => 1,
            StoreSalesGeneratorColumn::SsSoldHdemoSk => 1,
            StoreSalesGeneratorColumn::SsSoldAddrSk => 1,
            StoreSalesGeneratorColumn::SsSoldStoreSk => 1,
            StoreSalesGeneratorColumn::SsSoldPromoSk => 16,
            StoreSalesGeneratorColumn::SsTicketNumber => 1,
            StoreSalesGeneratorColumn::SsPricingQuantity => 1,
            StoreSalesGeneratorColumn::SsPricingWholesaleCost => 0,
            StoreSalesGeneratorColumn::SsPricingListPrice => 0,
            StoreSalesGeneratorColumn::SsPricingSalesPrice => 0,
            StoreSalesGeneratorColumn::SsPricingCouponAmt => 0,
            StoreSalesGeneratorColumn::SsPricingExtSalesPrice => 0,
            StoreSalesGeneratorColumn::SsPricingExtWholesaleCost => 0,
            StoreSalesGeneratorColumn::SsPricingExtListPrice => 0,
            StoreSalesGeneratorColumn::SsPricingExtTax => 0,
            StoreSalesGeneratorColumn::SsPricingNetPaid => 0,
            StoreSalesGeneratorColumn::SsPricingNetPaidIncTax => 0,
            StoreSalesGeneratorColumn::SsPricingNetProfit => 0,
            StoreSalesGeneratorColumn::SrIsReturned => 16,
            StoreSalesGeneratorColumn::SsPricing => 128,
            StoreSalesGeneratorColumn::SsNulls => 32,
            StoreSalesGeneratorColumn::SsPermutation => 0,
        }
    }
}

impl StoreSalesGeneratorColumn {
    /// Returns all variants in order
    pub fn all_variants() -> &'static [StoreSalesGeneratorColumn] {
        use StoreSalesGeneratorColumn::*;
        &[
            SsSoldDateSk,
            SsSoldTimeSk,
            SsSoldItemSk,
            SsSoldCustomerSk,
            SsSoldCdemoSk,
            SsSoldHdemoSk,
            SsSoldAddrSk,
            SsSoldStoreSk,
            SsSoldPromoSk,
            SsTicketNumber,
            SsPricingQuantity,
            SsPricingWholesaleCost,
            SsPricingListPrice,
            SsPricingSalesPrice,
            SsPricingCouponAmt,
            SsPricingExtSalesPrice,
            SsPricingExtWholesaleCost,
            SsPricingExtListPrice,
            SsPricingExtTax,
            SsPricingNetPaid,
            SsPricingNetPaidIncTax,
            SsPricingNetProfit,
            SrIsReturned,
            SsPricing,
            SsNulls,
            SsPermutation,
        ]
    }
}
