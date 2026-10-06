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

//! Store generator column definitions

use crate::column::Table;
use crate::generator::GeneratorColumn;

/// Generator columns for store table
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StoreGeneratorColumn {
    WStoreSk,
    WStoreId,
    WStoreRecStartDateId,
    WStoreRecEndDateId,
    WStoreClosedDateId,
    WStoreName,
    WStoreEmployees,
    WStoreFloorSpace,
    WStoreHours,
    WStoreManager,
    WStoreMarketId,
    WStoreTaxPercentage,
    WStoreGeographyClass,
    WStoreMarketDesc,
    WStoreMarketManager,
    WStoreDivisionId,
    WStoreDivisionName,
    WStoreCompanyId,
    WStoreCompanyName,
    WStoreAddressStreetNum,
    WStoreAddressStreetName1,
    WStoreAddressStreetType,
    WStoreAddressSuiteNum,
    WStoreAddressCity,
    WStoreAddressCounty,
    WStoreAddressState,
    WStoreAddressZip,
    WStoreAddressCountry,
    WStoreAddressGmtOffset,
    WStoreNulls,
    WStoreType,
    WStoreScd,
    WStoreAddress,
}

impl GeneratorColumn for StoreGeneratorColumn {
    fn get_table(&self) -> Table {
        Table::Store
    }

    fn get_global_column_number(&self) -> i32 {
        use StoreGeneratorColumn::*;
        match self {
            WStoreSk => 259,
            WStoreId => 260,
            WStoreRecStartDateId => 261,
            WStoreRecEndDateId => 262,
            WStoreClosedDateId => 263,
            WStoreName => 264,
            WStoreEmployees => 265,
            WStoreFloorSpace => 266,
            WStoreHours => 267,
            WStoreManager => 268,
            WStoreMarketId => 269,
            WStoreTaxPercentage => 270,
            WStoreGeographyClass => 271,
            WStoreMarketDesc => 272,
            WStoreMarketManager => 273,
            WStoreDivisionId => 274,
            WStoreDivisionName => 275,
            WStoreCompanyId => 276,
            WStoreCompanyName => 277,
            WStoreAddressStreetNum => 278,
            WStoreAddressStreetName1 => 279,
            WStoreAddressStreetType => 280,
            WStoreAddressSuiteNum => 281,
            WStoreAddressCity => 282,
            WStoreAddressCounty => 283,
            WStoreAddressState => 284,
            WStoreAddressZip => 285,
            WStoreAddressCountry => 286,
            WStoreAddressGmtOffset => 287,
            WStoreNulls => 288,
            WStoreType => 289,
            WStoreScd => 290,
            WStoreAddress => 291,
        }
    }

    fn get_seeds_per_row(&self) -> u32 {
        use StoreGeneratorColumn::*;
        match self {
            WStoreSk => 1,
            WStoreId => 1,
            WStoreRecStartDateId => 1,
            WStoreRecEndDateId => 2,
            WStoreClosedDateId => 2,
            WStoreName => 0,
            WStoreEmployees => 1,
            WStoreFloorSpace => 1,
            WStoreHours => 1,
            WStoreManager => 2,
            WStoreMarketId => 1,
            WStoreTaxPercentage => 1,
            WStoreGeographyClass => 1,
            WStoreMarketDesc => 100,
            WStoreMarketManager => 2,
            WStoreDivisionId => 1,
            WStoreDivisionName => 1,
            WStoreCompanyId => 1,
            WStoreCompanyName => 1,
            WStoreAddressStreetNum => 1,
            WStoreAddressStreetName1 => 1,
            WStoreAddressStreetType => 1,
            WStoreAddressSuiteNum => 1,
            WStoreAddressCity => 1,
            WStoreAddressCounty => 1,
            WStoreAddressState => 1,
            WStoreAddressZip => 1,
            WStoreAddressCountry => 1,
            WStoreAddressGmtOffset => 1,
            WStoreNulls => 2,
            WStoreType => 1,
            WStoreScd => 1,
            WStoreAddress => 7,
        }
    }
}

impl StoreGeneratorColumn {
    /// Get all generator columns in order
    pub fn all_columns() -> &'static [StoreGeneratorColumn] {
        use StoreGeneratorColumn::*;
        &[
            WStoreSk,
            WStoreId,
            WStoreRecStartDateId,
            WStoreRecEndDateId,
            WStoreClosedDateId,
            WStoreName,
            WStoreEmployees,
            WStoreFloorSpace,
            WStoreHours,
            WStoreManager,
            WStoreMarketId,
            WStoreTaxPercentage,
            WStoreGeographyClass,
            WStoreMarketDesc,
            WStoreMarketManager,
            WStoreDivisionId,
            WStoreDivisionName,
            WStoreCompanyId,
            WStoreCompanyName,
            WStoreAddressStreetNum,
            WStoreAddressStreetName1,
            WStoreAddressStreetType,
            WStoreAddressSuiteNum,
            WStoreAddressCity,
            WStoreAddressCounty,
            WStoreAddressState,
            WStoreAddressZip,
            WStoreAddressCountry,
            WStoreAddressGmtOffset,
            WStoreNulls,
            WStoreType,
            WStoreScd,
            WStoreAddress,
        ]
    }
}
