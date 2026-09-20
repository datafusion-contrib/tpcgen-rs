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

use crate::business_key_generator::make_business_key;
use crate::config::Table as ConfigTable;
use crate::generator::PromotionGeneratorColumn;
use crate::join_key_utils::generate_join_key;
use crate::nulls::create_null_bit_map;
use crate::random::RandomValueGenerator;
use crate::row::{AbstractRowGenerator, PromotionRow, RowGenerator, RowGeneratorResult};
use crate::table::Table;
use crate::types::{Date, Decimal};

const PROMO_START_MIN: i32 = -720;
const PROMO_START_MAX: i32 = 100;
const PROMO_LENGTH_MIN: i32 = 1;
const PROMO_LENGTH_MAX: i32 = 60;
const PROMO_NAME_LENGTH: i32 = 5;
const PROMO_DETAIL_LENGTH_MIN: i32 = 20;
const PROMO_DETAIL_LENGTH_MAX: i32 = 60;

pub struct PromotionRowGenerator {
    abstract_row_generator: AbstractRowGenerator,
}

impl PromotionRowGenerator {
    pub fn new() -> Self {
        PromotionRowGenerator {
            abstract_row_generator: AbstractRowGenerator::new(Table::Promotion),
        }
    }
}

impl Default for PromotionRowGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl RowGenerator for PromotionRowGenerator {
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
            Table::Promotion,
            self.abstract_row_generator
                .get_random_number_stream(&PromotionGeneratorColumn::PNulls),
        );

        let p_promo_sk = row_number_i64;
        let p_promo_id = make_business_key(row_number);

        let p_start_date_id = Date::JULIAN_DATE_MINIMUM as i64
            + RandomValueGenerator::generate_uniform_random_int(
                PROMO_START_MIN,
                PROMO_START_MAX,
                self.abstract_row_generator
                    .get_random_number_stream(&PromotionGeneratorColumn::PStartDateId),
            ) as i64;

        let p_end_date_id = p_start_date_id
            + RandomValueGenerator::generate_uniform_random_int(
                PROMO_LENGTH_MIN,
                PROMO_LENGTH_MAX,
                self.abstract_row_generator
                    .get_random_number_stream(&PromotionGeneratorColumn::PEndDateId),
            ) as i64;

        let p_item_sk = generate_join_key(
            &PromotionGeneratorColumn::PItemSk,
            self.abstract_row_generator
                .get_random_number_stream(&PromotionGeneratorColumn::PItemSk),
            ConfigTable::Item,
            1,
            scaling,
        )?;

        let p_cost = Decimal::new(100000, 2)?;
        let p_response_target = 1;

        // generate_word doesn't use random numbers - it deterministically creates from seed
        // We still need to consume the stream to maintain RNG state
        let _promo_name_stream = self
            .abstract_row_generator
            .get_random_number_stream(&PromotionGeneratorColumn::PPromoName);
        let p_promo_name = RandomValueGenerator::generate_word(
            row_number_i64,
            PROMO_NAME_LENGTH,
            crate::distribution::get_syllables_distribution(),
        );

        // Generate channel flags using a single random int (0-511)
        let mut flags = RandomValueGenerator::generate_uniform_random_int(
            0,
            511,
            self.abstract_row_generator
                .get_random_number_stream(&PromotionGeneratorColumn::PChannelDmail),
        );

        let p_channel_dmail = (flags & 0x01) != 0;
        flags <<= 1;

        let p_channel_email = (flags & 0x01) != 0;
        flags <<= 1;

        let p_channel_catalog = (flags & 0x01) != 0;
        flags <<= 1;

        let p_channel_tv = (flags & 0x01) != 0;
        flags <<= 1;

        let p_channel_radio = (flags & 0x01) != 0;
        flags <<= 1;

        let p_channel_press = (flags & 0x01) != 0;
        flags <<= 1;

        let p_channel_event = (flags & 0x01) != 0;
        flags <<= 1;

        let p_channel_demo = (flags & 0x01) != 0;
        flags <<= 1;

        let p_discount_active = (flags & 0x01) != 0;

        let p_channel_details = RandomValueGenerator::generate_random_text(
            PROMO_DETAIL_LENGTH_MIN,
            PROMO_DETAIL_LENGTH_MAX,
            self.abstract_row_generator
                .get_random_number_stream(&PromotionGeneratorColumn::PChannelDetails),
        );

        let p_purpose = "Unknown".to_string();

        Ok(RowGeneratorResult::new(PromotionRow::new(
            null_bit_map,
            p_promo_sk,
            p_promo_id,
            p_start_date_id,
            p_end_date_id,
            p_item_sk,
            p_cost,
            p_response_target,
            p_promo_name,
            p_channel_dmail,
            p_channel_email,
            p_channel_catalog,
            p_channel_tv,
            p_channel_radio,
            p_channel_press,
            p_channel_event,
            p_channel_demo,
            p_channel_details,
            p_purpose,
            p_discount_active,
        )))
    }

    fn consume_remaining_seeds_for_row(&mut self) {
        self.abstract_row_generator
            .consume_remaining_seeds_for_row();
    }

    fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64) {
        self.abstract_row_generator
            .skip_rows_until_starting_row_number(starting_row_number);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::row::dat_values;

    #[test]
    fn test_generate_promotion_row() {
        use crate::config::Session;

        let mut generator = PromotionRowGenerator::new();
        let session = Session::default();

        let result = generator.generate_row_and_child_rows(1, &session, None, None);
        assert!(result.is_ok());

        let row_result = result.unwrap();
        let values = dat_values(&row_result.get_rows()[0]);
        assert_eq!(values.len(), 19);
    }
}
