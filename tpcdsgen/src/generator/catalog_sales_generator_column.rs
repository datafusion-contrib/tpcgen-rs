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

//! Catalog sales generator column definitions

use crate::column::Table;
use crate::generator::GeneratorColumn;

/// Enum representing all generator columns for the catalog_sales table
/// Global column numbers 75-113
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CatalogSalesGeneratorColumn {
    CsSoldDateSk,               // 75
    CsSoldTimeSk,               // 76
    CsShipDateSk,               // 77
    CsBillCustomerSk,           // 78
    CsBillCdemoSk,              // 79
    CsBillHdemoSk,              // 80
    CsBillAddrSk,               // 81
    CsShipCustomerSk,           // 82
    CsShipCdemoSk,              // 83
    CsShipHdemoSk,              // 84
    CsShipAddrSk,               // 85
    CsCallCenterSk,             // 86
    CsCatalogPageSk,            // 87
    CsShipModeSk,               // 88
    CsWarehouseSk,              // 89
    CsSoldItemSk,               // 90
    CsPromoSk,                  // 91
    CsOrderNumber,              // 92
    CsPricingQuantity,          // 93
    CsPricingWholesaleCost,     // 94
    CsPricingListPrice,         // 95
    CsPricingSalesPrice,        // 96
    CsPricingCouponAmt,         // 97
    CsPricingExtSalesPrice,     // 98
    CsPricingExtDiscountAmount, // 99
    CsPricingExtWholesaleCost,  // 100
    CsPricingExtListPrice,      // 101
    CsPricingExtTax,            // 102
    CsPricingExtShipCost,       // 103
    CsPricingNetPaid,           // 104
    CsPricingNetPaidIncTax,     // 105
    CsPricingNetPaidIncShip,    // 106
    CsPricingNetPaidIncShipTax, // 107
    CsPricingNetProfit,         // 108
    CsPricing,                  // 109
    CsPermute,                  // 110
    CsNulls,                    // 111
    CrIsReturned,               // 112
    CsPermutation,              // 113
}

impl GeneratorColumn for CatalogSalesGeneratorColumn {
    fn get_table(&self) -> Table {
        Table::CatalogSales
    }

    fn get_global_column_number(&self) -> i32 {
        match self {
            CatalogSalesGeneratorColumn::CsSoldDateSk => 75,
            CatalogSalesGeneratorColumn::CsSoldTimeSk => 76,
            CatalogSalesGeneratorColumn::CsShipDateSk => 77,
            CatalogSalesGeneratorColumn::CsBillCustomerSk => 78,
            CatalogSalesGeneratorColumn::CsBillCdemoSk => 79,
            CatalogSalesGeneratorColumn::CsBillHdemoSk => 80,
            CatalogSalesGeneratorColumn::CsBillAddrSk => 81,
            CatalogSalesGeneratorColumn::CsShipCustomerSk => 82,
            CatalogSalesGeneratorColumn::CsShipCdemoSk => 83,
            CatalogSalesGeneratorColumn::CsShipHdemoSk => 84,
            CatalogSalesGeneratorColumn::CsShipAddrSk => 85,
            CatalogSalesGeneratorColumn::CsCallCenterSk => 86,
            CatalogSalesGeneratorColumn::CsCatalogPageSk => 87,
            CatalogSalesGeneratorColumn::CsShipModeSk => 88,
            CatalogSalesGeneratorColumn::CsWarehouseSk => 89,
            CatalogSalesGeneratorColumn::CsSoldItemSk => 90,
            CatalogSalesGeneratorColumn::CsPromoSk => 91,
            CatalogSalesGeneratorColumn::CsOrderNumber => 92,
            CatalogSalesGeneratorColumn::CsPricingQuantity => 93,
            CatalogSalesGeneratorColumn::CsPricingWholesaleCost => 94,
            CatalogSalesGeneratorColumn::CsPricingListPrice => 95,
            CatalogSalesGeneratorColumn::CsPricingSalesPrice => 96,
            CatalogSalesGeneratorColumn::CsPricingCouponAmt => 97,
            CatalogSalesGeneratorColumn::CsPricingExtSalesPrice => 98,
            CatalogSalesGeneratorColumn::CsPricingExtDiscountAmount => 99,
            CatalogSalesGeneratorColumn::CsPricingExtWholesaleCost => 100,
            CatalogSalesGeneratorColumn::CsPricingExtListPrice => 101,
            CatalogSalesGeneratorColumn::CsPricingExtTax => 102,
            CatalogSalesGeneratorColumn::CsPricingExtShipCost => 103,
            CatalogSalesGeneratorColumn::CsPricingNetPaid => 104,
            CatalogSalesGeneratorColumn::CsPricingNetPaidIncTax => 105,
            CatalogSalesGeneratorColumn::CsPricingNetPaidIncShip => 106,
            CatalogSalesGeneratorColumn::CsPricingNetPaidIncShipTax => 107,
            CatalogSalesGeneratorColumn::CsPricingNetProfit => 108,
            CatalogSalesGeneratorColumn::CsPricing => 109,
            CatalogSalesGeneratorColumn::CsPermute => 110,
            CatalogSalesGeneratorColumn::CsNulls => 111,
            CatalogSalesGeneratorColumn::CrIsReturned => 112,
            CatalogSalesGeneratorColumn::CsPermutation => 113,
        }
    }

    fn get_seeds_per_row(&self) -> u32 {
        match self {
            CatalogSalesGeneratorColumn::CsSoldDateSk => 1,
            CatalogSalesGeneratorColumn::CsSoldTimeSk => 2,
            CatalogSalesGeneratorColumn::CsShipDateSk => 14,
            CatalogSalesGeneratorColumn::CsBillCustomerSk => 1,
            CatalogSalesGeneratorColumn::CsBillCdemoSk => 1,
            CatalogSalesGeneratorColumn::CsBillHdemoSk => 1,
            CatalogSalesGeneratorColumn::CsBillAddrSk => 1,
            CatalogSalesGeneratorColumn::CsShipCustomerSk => 2,
            CatalogSalesGeneratorColumn::CsShipCdemoSk => 1,
            CatalogSalesGeneratorColumn::CsShipHdemoSk => 1,
            CatalogSalesGeneratorColumn::CsShipAddrSk => 1,
            CatalogSalesGeneratorColumn::CsCallCenterSk => 1,
            CatalogSalesGeneratorColumn::CsCatalogPageSk => 42,
            CatalogSalesGeneratorColumn::CsShipModeSk => 14,
            CatalogSalesGeneratorColumn::CsWarehouseSk => 14,
            CatalogSalesGeneratorColumn::CsSoldItemSk => 1,
            CatalogSalesGeneratorColumn::CsPromoSk => 14,
            CatalogSalesGeneratorColumn::CsOrderNumber => 1,
            CatalogSalesGeneratorColumn::CsPricingQuantity => 0,
            CatalogSalesGeneratorColumn::CsPricingWholesaleCost => 0,
            CatalogSalesGeneratorColumn::CsPricingListPrice => 0,
            CatalogSalesGeneratorColumn::CsPricingSalesPrice => 0,
            CatalogSalesGeneratorColumn::CsPricingCouponAmt => 0,
            CatalogSalesGeneratorColumn::CsPricingExtSalesPrice => 0,
            CatalogSalesGeneratorColumn::CsPricingExtDiscountAmount => 0,
            CatalogSalesGeneratorColumn::CsPricingExtWholesaleCost => 0,
            CatalogSalesGeneratorColumn::CsPricingExtListPrice => 0,
            CatalogSalesGeneratorColumn::CsPricingExtTax => 0,
            CatalogSalesGeneratorColumn::CsPricingExtShipCost => 0,
            CatalogSalesGeneratorColumn::CsPricingNetPaid => 0,
            CatalogSalesGeneratorColumn::CsPricingNetPaidIncTax => 0,
            CatalogSalesGeneratorColumn::CsPricingNetPaidIncShip => 0,
            CatalogSalesGeneratorColumn::CsPricingNetPaidIncShipTax => 0,
            CatalogSalesGeneratorColumn::CsPricingNetProfit => 0,
            CatalogSalesGeneratorColumn::CsPricing => 112,
            CatalogSalesGeneratorColumn::CsPermute => 0,
            CatalogSalesGeneratorColumn::CsNulls => 28,
            CatalogSalesGeneratorColumn::CrIsReturned => 14,
            CatalogSalesGeneratorColumn::CsPermutation => 0,
        }
    }
}

impl CatalogSalesGeneratorColumn {
    /// Returns all variants in order
    pub fn all_variants() -> &'static [CatalogSalesGeneratorColumn] {
        use CatalogSalesGeneratorColumn::*;
        &[
            CsSoldDateSk,
            CsSoldTimeSk,
            CsShipDateSk,
            CsBillCustomerSk,
            CsBillCdemoSk,
            CsBillHdemoSk,
            CsBillAddrSk,
            CsShipCustomerSk,
            CsShipCdemoSk,
            CsShipHdemoSk,
            CsShipAddrSk,
            CsCallCenterSk,
            CsCatalogPageSk,
            CsShipModeSk,
            CsWarehouseSk,
            CsSoldItemSk,
            CsPromoSk,
            CsOrderNumber,
            CsPricingQuantity,
            CsPricingWholesaleCost,
            CsPricingListPrice,
            CsPricingSalesPrice,
            CsPricingCouponAmt,
            CsPricingExtSalesPrice,
            CsPricingExtDiscountAmount,
            CsPricingExtWholesaleCost,
            CsPricingExtListPrice,
            CsPricingExtTax,
            CsPricingExtShipCost,
            CsPricingNetPaid,
            CsPricingNetPaidIncTax,
            CsPricingNetPaidIncShip,
            CsPricingNetPaidIncShipTax,
            CsPricingNetProfit,
            CsPricing,
            CsPermute,
            CsNulls,
            CrIsReturned,
            CsPermutation,
        ]
    }
}
