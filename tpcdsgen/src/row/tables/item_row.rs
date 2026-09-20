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

//! Item row structure and formatting

use crate::generator::{GeneratorColumn, ItemGeneratorColumn};
use crate::row::table_row::DatField;
use crate::types::{Date, Decimal};
use std::fmt;

/// Item row
#[derive(Clone)]
pub struct ItemRow {
    null_bit_map: i64,
    pub(crate) i_item_sk: i64,
    pub(crate) i_item_id: String,
    pub(crate) i_rec_start_date_id: i64,
    pub(crate) i_rec_end_date_id: i64,
    pub(crate) i_item_desc: String,
    pub(crate) i_current_price: Decimal,
    pub(crate) i_wholesale_cost: Decimal,
    pub(crate) i_brand_id: i64,
    pub(crate) i_brand: String,
    pub(crate) i_class_id: i64,
    pub(crate) i_class: String,
    pub(crate) i_category_id: i64,
    pub(crate) i_category: String,
    pub(crate) i_manufact_id: i64,
    pub(crate) i_manufact: String,
    pub(crate) i_size: String,
    pub(crate) i_formulation: String,
    pub(crate) i_color: String,
    pub(crate) i_units: String,
    pub(crate) i_container: String,
    pub(crate) i_manager_id: i64,
    pub(crate) i_product_name: String,
}

impl ItemRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        null_bit_map: i64,
        i_item_sk: i64,
        i_item_id: String,
        i_rec_start_date_id: i64,
        i_rec_end_date_id: i64,
        i_item_desc: String,
        i_current_price: Decimal,
        i_wholesale_cost: Decimal,
        i_brand_id: i64,
        i_brand: String,
        i_class_id: i64,
        i_class: String,
        i_category_id: i64,
        i_category: String,
        i_manufact_id: i64,
        i_manufact: String,
        i_size: String,
        i_formulation: String,
        i_color: String,
        i_units: String,
        i_container: String,
        i_manager_id: i64,
        i_product_name: String,
    ) -> Self {
        ItemRow {
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
        }
    }

    pub(crate) fn is_null(&self, column: &ItemGeneratorColumn) -> bool {
        let bit_position = column.get_global_column_number()
            - ItemGeneratorColumn::IItemSk.get_global_column_number();
        (self.null_bit_map & (1 << bit_position)) != 0
    }

    // Getters for SCD fields
    pub fn get_i_item_desc(&self) -> &str {
        &self.i_item_desc
    }

    pub fn get_i_wholesale_cost(&self) -> Decimal {
        self.i_wholesale_cost
    }

    pub fn get_i_brand_id(&self) -> i64 {
        self.i_brand_id
    }

    pub fn get_i_class_id(&self) -> i64 {
        self.i_class_id
    }

    pub fn get_i_manufact_id(&self) -> i64 {
        self.i_manufact_id
    }

    pub fn get_i_manufact(&self) -> &str {
        &self.i_manufact
    }

    pub fn get_i_formulation(&self) -> &str {
        &self.i_formulation
    }

    pub fn null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    pub fn get_i_item_sk(&self) -> i64 {
        self.i_item_sk
    }

    pub fn get_i_item_id(&self) -> &str {
        &self.i_item_id
    }

    pub fn get_i_rec_start_date_id(&self) -> i64 {
        self.i_rec_start_date_id
    }

    pub fn get_i_rec_end_date_id(&self) -> i64 {
        self.i_rec_end_date_id
    }

    pub fn get_i_current_price(&self) -> Decimal {
        self.i_current_price
    }

    pub fn get_i_brand(&self) -> &str {
        &self.i_brand
    }

    pub fn get_i_class(&self) -> &str {
        &self.i_class
    }

    pub fn get_i_category_id(&self) -> i64 {
        self.i_category_id
    }

    pub fn get_i_category(&self) -> &str {
        &self.i_category
    }

    pub fn get_i_size(&self) -> &str {
        &self.i_size
    }

    pub fn get_i_color(&self) -> &str {
        &self.i_color
    }

    pub fn get_i_units(&self) -> &str {
        &self.i_units
    }

    pub fn get_i_container(&self) -> &str {
        &self.i_container
    }

    pub fn get_i_manager_id(&self) -> i64 {
        self.i_manager_id
    }

    pub fn get_i_product_name(&self) -> &str {
        &self.i_product_name
    }
}

impl ItemRow {
    /// DAT field for a regular value or key: NULL when the null bit is set
    /// (item keys have no sentinel check).
    pub(crate) fn field<T>(&self, value: T, column: &ItemGeneratorColumn) -> DatField<T> {
        DatField::new(value, self.is_null(column))
    }

    /// DAT field for an SCD date: NULL when the null bit is set or the
    /// julian day is negative; the Date is only constructed when it will be
    /// printed.
    pub(crate) fn date_field(
        &self,
        julian_days: i64,
        column: &ItemGeneratorColumn,
    ) -> DatField<Date> {
        DatField::from(
            (!(self.is_null(column) || julian_days < 0))
                .then(|| Date::from_julian_days(julian_days as i32)),
        )
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces one
/// `|`-terminated field per column.
impl fmt::Display for ItemRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ItemGeneratorColumn::*;

        write!(
            f,
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|",
            self.field(self.i_item_sk, &IItemSk),
            self.field(&self.i_item_id, &IItemId),
            self.date_field(self.i_rec_start_date_id, &IRecStartDateId),
            self.date_field(self.i_rec_end_date_id, &IRecEndDateId),
            self.field(&self.i_item_desc, &IItemDesc),
            self.field(self.i_current_price, &ICurrentPrice),
            self.field(self.i_wholesale_cost, &IWholesaleCost),
            self.field(self.i_brand_id, &IBrandId),
            self.field(&self.i_brand, &IBrand),
            self.field(self.i_class_id, &IClassId),
            self.field(&self.i_class, &IClass),
            self.field(self.i_category_id, &ICategoryId),
            self.field(&self.i_category, &ICategory),
            self.field(self.i_manufact_id, &IManufactId),
            self.field(&self.i_manufact, &IManufact),
            self.field(&self.i_size, &ISize),
            self.field(&self.i_formulation, &IFormulation),
            self.field(&self.i_color, &IColor),
            self.field(&self.i_units, &IUnits),
            self.field(&self.i_container, &IContainer),
            self.field(self.i_manager_id, &IManagerId),
            self.field(&self.i_product_name, &IProductName),
        )
    }
}
