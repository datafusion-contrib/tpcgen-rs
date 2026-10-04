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

/// DbgenVersion generator columns (DbgenVersionGeneratorColumn enum)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DbgenVersionGeneratorColumn {
    DvVersion,
    DvCreateDate,
    DvCreateTime,
    DvCmdlineArgs,
}

impl DbgenVersionGeneratorColumn {
    /// Get all generator columns in order
    pub fn values() -> &'static [DbgenVersionGeneratorColumn] {
        use DbgenVersionGeneratorColumn::*;
        static VALUES: &[DbgenVersionGeneratorColumn] =
            &[DvVersion, DvCreateDate, DvCreateTime, DvCmdlineArgs];
        VALUES
    }

    /// Get the global column number and seeds per row for this generator column
    /// Values exactly match Java implementation
    fn get_column_info(&self) -> (i32, u32) {
        use DbgenVersionGeneratorColumn::*;
        match self {
            DvVersion => (476, 1),
            DvCreateDate => (477, 1),
            DvCreateTime => (478, 1),
            DvCmdlineArgs => (479, 1),
        }
    }
}

impl GeneratorColumn for DbgenVersionGeneratorColumn {
    fn get_table(&self) -> Table {
        Table::DbgenVersion
    }

    fn get_global_column_number(&self) -> i32 {
        self.get_column_info().0
    }

    fn get_seeds_per_row(&self) -> u32 {
        self.get_column_info().1
    }
}
