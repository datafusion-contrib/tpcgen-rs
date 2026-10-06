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

//! Item generator column definitions

use crate::column::Table;
use crate::generator::GeneratorColumn;

/// Generator columns for item table
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemGeneratorColumn {
    IItemSk,
    IItemId,
    IRecStartDateId,
    IRecEndDateId,
    IItemDesc,
    ICurrentPrice,
    IWholesaleCost,
    IBrandId,
    IBrand,
    IClassId,
    IClass,
    ICategoryId,
    ICategory,
    IManufactId,
    IManufact,
    ISize,
    IFormulation,
    IColor,
    IUnits,
    IContainer,
    IManagerId,
    IProductName,
    INulls,
    IScd,
    IPromoSk,
}

impl GeneratorColumn for ItemGeneratorColumn {
    fn get_table(&self) -> Table {
        Table::Item
    }

    fn get_global_column_number(&self) -> i32 {
        use ItemGeneratorColumn::*;
        match self {
            IItemSk => 203,
            IItemId => 204,
            IRecStartDateId => 205,
            IRecEndDateId => 206,
            IItemDesc => 207,
            ICurrentPrice => 208,
            IWholesaleCost => 209,
            IBrandId => 210,
            IBrand => 211,
            IClassId => 212,
            IClass => 213,
            ICategoryId => 214,
            ICategory => 215,
            IManufactId => 216,
            IManufact => 217,
            ISize => 218,
            IFormulation => 219,
            IColor => 220,
            IUnits => 221,
            IContainer => 222,
            IManagerId => 223,
            IProductName => 224,
            INulls => 225,
            IScd => 226,
            IPromoSk => 227,
        }
    }

    fn get_seeds_per_row(&self) -> u32 {
        use ItemGeneratorColumn::*;
        match self {
            IItemSk => 1,
            IItemId => 1,
            IRecStartDateId => 1,
            IRecEndDateId => 2,
            IItemDesc => 200,
            ICurrentPrice => 2,
            IWholesaleCost => 1,
            IBrandId => 1,
            IBrand => 1,
            IClassId => 1,
            IClass => 1,
            ICategoryId => 1,
            ICategory => 1,
            IManufactId => 2,
            IManufact => 1,
            ISize => 1,
            IFormulation => 50,
            IColor => 1,
            IUnits => 1,
            IContainer => 1,
            IManagerId => 2,
            IProductName => 1,
            INulls => 2,
            IScd => 1,
            IPromoSk => 2,
        }
    }
}

impl ItemGeneratorColumn {
    /// Get all generator columns in order
    pub fn all_columns() -> &'static [ItemGeneratorColumn] {
        use ItemGeneratorColumn::*;
        &[
            IItemSk,
            IItemId,
            IRecStartDateId,
            IRecEndDateId,
            IItemDesc,
            ICurrentPrice,
            IWholesaleCost,
            IBrandId,
            IBrand,
            IClassId,
            IClass,
            ICategoryId,
            ICategory,
            IManufactId,
            IManufact,
            ISize,
            IFormulation,
            IColor,
            IUnits,
            IContainer,
            IManagerId,
            IProductName,
            INulls,
            IScd,
            IPromoSk,
        ]
    }
}
