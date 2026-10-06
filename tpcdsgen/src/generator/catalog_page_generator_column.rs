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

//! Catalog page generator column definitions

use crate::column::Table;
use crate::generator::GeneratorColumn;

/// Enum representing all generator columns for the catalog_page table
/// Global column numbers 35-45
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CatalogPageGeneratorColumn {
    CpCatalogPageSk,     // 35
    CpCatalogPageId,     // 36
    CpStartDateId,       // 37
    CpEndDateId,         // 38
    CpPromoId,           // 39 (unused)
    CpDepartment,        // 40
    CpCatalogNumber,     // 41
    CpCatalogPageNumber, // 42
    CpDescription,       // 43
    CpType,              // 44
    CpNulls,             // 45
}

impl GeneratorColumn for CatalogPageGeneratorColumn {
    fn get_table(&self) -> Table {
        Table::CatalogPage
    }

    fn get_global_column_number(&self) -> i32 {
        match self {
            CatalogPageGeneratorColumn::CpCatalogPageSk => 35,
            CatalogPageGeneratorColumn::CpCatalogPageId => 36,
            CatalogPageGeneratorColumn::CpStartDateId => 37,
            CatalogPageGeneratorColumn::CpEndDateId => 38,
            CatalogPageGeneratorColumn::CpPromoId => 39,
            CatalogPageGeneratorColumn::CpDepartment => 40,
            CatalogPageGeneratorColumn::CpCatalogNumber => 41,
            CatalogPageGeneratorColumn::CpCatalogPageNumber => 42,
            CatalogPageGeneratorColumn::CpDescription => 43,
            CatalogPageGeneratorColumn::CpType => 44,
            CatalogPageGeneratorColumn::CpNulls => 45,
        }
    }

    fn get_seeds_per_row(&self) -> u32 {
        match self {
            CatalogPageGeneratorColumn::CpDescription => 100, // S_CP_DESCRIPTION
            CatalogPageGeneratorColumn::CpNulls => 2,
            _ => 1,
        }
    }
}

impl CatalogPageGeneratorColumn {
    /// Returns all variants in order
    pub fn all_variants() -> &'static [CatalogPageGeneratorColumn] {
        use CatalogPageGeneratorColumn::*;
        &[
            CpCatalogPageSk,
            CpCatalogPageId,
            CpStartDateId,
            CpEndDateId,
            CpPromoId,
            CpDepartment,
            CpCatalogNumber,
            CpCatalogPageNumber,
            CpDescription,
            CpType,
            CpNulls,
        ]
    }
}
