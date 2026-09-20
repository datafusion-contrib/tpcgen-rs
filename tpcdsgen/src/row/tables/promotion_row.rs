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

use crate::generator::{GeneratorColumn, PromotionGeneratorColumn};
use crate::row::table_row::DatField;
use crate::types::Decimal;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct PromotionRow {
    null_bit_map: i64,
    pub(crate) p_promo_sk: i64,
    pub(crate) p_promo_id: String,
    pub(crate) p_start_date_id: i64,
    pub(crate) p_end_date_id: i64,
    pub(crate) p_item_sk: i64,
    pub(crate) p_cost: Decimal,
    pub(crate) p_response_target: i32,
    pub(crate) p_promo_name: String,
    pub(crate) p_channel_dmail: bool,
    pub(crate) p_channel_email: bool,
    pub(crate) p_channel_catalog: bool,
    pub(crate) p_channel_tv: bool,
    pub(crate) p_channel_radio: bool,
    pub(crate) p_channel_press: bool,
    pub(crate) p_channel_event: bool,
    pub(crate) p_channel_demo: bool,
    pub(crate) p_channel_details: String,
    pub(crate) p_purpose: String,
    pub(crate) p_discount_active: bool,
}

impl PromotionRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        null_bit_map: i64,
        p_promo_sk: i64,
        p_promo_id: String,
        p_start_date_id: i64,
        p_end_date_id: i64,
        p_item_sk: i64,
        p_cost: Decimal,
        p_response_target: i32,
        p_promo_name: String,
        p_channel_dmail: bool,
        p_channel_email: bool,
        p_channel_catalog: bool,
        p_channel_tv: bool,
        p_channel_radio: bool,
        p_channel_press: bool,
        p_channel_event: bool,
        p_channel_demo: bool,
        p_channel_details: String,
        p_purpose: String,
        p_discount_active: bool,
    ) -> Self {
        PromotionRow {
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
        }
    }

    pub(crate) fn is_null_at(&self, column: PromotionGeneratorColumn) -> bool {
        let bit_position = column.get_global_column_number()
            - PromotionGeneratorColumn::PPromoSk.get_global_column_number();
        (self.null_bit_map & (1 << bit_position)) != 0
    }

    pub fn null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    pub fn get_p_promo_sk(&self) -> i64 {
        self.p_promo_sk
    }

    pub fn get_p_promo_id(&self) -> &str {
        &self.p_promo_id
    }

    pub fn get_p_start_date_id(&self) -> i64 {
        self.p_start_date_id
    }

    pub fn get_p_end_date_id(&self) -> i64 {
        self.p_end_date_id
    }

    pub fn get_p_item_sk(&self) -> i64 {
        self.p_item_sk
    }

    pub fn get_p_cost(&self) -> Decimal {
        self.p_cost
    }

    pub fn get_p_response_target(&self) -> i32 {
        self.p_response_target
    }

    pub fn get_p_promo_name(&self) -> &str {
        &self.p_promo_name
    }

    pub fn get_p_channel_dmail(&self) -> bool {
        self.p_channel_dmail
    }

    pub fn get_p_channel_email(&self) -> bool {
        self.p_channel_email
    }

    pub fn get_p_channel_catalog(&self) -> bool {
        self.p_channel_catalog
    }

    pub fn get_p_channel_tv(&self) -> bool {
        self.p_channel_tv
    }

    pub fn get_p_channel_radio(&self) -> bool {
        self.p_channel_radio
    }

    pub fn get_p_channel_press(&self) -> bool {
        self.p_channel_press
    }

    pub fn get_p_channel_event(&self) -> bool {
        self.p_channel_event
    }

    pub fn get_p_channel_demo(&self) -> bool {
        self.p_channel_demo
    }

    pub fn get_p_channel_details(&self) -> &str {
        &self.p_channel_details
    }

    pub fn get_p_purpose(&self) -> &str {
        &self.p_purpose
    }

    pub fn get_p_discount_active(&self) -> bool {
        self.p_discount_active
    }
}

impl PromotionRow {
    /// DAT field for a surrogate key: NULL when the null bit is set or the
    /// key is -1.
    pub(crate) fn key_field(&self, key: i64, column: PromotionGeneratorColumn) -> DatField<i64> {
        DatField::new(key, key == -1 || self.is_null_at(column))
    }

    /// DAT field for a regular value: NULL when the null bit is set.
    pub(crate) fn field<T>(&self, value: T, column: PromotionGeneratorColumn) -> DatField<T> {
        DatField::new(value, self.is_null_at(column))
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces one
/// `|`-terminated field per column.
impl fmt::Display for PromotionRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use PromotionGeneratorColumn::*;

        write!(
            f,
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|",
            self.key_field(self.p_promo_sk, PPromoSk),
            self.field(&self.p_promo_id, PPromoId),
            self.key_field(self.p_start_date_id, PStartDateId),
            self.key_field(self.p_end_date_id, PEndDateId),
            self.key_field(self.p_item_sk, PItemSk),
            self.field(self.p_cost, PCost),
            self.field(self.p_response_target, PResponseTarget),
            self.field(&self.p_promo_name, PPromoName),
            DatField::yes_no(self.p_channel_dmail, self.is_null_at(PChannelDmail)),
            DatField::yes_no(self.p_channel_email, self.is_null_at(PChannelEmail)),
            DatField::yes_no(self.p_channel_catalog, self.is_null_at(PChannelCatalog)),
            DatField::yes_no(self.p_channel_tv, self.is_null_at(PChannelTv)),
            DatField::yes_no(self.p_channel_radio, self.is_null_at(PChannelRadio)),
            DatField::yes_no(self.p_channel_press, self.is_null_at(PChannelPress)),
            DatField::yes_no(self.p_channel_event, self.is_null_at(PChannelEvent)),
            DatField::yes_no(self.p_channel_demo, self.is_null_at(PChannelDemo)),
            self.field(&self.p_channel_details, PChannelDetails),
            self.field(&self.p_purpose, PPurpose),
            DatField::yes_no(self.p_discount_active, self.is_null_at(PDiscountActive)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::row::dat_values;

    #[test]
    fn test_promotion_row_creation() {
        let row = PromotionRow::new(
            0,
            1,
            "test_id".to_string(),
            2450815,
            2450875,
            100,
            Decimal::new(1000, 2).unwrap(),
            1,
            "TestPromo".to_string(),
            true,
            false,
            true,
            false,
            true,
            false,
            true,
            false,
            "Details".to_string(),
            "Unknown".to_string(),
            true,
        );

        assert_eq!(row.p_promo_sk, 1);
        assert_eq!(row.p_promo_id, "test_id");
    }

    #[test]
    fn test_promotion_row_values() {
        let row = PromotionRow::new(
            0,
            1,
            "AAAAAAAABAAAAAAA".to_string(),
            2450815,
            2450875,
            100,
            Decimal::new(100000, 2).unwrap(),
            1,
            "TestPromo".to_string(),
            true,
            false,
            true,
            false,
            true,
            false,
            true,
            false,
            "Channel details".to_string(),
            "Unknown".to_string(),
            true,
        );

        let values = dat_values(&row);
        assert_eq!(values.len(), 19);
        assert_eq!(values[0], "1");
        assert_eq!(values[1], "AAAAAAAABAAAAAAA");
        assert_eq!(values[8], "Y"); // p_channel_dmail
        assert_eq!(values[9], "N"); // p_channel_email
    }
}
