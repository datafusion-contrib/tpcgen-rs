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

/// Generator columns for HOUSEHOLD_DEMOGRAPHICS table (HouseholdDemographicsGeneratorColumn)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HouseholdDemographicsGeneratorColumn {
    HdDemoSk,
    HdIncomeBandId,
    HdBuyPotential,
    HdDepCount,
    HdVehicleCount,
    HdNulls,
}

impl HouseholdDemographicsGeneratorColumn {
    /// Get all generator columns in order (values())
    pub fn values() -> &'static [HouseholdDemographicsGeneratorColumn] {
        use HouseholdDemographicsGeneratorColumn::*;
        static VALUES: &[HouseholdDemographicsGeneratorColumn] = &[
            HdDemoSk,
            HdIncomeBandId,
            HdBuyPotential,
            HdDepCount,
            HdVehicleCount,
            HdNulls,
        ];
        VALUES
    }
}

impl GeneratorColumn for HouseholdDemographicsGeneratorColumn {
    fn get_global_column_number(&self) -> i32 {
        match self {
            Self::HdDemoSk => 188,
            Self::HdIncomeBandId => 189,
            Self::HdBuyPotential => 190,
            Self::HdDepCount => 191,
            Self::HdVehicleCount => 192,
            Self::HdNulls => 193,
        }
    }

    fn get_seeds_per_row(&self) -> u32 {
        match self {
            Self::HdDemoSk => 1,
            Self::HdIncomeBandId => 1,
            Self::HdBuyPotential => 1,
            Self::HdDepCount => 1,
            Self::HdVehicleCount => 1,
            Self::HdNulls => 2,
        }
    }

    fn get_table(&self) -> Table {
        Table::HouseholdDemographics
    }
}
