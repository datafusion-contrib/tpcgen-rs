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

//! Inventory row data structure

use crate::generator::{GeneratorColumn, InventoryGeneratorColumn};
use crate::row::table_row::DatField;
use std::fmt;

/// Represents a single row in the inventory table.
#[derive(Clone)]
pub struct InventoryRow {
    null_bit_map: i64,
    pub(crate) inv_date_sk: i64,
    pub(crate) inv_item_sk: i64,
    pub(crate) inv_warehouse_sk: i64,
    pub(crate) inv_quantity_on_hand: i32,
}

impl InventoryRow {
    pub fn new(
        null_bit_map: i64,
        inv_date_sk: i64,
        inv_item_sk: i64,
        inv_warehouse_sk: i64,
        inv_quantity_on_hand: i32,
    ) -> Self {
        InventoryRow {
            null_bit_map,
            inv_date_sk,
            inv_item_sk,
            inv_warehouse_sk,
            inv_quantity_on_hand,
        }
    }

    pub(crate) fn is_null_at(&self, column: InventoryGeneratorColumn) -> bool {
        let bit_position = column.get_global_column_number()
            - InventoryGeneratorColumn::InvDateSk.get_global_column_number();
        (self.null_bit_map & (1 << bit_position)) != 0
    }

    pub fn null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    pub fn get_inv_date_sk(&self) -> i64 {
        self.inv_date_sk
    }

    pub fn get_inv_item_sk(&self) -> i64 {
        self.inv_item_sk
    }

    pub fn get_inv_warehouse_sk(&self) -> i64 {
        self.inv_warehouse_sk
    }

    pub fn get_inv_quantity_on_hand(&self) -> i32 {
        self.inv_quantity_on_hand
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces one
/// `|`-terminated field per column.
impl fmt::Display for InventoryRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use InventoryGeneratorColumn::*;

        write!(
            f,
            "{}|{}|{}|{}|",
            DatField::new(self.inv_date_sk, self.is_null_at(InvDateSk)),
            DatField::new(self.inv_item_sk, self.is_null_at(InvItemSk)),
            DatField::new(self.inv_warehouse_sk, self.is_null_at(InvWarehouseSk)),
            DatField::new(
                self.inv_quantity_on_hand,
                self.is_null_at(InvQuantityOnHand)
            ),
        )
    }
}
