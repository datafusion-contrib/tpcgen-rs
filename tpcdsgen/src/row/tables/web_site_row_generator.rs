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

use crate::config::Table as ConfigTable;
use crate::distribution::{FirstNamesWeights, NamesDistributions};
use crate::generator::WebSiteGeneratorColumn;
use crate::join_key_utils::generate_join_key;
use crate::nulls::create_null_bit_map;
use crate::random::RandomValueGenerator;
use crate::row::{AbstractRowGenerator, RowGenerator, RowGeneratorResult, WebSiteRow};
use crate::slowly_changing_dimension_utils::{
    compute_scd_key, get_value_for_slowly_changing_dimension,
};
use crate::table::Table;
use crate::types::{Address, Decimal};

pub struct WebSiteRowGenerator {
    abstract_generator: AbstractRowGenerator,
    previous_row: Option<WebSiteRow>,
}

impl Default for WebSiteRowGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl WebSiteRowGenerator {
    pub fn new() -> Self {
        Self {
            abstract_generator: AbstractRowGenerator::new(Table::WebSite),
            previous_row: None,
        }
    }
}

impl RowGenerator for WebSiteRowGenerator {
    fn generate_row_and_child_rows(
        &mut self,
        row_number: u64,
        session: &crate::config::Session,
        _parent_row_generator: Option<&mut dyn RowGenerator>,
        _child_row_generator: Option<&mut dyn RowGenerator>,
    ) -> crate::error::Result<RowGeneratorResult> {
        let row_number_i64 = i64::try_from(row_number).expect("row number fits in i64");

        let scaling = session.get_scaling();

        let null_bit_map = create_null_bit_map(
            Table::WebSite,
            self.abstract_generator
                .get_random_number_stream(&WebSiteGeneratorColumn::WebNulls),
        );

        let web_site_sk = row_number_i64;
        let web_class = "Unknown".to_string();

        let scd_key = compute_scd_key(Table::WebSite, row_number);
        let web_site_id = scd_key.get_business_key().to_string();
        let web_rec_start_date_id = scd_key.get_start_date();
        let web_rec_end_date_id = scd_key.get_end_date();
        let is_new_business_key = scd_key.is_new_business_key();

        // Generate open/close dates and name for new business keys only
        let (web_open_date, web_close_date, web_name) = if is_new_business_key {
            let open_date = generate_join_key(
                &WebSiteGeneratorColumn::WebOpenDate,
                self.abstract_generator
                    .get_random_number_stream(&WebSiteGeneratorColumn::WebOpenDate),
                ConfigTable::DateDim,
                row_number_i64,
                scaling,
            )?;

            let close_date = generate_join_key(
                &WebSiteGeneratorColumn::WebCloseDate,
                self.abstract_generator
                    .get_random_number_stream(&WebSiteGeneratorColumn::WebCloseDate),
                ConfigTable::DateDim,
                row_number_i64,
                scaling,
            )?;

            let close_date = if close_date > web_rec_end_date_id {
                -1
            } else {
                close_date
            };

            let name = format!("site_{}", (row_number / 6));

            (open_date, close_date, name)
        } else {
            let prev = self.previous_row.as_ref().unwrap();
            (
                prev.web_open_date(),
                prev.web_close_date(),
                prev.web_name().to_string(),
            )
        };

        // Field change flags control whether a field changes from one row to the next
        let mut field_change_flags = self
            .abstract_generator
            .get_random_number_stream(&WebSiteGeneratorColumn::WebScd)
            .next_random() as i32;

        // Generate web_manager
        let first_name = NamesDistributions::pick_random_first_name(
            if session.is_sexist() {
                FirstNamesWeights::MaleFrequency
            } else {
                FirstNamesWeights::GeneralFrequency
            },
            self.abstract_generator
                .get_random_number_stream(&WebSiteGeneratorColumn::WebManager),
        )?;
        let last_name = NamesDistributions::pick_random_last_name(
            self.abstract_generator
                .get_random_number_stream(&WebSiteGeneratorColumn::WebManager),
        )?;
        let mut web_manager = format!("{} {}", first_name, last_name);
        if let Some(ref prev) = self.previous_row {
            web_manager = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                prev.web_manager().to_string(),
                web_manager,
            );
        }
        field_change_flags >>= 1;

        // Generate web_market_id
        let mut web_market_id = RandomValueGenerator::generate_uniform_random_int(
            1,
            6,
            self.abstract_generator
                .get_random_number_stream(&WebSiteGeneratorColumn::WebMarketId),
        );
        if let Some(ref prev) = self.previous_row {
            web_market_id = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                prev.web_market_id(),
                web_market_id,
            );
        }
        field_change_flags >>= 1;

        // Generate web_market_class
        let mut web_market_class = RandomValueGenerator::generate_random_text(
            20,
            50,
            self.abstract_generator
                .get_random_number_stream(&WebSiteGeneratorColumn::WebMarketClass),
        );
        if let Some(ref prev) = self.previous_row {
            web_market_class = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                prev.web_market_class().to_string(),
                web_market_class,
            );
        }
        field_change_flags >>= 1;

        // Generate web_market_desc
        let mut web_market_desc = RandomValueGenerator::generate_random_text(
            20,
            100,
            self.abstract_generator
                .get_random_number_stream(&WebSiteGeneratorColumn::WebMarketDesc),
        );
        if let Some(ref prev) = self.previous_row {
            web_market_desc = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                prev.web_market_desc().to_string(),
                web_market_desc,
            );
        }
        field_change_flags >>= 1;

        // Generate web_market_manager
        let first_name = NamesDistributions::pick_random_first_name(
            if session.is_sexist() {
                FirstNamesWeights::MaleFrequency
            } else {
                FirstNamesWeights::GeneralFrequency
            },
            self.abstract_generator
                .get_random_number_stream(&WebSiteGeneratorColumn::WebMarketManager),
        )?;
        let last_name = NamesDistributions::pick_random_last_name(
            self.abstract_generator
                .get_random_number_stream(&WebSiteGeneratorColumn::WebMarketManager),
        )?;
        let mut web_market_manager = format!("{} {}", first_name, last_name);
        if let Some(ref prev) = self.previous_row {
            web_market_manager = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                prev.web_market_manager().to_string(),
                web_market_manager,
            );
        }
        field_change_flags >>= 1;

        // Generate web_company_id
        let mut web_company_id = RandomValueGenerator::generate_uniform_random_int(
            1,
            6,
            self.abstract_generator
                .get_random_number_stream(&WebSiteGeneratorColumn::WebCompanyId),
        );
        if let Some(ref prev) = self.previous_row {
            web_company_id = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                prev.web_company_id(),
                web_company_id,
            );
        }
        field_change_flags >>= 1;

        // Generate web_company_name
        // generate_word doesn't use random numbers - it deterministically creates from seed
        // We still need to consume the stream to maintain RNG state
        let _company_name_stream = self
            .abstract_generator
            .get_random_number_stream(&WebSiteGeneratorColumn::WebCompanyName);
        let mut web_company_name = RandomValueGenerator::generate_word(
            web_company_id as i64,
            100,
            crate::distribution::get_syllables_distribution(),
        );
        if let Some(ref prev) = self.previous_row {
            web_company_name = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                prev.web_company_name().to_string(),
                web_company_name,
            );
        }
        field_change_flags >>= 1;

        // Generate address
        let mut web_address = Address::make_address_for_column(
            Table::WebSite,
            self.abstract_generator
                .get_random_number_stream(&WebSiteGeneratorColumn::WebAddress),
            scaling,
        )?;

        // Some address fields always use new value due to bug in C code, but we still update flags
        field_change_flags >>= 1; // city
        field_change_flags >>= 1; // county

        // gmt_offset
        let mut gmt_offset = web_address.get_gmt_offset();
        if let Some(ref prev) = self.previous_row {
            gmt_offset = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                prev.web_address().get_gmt_offset(),
                gmt_offset,
            );
        }
        field_change_flags >>= 1;

        field_change_flags >>= 1; // state
        field_change_flags >>= 1; // streetType
        field_change_flags >>= 1; // streetName1
        field_change_flags >>= 1; // streetName2

        // street_number
        let mut street_number = web_address.get_street_number();
        if let Some(ref prev) = self.previous_row {
            street_number = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                prev.web_address().get_street_number(),
                street_number,
            );
        }
        field_change_flags >>= 1;

        // zip
        let mut zip = web_address.get_zip();
        if let Some(ref prev) = self.previous_row {
            zip = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                prev.web_address().get_zip(),
                zip,
            );
        }
        field_change_flags >>= 1;

        // Reconstruct address with potentially changed fields
        web_address = Address::new(
            web_address.get_suite_number().to_string(),
            street_number,
            web_address.get_street_name1().to_string(),
            web_address.get_street_name2().to_string(),
            web_address.get_street_type().to_string(),
            web_address.get_city().to_string(),
            web_address.get_county().map(|s| s.to_string()),
            web_address.get_state().to_string(),
            web_address.get_country().to_string(),
            zip,
            gmt_offset,
        )?;

        // Generate web_tax_percentage
        let mut web_tax_percentage = RandomValueGenerator::generate_uniform_random_decimal(
            Decimal::ZERO,
            Decimal::new(12, 2)?,
            self.abstract_generator
                .get_random_number_stream(&WebSiteGeneratorColumn::WebTaxPercentage),
        );
        if let Some(ref prev) = self.previous_row {
            web_tax_percentage = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_business_key,
                *prev.web_tax_percentage(),
                web_tax_percentage,
            );
        }

        let row = WebSiteRow::new(
            null_bit_map,
            web_site_sk,
            web_site_id,
            web_rec_start_date_id,
            web_rec_end_date_id,
            web_name,
            web_open_date,
            web_close_date,
            web_class,
            web_manager,
            web_market_id,
            web_market_class,
            web_market_desc,
            web_market_manager,
            web_company_id,
            web_company_name,
            web_address,
            web_tax_percentage,
        );

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::row::dat_values;

    #[test]
    fn test_web_site_row_generator_creation() {
        let _generator = WebSiteRowGenerator::new();
    }

    #[test]
    fn test_generate_web_site_row() {
        use crate::config::Session;

        let mut generator = WebSiteRowGenerator::new();
        let session = Session::default();

        let result = generator.generate_row_and_child_rows(1, &session, None, None);
        assert!(result.is_ok());

        let row_result = result.unwrap();
        let values = dat_values(&row_result.get_rows()[0]);
        assert_eq!(values.len(), 26);
    }
}
