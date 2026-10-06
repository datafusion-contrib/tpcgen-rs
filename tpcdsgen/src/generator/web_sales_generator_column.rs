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

//! Web sales generator column definitions

use crate::column::Table;
use crate::generator::GeneratorColumn;

/// Enum representing all generator columns for the web_sales table
/// Global column numbers 409-446
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WebSalesGeneratorColumn {
    WsSoldDateSk,               // 409
    WsSoldTimeSk,               // 410
    WsShipDateSk,               // 411
    WsItemSk,                   // 412
    WsBillCustomerSk,           // 413
    WsBillCdemoSk,              // 414
    WsBillHdemoSk,              // 415
    WsBillAddrSk,               // 416
    WsShipCustomerSk,           // 417
    WsShipCdemoSk,              // 418
    WsShipHdemoSk,              // 419
    WsShipAddrSk,               // 420
    WsWebPageSk,                // 421
    WsWebSiteSk,                // 422
    WsShipModeSk,               // 423
    WsWarehouseSk,              // 424
    WsPromoSk,                  // 425
    WsOrderNumber,              // 426
    WsPricingQuantity,          // 427
    WsPricingWholesaleCost,     // 428
    WsPricingListPrice,         // 429
    WsPricingSalesPrice,        // 430
    WsPricingExtDiscountAmt,    // 431
    WsPricingExtSalesPrice,     // 432
    WsPricingExtWholesaleCost,  // 433
    WsPricingExtListPrice,      // 434
    WsPricingExtTax,            // 435
    WsPricingCouponAmt,         // 436
    WsPricingExtShipCost,       // 437
    WsPricingNetPaid,           // 438
    WsPricingNetPaidIncTax,     // 439
    WsPricingNetPaidIncShip,    // 440
    WsPricingNetPaidIncShipTax, // 441
    WsPricingNetProfit,         // 442
    WsPricing,                  // 443
    WsNulls,                    // 444
    WrIsReturned,               // 445
    WsPermutation,              // 446
}

impl GeneratorColumn for WebSalesGeneratorColumn {
    fn get_table(&self) -> Table {
        Table::WebSales
    }

    fn get_global_column_number(&self) -> i32 {
        match self {
            WebSalesGeneratorColumn::WsSoldDateSk => 409,
            WebSalesGeneratorColumn::WsSoldTimeSk => 410,
            WebSalesGeneratorColumn::WsShipDateSk => 411,
            WebSalesGeneratorColumn::WsItemSk => 412,
            WebSalesGeneratorColumn::WsBillCustomerSk => 413,
            WebSalesGeneratorColumn::WsBillCdemoSk => 414,
            WebSalesGeneratorColumn::WsBillHdemoSk => 415,
            WebSalesGeneratorColumn::WsBillAddrSk => 416,
            WebSalesGeneratorColumn::WsShipCustomerSk => 417,
            WebSalesGeneratorColumn::WsShipCdemoSk => 418,
            WebSalesGeneratorColumn::WsShipHdemoSk => 419,
            WebSalesGeneratorColumn::WsShipAddrSk => 420,
            WebSalesGeneratorColumn::WsWebPageSk => 421,
            WebSalesGeneratorColumn::WsWebSiteSk => 422,
            WebSalesGeneratorColumn::WsShipModeSk => 423,
            WebSalesGeneratorColumn::WsWarehouseSk => 424,
            WebSalesGeneratorColumn::WsPromoSk => 425,
            WebSalesGeneratorColumn::WsOrderNumber => 426,
            WebSalesGeneratorColumn::WsPricingQuantity => 427,
            WebSalesGeneratorColumn::WsPricingWholesaleCost => 428,
            WebSalesGeneratorColumn::WsPricingListPrice => 429,
            WebSalesGeneratorColumn::WsPricingSalesPrice => 430,
            WebSalesGeneratorColumn::WsPricingExtDiscountAmt => 431,
            WebSalesGeneratorColumn::WsPricingExtSalesPrice => 432,
            WebSalesGeneratorColumn::WsPricingExtWholesaleCost => 433,
            WebSalesGeneratorColumn::WsPricingExtListPrice => 434,
            WebSalesGeneratorColumn::WsPricingExtTax => 435,
            WebSalesGeneratorColumn::WsPricingCouponAmt => 436,
            WebSalesGeneratorColumn::WsPricingExtShipCost => 437,
            WebSalesGeneratorColumn::WsPricingNetPaid => 438,
            WebSalesGeneratorColumn::WsPricingNetPaidIncTax => 439,
            WebSalesGeneratorColumn::WsPricingNetPaidIncShip => 440,
            WebSalesGeneratorColumn::WsPricingNetPaidIncShipTax => 441,
            WebSalesGeneratorColumn::WsPricingNetProfit => 442,
            WebSalesGeneratorColumn::WsPricing => 443,
            WebSalesGeneratorColumn::WsNulls => 444,
            WebSalesGeneratorColumn::WrIsReturned => 445,
            WebSalesGeneratorColumn::WsPermutation => 446,
        }
    }

    fn get_seeds_per_row(&self) -> u32 {
        match self {
            WebSalesGeneratorColumn::WsSoldDateSk => 2,
            WebSalesGeneratorColumn::WsSoldTimeSk => 2,
            WebSalesGeneratorColumn::WsShipDateSk => 16,
            WebSalesGeneratorColumn::WsItemSk => 1,
            WebSalesGeneratorColumn::WsBillCustomerSk => 1,
            WebSalesGeneratorColumn::WsBillCdemoSk => 1,
            WebSalesGeneratorColumn::WsBillHdemoSk => 1,
            WebSalesGeneratorColumn::WsBillAddrSk => 1,
            WebSalesGeneratorColumn::WsShipCustomerSk => 2,
            WebSalesGeneratorColumn::WsShipCdemoSk => 2,
            WebSalesGeneratorColumn::WsShipHdemoSk => 1,
            WebSalesGeneratorColumn::WsShipAddrSk => 1,
            WebSalesGeneratorColumn::WsWebPageSk => 16,
            WebSalesGeneratorColumn::WsWebSiteSk => 16,
            WebSalesGeneratorColumn::WsShipModeSk => 16,
            WebSalesGeneratorColumn::WsWarehouseSk => 16,
            WebSalesGeneratorColumn::WsPromoSk => 16,
            WebSalesGeneratorColumn::WsOrderNumber => 1,
            WebSalesGeneratorColumn::WsPricingQuantity => 1,
            WebSalesGeneratorColumn::WsPricingWholesaleCost => 1,
            WebSalesGeneratorColumn::WsPricingListPrice => 0,
            WebSalesGeneratorColumn::WsPricingSalesPrice => 0,
            WebSalesGeneratorColumn::WsPricingExtDiscountAmt => 0,
            WebSalesGeneratorColumn::WsPricingExtSalesPrice => 0,
            WebSalesGeneratorColumn::WsPricingExtWholesaleCost => 0,
            WebSalesGeneratorColumn::WsPricingExtListPrice => 0,
            WebSalesGeneratorColumn::WsPricingExtTax => 0,
            WebSalesGeneratorColumn::WsPricingCouponAmt => 0,
            WebSalesGeneratorColumn::WsPricingExtShipCost => 0,
            WebSalesGeneratorColumn::WsPricingNetPaid => 0,
            WebSalesGeneratorColumn::WsPricingNetPaidIncTax => 0,
            WebSalesGeneratorColumn::WsPricingNetPaidIncShip => 0,
            WebSalesGeneratorColumn::WsPricingNetPaidIncShipTax => 0,
            WebSalesGeneratorColumn::WsPricingNetProfit => 0,
            WebSalesGeneratorColumn::WsPricing => 128,
            WebSalesGeneratorColumn::WsNulls => 32,
            WebSalesGeneratorColumn::WrIsReturned => 16,
            WebSalesGeneratorColumn::WsPermutation => 0,
        }
    }
}

impl WebSalesGeneratorColumn {
    /// Returns all variants in order
    pub fn all_variants() -> &'static [WebSalesGeneratorColumn] {
        use WebSalesGeneratorColumn::*;
        &[
            WsSoldDateSk,
            WsSoldTimeSk,
            WsShipDateSk,
            WsItemSk,
            WsBillCustomerSk,
            WsBillCdemoSk,
            WsBillHdemoSk,
            WsBillAddrSk,
            WsShipCustomerSk,
            WsShipCdemoSk,
            WsShipHdemoSk,
            WsShipAddrSk,
            WsWebPageSk,
            WsWebSiteSk,
            WsShipModeSk,
            WsWarehouseSk,
            WsPromoSk,
            WsOrderNumber,
            WsPricingQuantity,
            WsPricingWholesaleCost,
            WsPricingListPrice,
            WsPricingSalesPrice,
            WsPricingExtDiscountAmt,
            WsPricingExtSalesPrice,
            WsPricingExtWholesaleCost,
            WsPricingExtListPrice,
            WsPricingExtTax,
            WsPricingCouponAmt,
            WsPricingExtShipCost,
            WsPricingNetPaid,
            WsPricingNetPaidIncTax,
            WsPricingNetPaidIncShip,
            WsPricingNetPaidIncShipTax,
            WsPricingNetProfit,
            WsPricing,
            WsNulls,
            WrIsReturned,
            WsPermutation,
        ]
    }
}
