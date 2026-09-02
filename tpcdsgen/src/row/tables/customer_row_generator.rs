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

//! Customer row generator (CustomerRowGenerator)

use crate::business_key_generator::make_business_key;
use crate::config::Session;
use crate::distribution::{
    pick_random_country, FirstNamesWeights, NamesDistributions, SalutationsWeights,
};
use crate::error::Result;
use crate::generator::CustomerGeneratorColumn;
use crate::join_key_utils::generate_join_key;
use crate::nulls::create_null_bit_map;
use crate::random::RandomValueGenerator;
use crate::row::customer_row::CustomerRow;
use crate::row::{AbstractRowGenerator, RowGenerator, RowGeneratorResult};
use crate::table::Table;
use crate::types::Date;

/// Customer row generator (CustomerRowGenerator)
pub struct CustomerRowGenerator {
    abstract_generator: AbstractRowGenerator,
}

impl CustomerRowGenerator {
    pub fn new() -> Self {
        CustomerRowGenerator {
            abstract_generator: AbstractRowGenerator::new(Table::Customer),
        }
    }

    fn generate_customer_row(&mut self, row_number: u64, session: &Session) -> Result<CustomerRow> {
        use CustomerGeneratorColumn::*;

        let row_number_i64 = i64::try_from(row_number).expect("row number fits in i64");

        let c_customer_sk = row_number_i64;
        let c_customer_id = make_business_key(row_number);

        // Preferred customer flag - MUST BE FIRST (matches Java order line 72)
        let stream = self
            .abstract_generator
            .get_random_number_stream(&CPreferredCustFlag);
        let random_int = RandomValueGenerator::generate_uniform_random_int(1, 100, stream);
        let c_preferred_percent = 50;
        let c_preferred_cust_flag = random_int < c_preferred_percent;

        let scaling = session.get_scaling();

        // Generate join keys (matches Java order lines 77-79)
        let stream = self
            .abstract_generator
            .get_random_number_stream(&CCurrentHdemoSk);
        let c_current_hdemo_sk = generate_join_key(
            &CCurrentHdemoSk,
            stream,
            crate::config::Table::HouseholdDemographics,
            1,
            scaling,
        )?;

        let stream = self
            .abstract_generator
            .get_random_number_stream(&CCurrentCdemoSk);
        let c_current_cdemo_sk = generate_join_key(
            &CCurrentCdemoSk,
            stream,
            crate::config::Table::CustomerDemographics,
            1,
            scaling,
        )?;

        let stream = self
            .abstract_generator
            .get_random_number_stream(&CCurrentAddrSk);
        let c_current_addr_sk = generate_join_key(
            &CCurrentAddrSk,
            stream,
            crate::config::Table::CustomerAddress,
            c_customer_sk,
            scaling,
        )?;

        // Name generation (matches Java order lines 81-85)
        let stream = self
            .abstract_generator
            .get_random_number_stream(&CFirstName);
        let name_index =
            NamesDistributions::pick_random_index(FirstNamesWeights::GeneralFrequency, stream)?;
        let c_first_name = NamesDistributions::get_first_name_from_index(name_index)?.to_string();

        let stream = self.abstract_generator.get_random_number_stream(&CLastName);
        let c_last_name = NamesDistributions::pick_random_last_name(stream)?.to_string();

        // Salutation based on gender frequency
        let female_name_weight = NamesDistributions::get_weight_for_index(
            name_index,
            FirstNamesWeights::FemaleFrequency,
        )?;
        let salutation_weight = if female_name_weight == 0 {
            SalutationsWeights::Male
        } else {
            SalutationsWeights::Female
        };
        let stream = self
            .abstract_generator
            .get_random_number_stream(&CSalutation);
        let c_salutation =
            NamesDistributions::pick_random_salutation(salutation_weight, stream)?.to_string();

        // Birthday generation (matches Java order lines 87-95)
        let max_birthday = Date::new(1992, 12, 31);
        let min_birthday = Date::new(1924, 1, 1);
        let one_year_ago = Date::from_julian_days(Date::JULIAN_TODAYS_DATE - 365);
        let ten_years_ago = Date::from_julian_days(Date::JULIAN_TODAYS_DATE - 3650);
        let today = Date::from_julian_days(Date::JULIAN_TODAYS_DATE);

        let stream = self.abstract_generator.get_random_number_stream(&CBirthDay);
        let birthday =
            RandomValueGenerator::generate_uniform_random_date(min_birthday, max_birthday, stream)?;
        let c_birth_day = birthday.day();
        let c_birth_month = birthday.month();
        let c_birth_year = birthday.year();

        // Email generation (matches Java order line 97)
        let stream = self
            .abstract_generator
            .get_random_number_stream(&CEmailAddress);
        let c_email_address =
            RandomValueGenerator::generate_random_email(&c_first_name, &c_last_name, stream);

        // Last review date (matches Java order lines 98-99)
        let stream = self
            .abstract_generator
            .get_random_number_stream(&CLastReviewDate);
        let last_review_date =
            RandomValueGenerator::generate_uniform_random_date(one_year_ago, today, stream)?;
        let c_last_review_date = last_review_date.to_julian_days();

        // First sales date (matches Java order lines 100-102)
        let stream = self
            .abstract_generator
            .get_random_number_stream(&CFirstSalesDateId);
        let first_sales_date =
            RandomValueGenerator::generate_uniform_random_date(ten_years_ago, today, stream)?;
        let c_first_sales_date_id = first_sales_date.to_julian_days();
        let c_first_shipto_date_id = c_first_sales_date_id + 30;

        // Birth country (matches Java order line 104)
        let stream = self
            .abstract_generator
            .get_random_number_stream(&CBirthCountry);
        let c_birth_country = pick_random_country(stream)?.to_string();

        // Generate null bit map (matches Java order line 123)
        let stream = self.abstract_generator.get_random_number_stream(&CNulls);
        let null_bit_map = create_null_bit_map(Table::Customer, stream);

        Ok(CustomerRow::new(
            c_customer_sk,
            c_customer_id,
            c_current_cdemo_sk,
            c_current_hdemo_sk,
            c_current_addr_sk,
            c_first_shipto_date_id,
            c_first_sales_date_id,
            c_salutation,
            c_first_name,
            c_last_name,
            c_preferred_cust_flag,
            c_birth_day,
            c_birth_month,
            c_birth_year,
            c_birth_country,
            c_email_address,
            c_last_review_date,
            null_bit_map,
        ))
    }
}

impl Default for CustomerRowGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl RowGenerator for CustomerRowGenerator {
    fn generate_row_and_child_rows(
        &mut self,
        row_number: u64,
        session: &Session,
        _parent_row_generator: Option<&mut dyn RowGenerator>,
        _child_row_generator: Option<&mut dyn RowGenerator>,
    ) -> Result<RowGeneratorResult> {
        let row = self.generate_customer_row(row_number, session)?;
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
