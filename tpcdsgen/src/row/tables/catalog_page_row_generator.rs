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

//! Catalog page row generator

use crate::config::Session;
use crate::error::Result;
use crate::generator::CatalogPageGeneratorColumn;
use crate::nulls::create_null_bit_map;
use crate::random::RandomValueGenerator;
use crate::row::catalog_page_row::CatalogPageRow;
use crate::row::AbstractRowGenerator;
use crate::table::Table;
use crate::types::Date;

const CATALOGS_PER_YEAR: i32 = 18;
const WIDTH_CP_DESCRIPTION: i32 = 100;

pub struct CatalogPageRowGenerator {
    abstract_generator: AbstractRowGenerator,
    session: Session,
}

impl CatalogPageRowGenerator {
    /// Generate source rows `1..=row_count`.
    pub fn new(session: Session, row_count: u64) -> Self {
        CatalogPageRowGenerator {
            abstract_generator: AbstractRowGenerator::new(Table::CatalogPage, row_count),
            session,
        }
    }

    fn generate_catalog_page_row(&mut self, row_number: u64) -> Result<CatalogPageRow> {
        use CatalogPageGeneratorColumn::*;

        let session = &self.session;
        let row_number_i64 = i64::try_from(row_number).expect("row number fits in i64");

        let cp_catalog_page_sk = row_number_i64;
        let cp_department = "DEPARTMENT".to_string();

        // Generate null bit map
        let stream = self.abstract_generator.get_random_number_stream(&CpNulls);
        let null_bit_map = create_null_bit_map(Table::CatalogPage, stream);

        // Generate business key
        let cp_catalog_page_id = crate::business_key_generator::make_business_key(row_number);

        // Calculate catalog page numbers
        let row_count = session
            .get_scaling()
            .get_row_count(crate::config::table::Table::CatalogPage);
        let catalog_page_max = ((row_count / CATALOGS_PER_YEAR as u64) as i32)
            / (Date::DATE_MAXIMUM.year() - Date::DATE_MINIMUM.year() + 2);
        let cp_catalog_number = ((row_number_i64 - 1) / catalog_page_max as i64 + 1) as i32;
        let cp_catalog_page_number = ((row_number_i64 - 1) % catalog_page_max as i64 + 1) as i32;

        // Calculate catalog interval and type
        let catalog_interval = (cp_catalog_number - 1) % CATALOGS_PER_YEAR;
        let (cp_type, duration, offset) = match catalog_interval {
            0 | 1 => {
                // bi-annual
                let duration = 182;
                let offset = catalog_interval * duration;
                ("bi-annual", duration, offset)
            }
            2..=5 => {
                // quarterly (Q1-Q4)
                let duration = 91;
                let offset = (catalog_interval - 2) * duration;
                ("quarterly", duration, offset)
            }
            _ => {
                // monthly
                let duration = 30;
                let offset = (catalog_interval - 6) * duration;
                ("monthly", duration, offset)
            }
        };

        // Calculate start and end dates
        let cp_start_date_id = Date::JULIAN_DATA_START_DATE
            + offset as i64
            + ((cp_catalog_number - 1) / CATALOGS_PER_YEAR) as i64 * 365;
        let cp_end_date_id = cp_start_date_id + duration as i64 - 1;

        // Generate description
        let stream = self
            .abstract_generator
            .get_random_number_stream(&CpDescription);
        let cp_description = RandomValueGenerator::generate_random_text(
            WIDTH_CP_DESCRIPTION / 2,
            WIDTH_CP_DESCRIPTION - 1,
            stream,
        );

        let row = CatalogPageRow::new(
            null_bit_map,
            cp_catalog_page_sk,
            cp_catalog_page_id,
            cp_start_date_id,
            cp_end_date_id,
            cp_department,
            cp_catalog_number,
            cp_catalog_page_number,
            cp_description,
            cp_type.to_string(),
        );

        Ok(row)
    }

    /// Start generating at `starting_row_number` (1-based), fast forwarding
    /// the random number streams to that row.
    pub fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64) {
        self.abstract_generator
            .skip_rows_until_starting_row_number(starting_row_number);
    }

    /// Restrict generation to source rows
    /// `starting_row_number..=ending_row_number` (1-based, inclusive).
    ///
    /// The ending row number is clamped to the table's row count.
    pub fn set_source_row_range(&mut self, starting_row_number: u64, ending_row_number: u64) {
        self.abstract_generator
            .set_source_row_range(starting_row_number, ending_row_number);
    }
}

impl Iterator for CatalogPageRowGenerator {
    type Item = CatalogPageRow;

    fn next(&mut self) -> Option<CatalogPageRow> {
        let row_number = self.abstract_generator.next_row_number()?;
        let row = self.generate_catalog_page_row(row_number).expect("row gen");
        self.abstract_generator.finish_row();
        Some(row)
    }
}
