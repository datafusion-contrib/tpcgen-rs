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

//! Catalog returns generator column definitions

use crate::column::Table;
use crate::generator::GeneratorColumn;

/// Enum representing all generator columns for the catalog_returns table
/// Global column numbers 46-74
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CatalogReturnsGeneratorColumn {
    CrReturnedDateSk,        // 46
    CrReturnedTimeSk,        // 47
    CrItemSk,                // 48
    CrRefundedCustomerSk,    // 49
    CrRefundedCdemoSk,       // 50
    CrRefundedHdemoSk,       // 51
    CrRefundedAddrSk,        // 52
    CrReturningCustomerSk,   // 53
    CrReturningCdemoSk,      // 54
    CrReturningHdemoSk,      // 55
    CrReturningAddrSk,       // 56
    CrCallCenterSk,          // 57
    CrCatalogPageSk,         // 58
    CrShipModeSk,            // 59
    CrWarehouseSk,           // 60
    CrReasonSk,              // 61
    CrOrderNumber,           // 62
    CrPricingQuantity,       // 63
    CrPricingNetPaid,        // 64
    CrPricingExtTax,         // 65
    CrPricingNetPaidIncTax,  // 66
    CrPricingFee,            // 67
    CrPricingExtShipCost,    // 68
    CrPricingRefundedCash,   // 69
    CrPricingReversedCharge, // 70
    CrPricingStoreCredit,    // 71
    CrPricingNetLoss,        // 72
    CrNulls,                 // 73
    CrPricing,               // 74
}

impl GeneratorColumn for CatalogReturnsGeneratorColumn {
    fn get_table(&self) -> Table {
        Table::CatalogReturns
    }

    fn get_global_column_number(&self) -> i32 {
        match self {
            CatalogReturnsGeneratorColumn::CrReturnedDateSk => 46,
            CatalogReturnsGeneratorColumn::CrReturnedTimeSk => 47,
            CatalogReturnsGeneratorColumn::CrItemSk => 48,
            CatalogReturnsGeneratorColumn::CrRefundedCustomerSk => 49,
            CatalogReturnsGeneratorColumn::CrRefundedCdemoSk => 50,
            CatalogReturnsGeneratorColumn::CrRefundedHdemoSk => 51,
            CatalogReturnsGeneratorColumn::CrRefundedAddrSk => 52,
            CatalogReturnsGeneratorColumn::CrReturningCustomerSk => 53,
            CatalogReturnsGeneratorColumn::CrReturningCdemoSk => 54,
            CatalogReturnsGeneratorColumn::CrReturningHdemoSk => 55,
            CatalogReturnsGeneratorColumn::CrReturningAddrSk => 56,
            CatalogReturnsGeneratorColumn::CrCallCenterSk => 57,
            CatalogReturnsGeneratorColumn::CrCatalogPageSk => 58,
            CatalogReturnsGeneratorColumn::CrShipModeSk => 59,
            CatalogReturnsGeneratorColumn::CrWarehouseSk => 60,
            CatalogReturnsGeneratorColumn::CrReasonSk => 61,
            CatalogReturnsGeneratorColumn::CrOrderNumber => 62,
            CatalogReturnsGeneratorColumn::CrPricingQuantity => 63,
            CatalogReturnsGeneratorColumn::CrPricingNetPaid => 64,
            CatalogReturnsGeneratorColumn::CrPricingExtTax => 65,
            CatalogReturnsGeneratorColumn::CrPricingNetPaidIncTax => 66,
            CatalogReturnsGeneratorColumn::CrPricingFee => 67,
            CatalogReturnsGeneratorColumn::CrPricingExtShipCost => 68,
            CatalogReturnsGeneratorColumn::CrPricingRefundedCash => 69,
            CatalogReturnsGeneratorColumn::CrPricingReversedCharge => 70,
            CatalogReturnsGeneratorColumn::CrPricingStoreCredit => 71,
            CatalogReturnsGeneratorColumn::CrPricingNetLoss => 72,
            CatalogReturnsGeneratorColumn::CrNulls => 73,
            CatalogReturnsGeneratorColumn::CrPricing => 74,
        }
    }

    fn get_seeds_per_row(&self) -> u32 {
        match self {
            CatalogReturnsGeneratorColumn::CrReturnedDateSk => 28,
            CatalogReturnsGeneratorColumn::CrReturnedTimeSk => 28,
            CatalogReturnsGeneratorColumn::CrItemSk => 14,
            CatalogReturnsGeneratorColumn::CrRefundedCustomerSk => 14,
            CatalogReturnsGeneratorColumn::CrRefundedCdemoSk => 14,
            CatalogReturnsGeneratorColumn::CrRefundedHdemoSk => 14,
            CatalogReturnsGeneratorColumn::CrRefundedAddrSk => 14,
            CatalogReturnsGeneratorColumn::CrReturningCustomerSk => 28,
            CatalogReturnsGeneratorColumn::CrReturningCdemoSk => 14,
            CatalogReturnsGeneratorColumn::CrReturningHdemoSk => 14,
            CatalogReturnsGeneratorColumn::CrReturningAddrSk => 14,
            CatalogReturnsGeneratorColumn::CrCallCenterSk => 0,
            CatalogReturnsGeneratorColumn::CrCatalogPageSk => 14,
            CatalogReturnsGeneratorColumn::CrShipModeSk => 14,
            CatalogReturnsGeneratorColumn::CrWarehouseSk => 14,
            CatalogReturnsGeneratorColumn::CrReasonSk => 14,
            CatalogReturnsGeneratorColumn::CrOrderNumber => 0,
            CatalogReturnsGeneratorColumn::CrPricingQuantity => 0,
            CatalogReturnsGeneratorColumn::CrPricingNetPaid => 0,
            CatalogReturnsGeneratorColumn::CrPricingExtTax => 0,
            CatalogReturnsGeneratorColumn::CrPricingNetPaidIncTax => 0,
            CatalogReturnsGeneratorColumn::CrPricingFee => 0,
            CatalogReturnsGeneratorColumn::CrPricingExtShipCost => 0,
            CatalogReturnsGeneratorColumn::CrPricingRefundedCash => 0,
            CatalogReturnsGeneratorColumn::CrPricingReversedCharge => 0,
            CatalogReturnsGeneratorColumn::CrPricingStoreCredit => 0,
            CatalogReturnsGeneratorColumn::CrPricingNetLoss => 0,
            CatalogReturnsGeneratorColumn::CrNulls => 28,
            CatalogReturnsGeneratorColumn::CrPricing => 70,
        }
    }
}

impl CatalogReturnsGeneratorColumn {
    /// Returns all variants in order
    pub fn all_variants() -> &'static [CatalogReturnsGeneratorColumn] {
        use CatalogReturnsGeneratorColumn::*;
        &[
            CrReturnedDateSk,
            CrReturnedTimeSk,
            CrItemSk,
            CrRefundedCustomerSk,
            CrRefundedCdemoSk,
            CrRefundedHdemoSk,
            CrRefundedAddrSk,
            CrReturningCustomerSk,
            CrReturningCdemoSk,
            CrReturningHdemoSk,
            CrReturningAddrSk,
            CrCallCenterSk,
            CrCatalogPageSk,
            CrShipModeSk,
            CrWarehouseSk,
            CrReasonSk,
            CrOrderNumber,
            CrPricingQuantity,
            CrPricingNetPaid,
            CrPricingExtTax,
            CrPricingNetPaidIncTax,
            CrPricingFee,
            CrPricingExtShipCost,
            CrPricingRefundedCash,
            CrPricingReversedCharge,
            CrPricingStoreCredit,
            CrPricingNetLoss,
            CrNulls,
            CrPricing,
        ]
    }
}
