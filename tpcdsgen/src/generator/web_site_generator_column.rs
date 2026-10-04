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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WebSiteGeneratorColumn {
    WebSiteSk,
    WebSiteId,
    WebRecStartDateId,
    WebRecEndDateId,
    WebName,
    WebOpenDate,
    WebCloseDate,
    WebClass,
    WebManager,
    WebMarketId,
    WebMarketClass,
    WebMarketDesc,
    WebMarketManager,
    WebCompanyId,
    WebCompanyName,
    WebAddressStreetNum,
    WebAddressStreetName1,
    WebAddressStreetType,
    WebAddressSuiteNum,
    WebAddressCity,
    WebAddressCounty,
    WebAddressState,
    WebAddressZip,
    WebAddressCountry,
    WebAddressGmtOffset,
    WebTaxPercentage,
    WebNulls,
    WebAddress,
    WebScd,
}

impl WebSiteGeneratorColumn {
    pub fn values() -> &'static [WebSiteGeneratorColumn] {
        &[
            WebSiteGeneratorColumn::WebSiteSk,
            WebSiteGeneratorColumn::WebSiteId,
            WebSiteGeneratorColumn::WebRecStartDateId,
            WebSiteGeneratorColumn::WebRecEndDateId,
            WebSiteGeneratorColumn::WebName,
            WebSiteGeneratorColumn::WebOpenDate,
            WebSiteGeneratorColumn::WebCloseDate,
            WebSiteGeneratorColumn::WebClass,
            WebSiteGeneratorColumn::WebManager,
            WebSiteGeneratorColumn::WebMarketId,
            WebSiteGeneratorColumn::WebMarketClass,
            WebSiteGeneratorColumn::WebMarketDesc,
            WebSiteGeneratorColumn::WebMarketManager,
            WebSiteGeneratorColumn::WebCompanyId,
            WebSiteGeneratorColumn::WebCompanyName,
            WebSiteGeneratorColumn::WebAddressStreetNum,
            WebSiteGeneratorColumn::WebAddressStreetName1,
            WebSiteGeneratorColumn::WebAddressStreetType,
            WebSiteGeneratorColumn::WebAddressSuiteNum,
            WebSiteGeneratorColumn::WebAddressCity,
            WebSiteGeneratorColumn::WebAddressCounty,
            WebSiteGeneratorColumn::WebAddressState,
            WebSiteGeneratorColumn::WebAddressZip,
            WebSiteGeneratorColumn::WebAddressCountry,
            WebSiteGeneratorColumn::WebAddressGmtOffset,
            WebSiteGeneratorColumn::WebTaxPercentage,
            WebSiteGeneratorColumn::WebNulls,
            WebSiteGeneratorColumn::WebAddress,
            WebSiteGeneratorColumn::WebScd,
        ]
    }
}

impl GeneratorColumn for WebSiteGeneratorColumn {
    fn get_table(&self) -> Table {
        Table::WebSite
    }

    fn get_global_column_number(&self) -> i32 {
        match self {
            WebSiteGeneratorColumn::WebSiteSk => 447,
            WebSiteGeneratorColumn::WebSiteId => 448,
            WebSiteGeneratorColumn::WebRecStartDateId => 449,
            WebSiteGeneratorColumn::WebRecEndDateId => 450,
            WebSiteGeneratorColumn::WebName => 451,
            WebSiteGeneratorColumn::WebOpenDate => 452,
            WebSiteGeneratorColumn::WebCloseDate => 453,
            WebSiteGeneratorColumn::WebClass => 454,
            WebSiteGeneratorColumn::WebManager => 455,
            WebSiteGeneratorColumn::WebMarketId => 456,
            WebSiteGeneratorColumn::WebMarketClass => 457,
            WebSiteGeneratorColumn::WebMarketDesc => 458,
            WebSiteGeneratorColumn::WebMarketManager => 459,
            WebSiteGeneratorColumn::WebCompanyId => 460,
            WebSiteGeneratorColumn::WebCompanyName => 461,
            WebSiteGeneratorColumn::WebAddressStreetNum => 462,
            WebSiteGeneratorColumn::WebAddressStreetName1 => 463,
            WebSiteGeneratorColumn::WebAddressStreetType => 464,
            WebSiteGeneratorColumn::WebAddressSuiteNum => 465,
            WebSiteGeneratorColumn::WebAddressCity => 466,
            WebSiteGeneratorColumn::WebAddressCounty => 467,
            WebSiteGeneratorColumn::WebAddressState => 468,
            WebSiteGeneratorColumn::WebAddressZip => 469,
            WebSiteGeneratorColumn::WebAddressCountry => 470,
            WebSiteGeneratorColumn::WebAddressGmtOffset => 471,
            WebSiteGeneratorColumn::WebTaxPercentage => 472,
            WebSiteGeneratorColumn::WebNulls => 473,
            WebSiteGeneratorColumn::WebAddress => 474,
            WebSiteGeneratorColumn::WebScd => 475,
        }
    }

    fn get_seeds_per_row(&self) -> u32 {
        match self {
            WebSiteGeneratorColumn::WebManager => 2,
            WebSiteGeneratorColumn::WebMarketClass => 20,
            WebSiteGeneratorColumn::WebMarketDesc => 100,
            WebSiteGeneratorColumn::WebMarketManager => 2,
            WebSiteGeneratorColumn::WebNulls => 2,
            WebSiteGeneratorColumn::WebAddress => 7,
            WebSiteGeneratorColumn::WebScd => 70,
            _ => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_site_generator_column_count() {
        assert_eq!(WebSiteGeneratorColumn::values().len(), 29);
    }

    #[test]
    fn test_web_site_generator_column_numbers() {
        assert_eq!(
            WebSiteGeneratorColumn::WebSiteSk.get_global_column_number(),
            447
        );
        assert_eq!(
            WebSiteGeneratorColumn::WebScd.get_global_column_number(),
            475
        );
    }

    #[test]
    fn test_web_site_generator_seeds_per_row() {
        assert_eq!(WebSiteGeneratorColumn::WebSiteSk.get_seeds_per_row(), 1);
        assert_eq!(WebSiteGeneratorColumn::WebManager.get_seeds_per_row(), 2);
        assert_eq!(
            WebSiteGeneratorColumn::WebMarketClass.get_seeds_per_row(),
            20
        );
        assert_eq!(
            WebSiteGeneratorColumn::WebMarketDesc.get_seeds_per_row(),
            100
        );
        assert_eq!(WebSiteGeneratorColumn::WebAddress.get_seeds_per_row(), 7);
        assert_eq!(WebSiteGeneratorColumn::WebScd.get_seeds_per_row(), 70);
    }
}
