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

//! Item row generator (Slowly Changing Dimension)

use crate::config::Session;
use crate::distribution::{
    get_brand_syllables_distribution, get_category_at_index, get_has_size_at_index,
    pick_random_category_class, pick_random_category_index, pick_random_color,
    pick_random_current_price_range, pick_random_manager_id_range, pick_random_manufact_id_range,
    pick_random_size, pick_random_unit, ColorsWeights, IdWeights, SizeWeights,
};
use crate::error::Result;
use crate::generator::ItemGeneratorColumn;
use crate::join_key_utils::generate_join_key;
use crate::nulls::create_null_bit_map;
use crate::random::RandomValueGenerator;
use crate::row::item_row::ItemRow;
use crate::row::{AbstractRowGenerator, RowGenerator, RowGeneratorResult};
use crate::slowly_changing_dimension_utils::{
    compute_scd_key, get_value_for_slowly_changing_dimension,
};
use crate::table::Table;
use crate::types::Decimal;

fn min_item_markdown_pct() -> Decimal {
    Decimal::new(30, 2).unwrap()
}

fn max_item_markdown_pct() -> Decimal {
    Decimal::new(90, 2).unwrap()
}
const ROW_SIZE_I_PRODUCT_NAME: i32 = 50;
const ROW_SIZE_I_ITEM_DESC: i32 = 200;
const ROW_SIZE_I_MANUFACT: i32 = 50;
const ROW_SIZE_I_FORMULATION: i32 = 20;
#[allow(dead_code)]
const I_PROMO_PERCENTAGE: i32 = 20;

pub struct ItemRowGenerator {
    abstract_generator: AbstractRowGenerator,
    previous_row: Option<ItemRow>,
}

impl ItemRowGenerator {
    pub fn new() -> Self {
        ItemRowGenerator {
            abstract_generator: AbstractRowGenerator::new(Table::Item),
            previous_row: None,
        }
    }

    fn generate_item_row(&mut self, row_number: u64, session: &Session) -> Result<ItemRow> {
        use ItemGeneratorColumn::*;

        let row_number_i64 = i64::try_from(row_number).expect("row number fits in i64");

        // Generate null bit map first
        let stream = self.abstract_generator.get_random_number_stream(&INulls);
        let null_bit_map = create_null_bit_map(Table::Item, stream);

        let i_item_sk = row_number_i64;

        // Generate manager ID range
        let stream = self
            .abstract_generator
            .get_random_number_stream(&IManagerId);
        let (manager_min, manager_max) = pick_random_manager_id_range(IdWeights::Unified, stream)?;
        let stream = self
            .abstract_generator
            .get_random_number_stream(&IManagerId);
        let i_manager_id = RandomValueGenerator::generate_uniform_random_key(
            manager_min as i64,
            manager_max as i64,
            stream,
        );

        // Compute SCD key
        let scd_key = compute_scd_key(Table::Item, row_number);
        let i_item_id = scd_key.get_business_key().to_string();
        let i_rec_start_date_id = scd_key.get_start_date();
        let i_rec_end_date_id = scd_key.get_end_date();
        let is_new_business_key = scd_key.is_new_business_key();

        // Get field change flags
        let stream = self.abstract_generator.get_random_number_stream(&IScd);
        let mut field_change_flags = stream.next_random() as i32;

        // Generate item description
        let stream = self.abstract_generator.get_random_number_stream(&IItemDesc);
        let mut i_item_desc =
            RandomValueGenerator::generate_random_text(1, ROW_SIZE_I_ITEM_DESC, stream);
        if let Some(ref prev_row) = self.previous_row {
            i_item_desc = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                prev_row.get_i_item_desc().to_string(),
                i_item_desc,
            );
        }
        field_change_flags >>= 1;

        // Generate current price - There is a bug in C code that always chooses new record
        let stream = self
            .abstract_generator
            .get_random_number_stream(&ICurrentPrice);
        let (price_min, price_max) = pick_random_current_price_range(stream)?;
        let stream = self
            .abstract_generator
            .get_random_number_stream(&ICurrentPrice);
        let i_current_price =
            RandomValueGenerator::generate_uniform_random_decimal(price_min, price_max, stream);
        field_change_flags >>= 1;

        // Generate wholesale cost
        let stream = self
            .abstract_generator
            .get_random_number_stream(&IWholesaleCost);
        let markdown = RandomValueGenerator::generate_uniform_random_decimal(
            min_item_markdown_pct(),
            max_item_markdown_pct(),
            stream,
        );
        let mut i_wholesale_cost = Decimal::multiply(i_current_price, markdown);
        if let Some(ref prev_row) = self.previous_row {
            i_wholesale_cost = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                prev_row.get_i_wholesale_cost(),
                i_wholesale_cost,
            );
        }
        field_change_flags >>= 1;

        // Generate category
        let stream = self.abstract_generator.get_random_number_stream(&ICategory);
        let i_category_index = pick_random_category_index(stream)?;
        let i_category_id = (i_category_index + 1) as i64;
        let i_category = get_category_at_index(i_category_index).to_string();

        // Generate class
        let stream = self.abstract_generator.get_random_number_stream(&IClass);
        let category_class = pick_random_category_class(i_category_index, stream)?;
        let i_class = category_class.get_name().to_string();
        let new_class_id = category_class.get_id();
        let mut i_class_id = new_class_id;
        if let Some(ref prev_row) = self.previous_row {
            i_class_id = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                prev_row.get_i_class_id(),
                i_class_id,
            );
        }
        field_change_flags >>= 1;

        // Generate brand
        let brand_count = category_class.get_brand_count();
        let i_brand_id_base = row_number_i64 % brand_count as i64 + 1;
        let i_brand = format!(
            "{} #{}",
            RandomValueGenerator::generate_word(
                i_category_id * 10 + new_class_id,
                45,
                get_brand_syllables_distribution(),
            ),
            i_brand_id_base
        );
        let mut i_brand_id = i_brand_id_base + (i_category_id * 1000 + new_class_id) * 1000;
        if let Some(ref prev_row) = self.previous_row {
            i_brand_id = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                prev_row.get_i_brand_id(),
                i_brand_id,
            );
        }
        field_change_flags >>= 1;

        // Generate size - always uses new value due to bug in C code
        let has_size = get_has_size_at_index(i_category_index);
        let stream = self.abstract_generator.get_random_number_stream(&ISize);
        let i_size = pick_random_size(
            if has_size == 0 {
                SizeWeights::NoSize
            } else {
                SizeWeights::Sized
            },
            stream,
        )?;
        field_change_flags >>= 1;

        // Generate manufact ID
        let stream = self
            .abstract_generator
            .get_random_number_stream(&IManufactId);
        let (manufact_min, manufact_max) =
            pick_random_manufact_id_range(IdWeights::Unified, stream)?;
        let stream = self
            .abstract_generator
            .get_random_number_stream(&IManufactId);
        let mut i_manufact_id =
            RandomValueGenerator::generate_uniform_random_int(manufact_min, manufact_max, stream)
                as i64;
        if let Some(ref prev_row) = self.previous_row {
            i_manufact_id = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                prev_row.get_i_manufact_id(),
                i_manufact_id,
            );
        }
        field_change_flags >>= 1;

        // Generate manufact name
        let mut i_manufact = RandomValueGenerator::generate_word(
            i_manufact_id,
            ROW_SIZE_I_MANUFACT,
            crate::distribution::get_syllables_distribution(),
        );
        if let Some(ref prev_row) = self.previous_row {
            i_manufact = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                prev_row.get_i_manufact().to_string(),
                i_manufact,
            );
        }
        field_change_flags >>= 1;

        // Generate formulation
        let stream = self
            .abstract_generator
            .get_random_number_stream(&IFormulation);
        let mut i_formulation = RandomValueGenerator::generate_random_charset(
            RandomValueGenerator::DIGITS,
            ROW_SIZE_I_FORMULATION,
            ROW_SIZE_I_FORMULATION,
            stream,
        );
        let stream = self
            .abstract_generator
            .get_random_number_stream(&IFormulation);
        let color = pick_random_color(ColorsWeights::Skewed, stream)?;
        let stream = self
            .abstract_generator
            .get_random_number_stream(&IFormulation);
        let position = RandomValueGenerator::generate_uniform_random_int(
            0,
            i_formulation.len() as i32 - color.len() as i32 - 1,
            stream,
        ) as usize;
        // Replace part of formulation with color
        let mut formulation_chars: Vec<char> = i_formulation.chars().collect();
        for (i, c) in color.chars().enumerate() {
            if position + i < formulation_chars.len() {
                formulation_chars[position + i] = c;
            }
        }
        i_formulation = formulation_chars.into_iter().collect();
        if let Some(ref prev_row) = self.previous_row {
            i_formulation = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                prev_row.get_i_formulation().to_string(),
                i_formulation,
            );
        }

        // These fields always use new value due to bug in C code
        let stream = self.abstract_generator.get_random_number_stream(&IColor);
        let i_color = pick_random_color(ColorsWeights::Skewed, stream)?;

        let stream = self.abstract_generator.get_random_number_stream(&IUnits);
        let i_units = pick_random_unit(stream)?;

        let i_container = "Unknown".to_string();

        let i_product_name = RandomValueGenerator::generate_word(
            row_number_i64,
            ROW_SIZE_I_PRODUCT_NAME,
            crate::distribution::get_syllables_distribution(),
        );

        // Generate promo sk
        // Note: i_promo_sk is generated but not included in output (matches Java behavior)
        // We still need to generate it to consume the random seeds
        let stream = self.abstract_generator.get_random_number_stream(&IPromoSk);
        let _i_promo_sk = generate_join_key(
            &IPromoSk,
            stream,
            crate::config::Table::Promotion,
            1,
            session.get_scaling(),
        )?;
        let stream = self.abstract_generator.get_random_number_stream(&IPromoSk);
        let _temp = RandomValueGenerator::generate_uniform_random_int(1, 100, stream);

        let row = ItemRow::new(
            null_bit_map,
            i_item_sk,
            i_item_id,
            i_rec_start_date_id,
            i_rec_end_date_id,
            i_item_desc,
            i_current_price,
            i_wholesale_cost,
            i_brand_id,
            i_brand,
            i_class_id,
            i_class,
            i_category_id,
            i_category,
            i_manufact_id,
            i_manufact,
            i_size,
            i_formulation,
            i_color,
            i_units,
            i_container,
            i_manager_id,
            i_product_name,
        );

        Ok(row)
    }
}

impl Default for ItemRowGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl RowGenerator for ItemRowGenerator {
    fn generate_row_and_child_rows(
        &mut self,
        row_number: u64,
        session: &Session,
        _parent_row_generator: Option<&mut dyn RowGenerator>,
        _child_row_generator: Option<&mut dyn RowGenerator>,
    ) -> Result<RowGeneratorResult> {
        let row = self.generate_item_row(row_number, session)?;
        // Store for SCD logic on next row
        self.previous_row = Some(row.clone());
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
