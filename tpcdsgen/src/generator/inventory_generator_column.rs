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

//! Inventory generator column definitions

use crate::column::Table;
use crate::generator::GeneratorColumn;

/// Generator columns for the inventory table.
/// These map to InventoryGeneratorColumn.java
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InventoryGeneratorColumn {
    InvDateSk,
    InvItemSk,
    InvWarehouseSk,
    InvQuantityOnHand,
    InvNulls,
}

impl InventoryGeneratorColumn {
    /// Get all variants in order
    pub fn all_variants() -> &'static [InventoryGeneratorColumn] {
        use InventoryGeneratorColumn::*;
        static VARIANTS: [InventoryGeneratorColumn; 5] = [
            InvDateSk,
            InvItemSk,
            InvWarehouseSk,
            InvQuantityOnHand,
            InvNulls,
        ];
        &VARIANTS
    }
}

impl GeneratorColumn for InventoryGeneratorColumn {
    fn get_table(&self) -> Table {
        Table::Inventory
    }

    fn get_global_column_number(&self) -> i32 {
        match self {
            InventoryGeneratorColumn::InvDateSk => 198,
            InventoryGeneratorColumn::InvItemSk => 199,
            InventoryGeneratorColumn::InvWarehouseSk => 200,
            InventoryGeneratorColumn::InvQuantityOnHand => 201,
            InventoryGeneratorColumn::InvNulls => 202,
        }
    }

    fn get_seeds_per_row(&self) -> u32 {
        match self {
            InventoryGeneratorColumn::InvDateSk => 1,
            InventoryGeneratorColumn::InvItemSk => 1,
            InventoryGeneratorColumn::InvWarehouseSk => 1,
            InventoryGeneratorColumn::InvQuantityOnHand => 1,
            InventoryGeneratorColumn::InvNulls => 2,
        }
    }
}
