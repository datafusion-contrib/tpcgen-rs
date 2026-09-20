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

//! Catalog page row structure and formatting

use crate::generator::{CatalogPageGeneratorColumn, GeneratorColumn};
use crate::row::table_row::DatField;
use std::fmt;

/// Catalog page row
#[derive(Clone)]
pub struct CatalogPageRow {
    pub(crate) null_bit_map: i64,
    pub(crate) cp_catalog_page_sk: i64,
    pub(crate) cp_catalog_page_id: String,
    pub(crate) cp_start_date_id: i64,
    pub(crate) cp_end_date_id: i64,
    pub(crate) cp_department: String,
    pub(crate) cp_catalog_number: i32,
    pub(crate) cp_catalog_page_number: i32,
    pub(crate) cp_description: String,
    pub(crate) cp_type: String,
}

impl CatalogPageRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        null_bit_map: i64,
        cp_catalog_page_sk: i64,
        cp_catalog_page_id: String,
        cp_start_date_id: i64,
        cp_end_date_id: i64,
        cp_department: String,
        cp_catalog_number: i32,
        cp_catalog_page_number: i32,
        cp_description: String,
        cp_type: String,
    ) -> Self {
        CatalogPageRow {
            null_bit_map,
            cp_catalog_page_sk,
            cp_catalog_page_id,
            cp_start_date_id,
            cp_end_date_id,
            cp_department,
            cp_catalog_number,
            cp_catalog_page_number,
            cp_description,
            cp_type,
        }
    }

    pub(crate) fn is_null(&self, column: &CatalogPageGeneratorColumn) -> bool {
        let bit_position = column.get_global_column_number()
            - CatalogPageGeneratorColumn::CpCatalogPageSk.get_global_column_number();
        (self.null_bit_map & (1 << bit_position)) != 0
    }

    pub fn null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    pub fn get_cp_catalog_page_sk(&self) -> i64 {
        self.cp_catalog_page_sk
    }

    pub fn get_cp_catalog_page_id(&self) -> &str {
        &self.cp_catalog_page_id
    }

    pub fn get_cp_start_date_id(&self) -> i64 {
        self.cp_start_date_id
    }

    pub fn get_cp_end_date_id(&self) -> i64 {
        self.cp_end_date_id
    }

    pub fn get_cp_department(&self) -> &str {
        &self.cp_department
    }

    pub fn get_cp_catalog_number(&self) -> i32 {
        self.cp_catalog_number
    }

    pub fn get_cp_catalog_page_number(&self) -> i32 {
        self.cp_catalog_page_number
    }

    pub fn get_cp_description(&self) -> &str {
        &self.cp_description
    }

    pub fn get_cp_type(&self) -> &str {
        &self.cp_type
    }
}

impl CatalogPageRow {
    /// DAT field for a surrogate key: NULL when the null bit is set or the
    /// key is negative.
    pub(crate) fn key_field(
        &self,
        value: i64,
        column: &CatalogPageGeneratorColumn,
    ) -> DatField<i64> {
        DatField::new(value, self.is_null(column) || value < 0)
    }

    /// DAT field for a regular value: NULL when the null bit is set.
    pub(crate) fn field<T>(&self, value: T, column: &CatalogPageGeneratorColumn) -> DatField<T> {
        DatField::new(value, self.is_null(column))
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces one
/// `|`-terminated field per column.
impl fmt::Display for CatalogPageRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CatalogPageGeneratorColumn::*;

        write!(
            f,
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|",
            self.key_field(self.cp_catalog_page_sk, &CpCatalogPageSk),
            self.field(&self.cp_catalog_page_id, &CpCatalogPageId),
            self.key_field(self.cp_start_date_id, &CpStartDateId),
            self.key_field(self.cp_end_date_id, &CpEndDateId),
            self.field(&self.cp_department, &CpDepartment),
            self.field(self.cp_catalog_number, &CpCatalogNumber),
            self.field(self.cp_catalog_page_number, &CpCatalogPageNumber),
            self.field(&self.cp_description, &CpDescription),
            self.field(&self.cp_type, &CpType),
        )
    }
}
