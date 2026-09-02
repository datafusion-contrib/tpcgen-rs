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

//! Inventory row generator
//!
//! The inventory table represents weekly snapshots of item inventory levels
//! at each warehouse. It's a cross-join of items x warehouses x weeks.

use crate::config::Session;
use crate::error::Result;
use crate::generator::InventoryGeneratorColumn;
use crate::nulls::create_null_bit_map;
use crate::random::RandomValueGenerator;
use crate::row::inventory_row::InventoryRow;
use crate::row::{AbstractRowGenerator, RowGenerator, RowGeneratorResult};
use crate::slowly_changing_dimension_utils::match_surrogate_key;
use crate::table::Table;
use crate::types::Date;

pub struct InventoryRowGenerator {
    abstract_generator: AbstractRowGenerator,
}

impl InventoryRowGenerator {
    pub fn new() -> Self {
        InventoryRowGenerator {
            abstract_generator: AbstractRowGenerator::new(Table::Inventory),
        }
    }
}

impl Default for InventoryRowGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl RowGenerator for InventoryRowGenerator {
    fn generate_row_and_child_rows(
        &mut self,
        row_number: u64,
        session: &Session,
        _parent_row_generator: Option<&mut dyn RowGenerator>,
        _child_row_generator: Option<&mut dyn RowGenerator>,
    ) -> Result<RowGeneratorResult> {
        use InventoryGeneratorColumn::*;

        let scaling = session.get_scaling();

        // Generate null bit map
        let stream = self.abstract_generator.get_random_number_stream(&InvNulls);
        let null_bit_map = create_null_bit_map(Table::Inventory, stream);

        // Decode the row number into item, warehouse, and date indices
        // This is a cross-join: item x warehouse x date
        let mut index = row_number - 1;

        // Get item count (unique item IDs, not row count since Item keeps history)
        let item_count = scaling.get_id_count(crate::config::Table::Item);

        // Item cycles fastest
        let inv_item_sk_unique = (index % item_count) + 1;
        index /= item_count;

        // Get warehouse count
        let warehouse_count = scaling.get_row_count(crate::config::Table::Warehouse);

        // Warehouse cycles next
        let inv_warehouse_sk = (index % warehouse_count) + 1;
        index /= warehouse_count;

        // Date cycles slowest - inventory is updated weekly
        let inv_date_sk = Date::JULIAN_DATE_MINIMUM as i64
            + i64::try_from(index * 7).expect("inventory date offset fits in i64");

        // The join between item and inventory is tricky. The item_id selected above identifies
        // a unique part num but item is a slowly changing dimension, so we need to account for
        // that in selecting the surrogate key to join with
        let inv_item_sk = match_surrogate_key(
            i64::try_from(inv_item_sk_unique).expect("inventory item key fits in i64"),
            inv_date_sk,
            crate::config::Table::Item,
            scaling,
        );

        // Generate random quantity on hand (0-1000)
        let stream = self
            .abstract_generator
            .get_random_number_stream(&InvQuantityOnHand);
        let inv_quantity_on_hand =
            RandomValueGenerator::generate_uniform_random_int(0, 1000, stream);

        let row = InventoryRow::new(
            null_bit_map,
            inv_date_sk,
            inv_item_sk,
            i64::try_from(inv_warehouse_sk).expect("warehouse key fits in i64"),
            inv_quantity_on_hand,
        );

        Ok(RowGeneratorResult::new(row))
    }

    fn consume_remaining_seeds_for_row(&mut self) {
        self.abstract_generator.consume_remaining_seeds_for_row();
    }

    fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64) {
        self.abstract_generator
            .skip_rows_until_starting_row_number(starting_row_number);
    }
}
