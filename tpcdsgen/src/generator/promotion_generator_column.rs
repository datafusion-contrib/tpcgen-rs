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

use crate::column::Table;
use crate::generator::GeneratorColumn;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromotionGeneratorColumn {
    PPromoSk,
    PPromoId,
    PStartDateId,
    PEndDateId,
    PItemSk,
    PCost,
    PResponseTarget,
    PPromoName,
    PChannelDmail,
    PChannelEmail,
    PChannelCatalog,
    PChannelTv,
    PChannelRadio,
    PChannelPress,
    PChannelEvent,
    PChannelDemo,
    PChannelDetails,
    PPurpose,
    PDiscountActive,
    PNulls,
}

impl PromotionGeneratorColumn {
    pub fn values() -> &'static [PromotionGeneratorColumn] {
        &[
            PromotionGeneratorColumn::PPromoSk,
            PromotionGeneratorColumn::PPromoId,
            PromotionGeneratorColumn::PStartDateId,
            PromotionGeneratorColumn::PEndDateId,
            PromotionGeneratorColumn::PItemSk,
            PromotionGeneratorColumn::PCost,
            PromotionGeneratorColumn::PResponseTarget,
            PromotionGeneratorColumn::PPromoName,
            PromotionGeneratorColumn::PChannelDmail,
            PromotionGeneratorColumn::PChannelEmail,
            PromotionGeneratorColumn::PChannelCatalog,
            PromotionGeneratorColumn::PChannelTv,
            PromotionGeneratorColumn::PChannelRadio,
            PromotionGeneratorColumn::PChannelPress,
            PromotionGeneratorColumn::PChannelEvent,
            PromotionGeneratorColumn::PChannelDemo,
            PromotionGeneratorColumn::PChannelDetails,
            PromotionGeneratorColumn::PPurpose,
            PromotionGeneratorColumn::PDiscountActive,
            PromotionGeneratorColumn::PNulls,
        ]
    }
}

impl GeneratorColumn for PromotionGeneratorColumn {
    fn get_table(&self) -> Table {
        Table::Promotion
    }

    fn get_global_column_number(&self) -> i32 {
        match self {
            PromotionGeneratorColumn::PPromoSk => 228,
            PromotionGeneratorColumn::PPromoId => 229,
            PromotionGeneratorColumn::PStartDateId => 230,
            PromotionGeneratorColumn::PEndDateId => 231,
            PromotionGeneratorColumn::PItemSk => 232,
            PromotionGeneratorColumn::PCost => 233,
            PromotionGeneratorColumn::PResponseTarget => 234,
            PromotionGeneratorColumn::PPromoName => 235,
            PromotionGeneratorColumn::PChannelDmail => 236,
            PromotionGeneratorColumn::PChannelEmail => 237,
            PromotionGeneratorColumn::PChannelCatalog => 238,
            PromotionGeneratorColumn::PChannelTv => 239,
            PromotionGeneratorColumn::PChannelRadio => 240,
            PromotionGeneratorColumn::PChannelPress => 241,
            PromotionGeneratorColumn::PChannelEvent => 242,
            PromotionGeneratorColumn::PChannelDemo => 243,
            PromotionGeneratorColumn::PChannelDetails => 244,
            PromotionGeneratorColumn::PPurpose => 245,
            PromotionGeneratorColumn::PDiscountActive => 246,
            PromotionGeneratorColumn::PNulls => 247,
        }
    }

    fn get_seeds_per_row(&self) -> u32 {
        match self {
            PromotionGeneratorColumn::PChannelDetails => 100,
            PromotionGeneratorColumn::PNulls => 2,
            _ => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_promotion_generator_column_count() {
        assert_eq!(PromotionGeneratorColumn::values().len(), 20);
    }

    #[test]
    fn test_promotion_generator_global_column_numbers() {
        assert_eq!(
            PromotionGeneratorColumn::PPromoSk.get_global_column_number(),
            228
        );
        assert_eq!(
            PromotionGeneratorColumn::PNulls.get_global_column_number(),
            247
        );
    }

    #[test]
    fn test_promotion_generator_seeds_per_row() {
        assert_eq!(PromotionGeneratorColumn::PPromoSk.get_seeds_per_row(), 1);
        assert_eq!(
            PromotionGeneratorColumn::PChannelDetails.get_seeds_per_row(),
            100
        );
        assert_eq!(PromotionGeneratorColumn::PNulls.get_seeds_per_row(), 2);
    }

    #[test]
    fn test_promotion_generator_table() {
        assert_eq!(
            PromotionGeneratorColumn::PPromoSk.get_table(),
            Table::Promotion
        );
    }
}
