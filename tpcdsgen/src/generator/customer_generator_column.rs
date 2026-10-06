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

//! Customer generator column definitions (CustomerGeneratorColumn)

use crate::column::Table;
use crate::generator::GeneratorColumn;

/// Customer generator column enum (CustomerGeneratorColumn)
///
/// Each variant contains the global column number and seeds per row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CustomerGeneratorColumn {
    CCustomerSk,
    CCustomerId,
    CCurrentCdemoSk,
    CCurrentHdemoSk,
    CCurrentAddrSk,
    CFirstShiptoDateId,
    CFirstSalesDateId,
    CSalutation,
    CFirstName,
    CLastName,
    CPreferredCustFlag,
    CBirthDay,
    CBirthMonth,
    CBirthYear,
    CBirthCountry,
    CLogin,
    CEmailAddress,
    CLastReviewDate,
    CNulls,
}

impl CustomerGeneratorColumn {
    /// Get all column values in order
    pub fn values() -> &'static [CustomerGeneratorColumn] {
        use CustomerGeneratorColumn::*;
        static VALUES: &[CustomerGeneratorColumn] = &[
            CCustomerSk,
            CCustomerId,
            CCurrentCdemoSk,
            CCurrentHdemoSk,
            CCurrentAddrSk,
            CFirstShiptoDateId,
            CFirstSalesDateId,
            CSalutation,
            CFirstName,
            CLastName,
            CPreferredCustFlag,
            CBirthDay,
            CBirthMonth,
            CBirthYear,
            CBirthCountry,
            CLogin,
            CEmailAddress,
            CLastReviewDate,
            CNulls,
        ];
        VALUES
    }

    /// Get the global column number for this column
    fn global_column_number(&self) -> i32 {
        use CustomerGeneratorColumn::*;
        match self {
            CCustomerSk => 114,
            CCustomerId => 115,
            CCurrentCdemoSk => 116,
            CCurrentHdemoSk => 117,
            CCurrentAddrSk => 118,
            CFirstShiptoDateId => 119,
            CFirstSalesDateId => 120,
            CSalutation => 121,
            CFirstName => 122,
            CLastName => 123,
            CPreferredCustFlag => 124,
            CBirthDay => 125,
            CBirthMonth => 126,
            CBirthYear => 127,
            CBirthCountry => 128,
            CLogin => 129,
            CEmailAddress => 130,
            CLastReviewDate => 131,
            CNulls => 132,
        }
    }

    /// Get the seeds per row for this column
    fn seeds_per_row(&self) -> u32 {
        use CustomerGeneratorColumn::*;
        match self {
            CCustomerSk => 1,
            CCustomerId => 1,
            CCurrentCdemoSk => 1,
            CCurrentHdemoSk => 1,
            CCurrentAddrSk => 1,
            CFirstShiptoDateId => 0,
            CFirstSalesDateId => 1,
            CSalutation => 1,
            CFirstName => 1,
            CLastName => 1,
            CPreferredCustFlag => 2,
            CBirthDay => 1,
            CBirthMonth => 0,
            CBirthYear => 0,
            CBirthCountry => 1,
            CLogin => 1,
            CEmailAddress => 23,
            CLastReviewDate => 1,
            CNulls => 2,
        }
    }
}

impl GeneratorColumn for CustomerGeneratorColumn {
    fn get_table(&self) -> Table {
        Table::Customer
    }

    fn get_global_column_number(&self) -> i32 {
        self.global_column_number()
    }

    fn get_seeds_per_row(&self) -> u32 {
        self.seeds_per_row()
    }
}
