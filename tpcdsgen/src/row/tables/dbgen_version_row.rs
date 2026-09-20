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

use crate::row::table_row::DatField;
use crate::types::Date;
use std::fmt;

/// DbgenVersion table row
#[derive(Debug, Clone)]
pub struct DbgenVersionRow {
    null_bit_map: i64,
    pub(crate) dv_version: String,
    pub(crate) dv_create_date: Date,
    pub(crate) dv_create_time: i32,
    pub(crate) dv_cmdline_args: String,
}

impl DbgenVersionRow {
    pub fn new(
        null_bit_map: i64,
        dv_version: String,
        dv_create_date: Date,
        dv_create_time: i32,
        dv_cmdline_args: String,
    ) -> Self {
        DbgenVersionRow {
            null_bit_map,
            dv_version,
            dv_create_date,
            dv_create_time,
            dv_cmdline_args,
        }
    }

    /// Check if a column should be null based on the null bitmap
    fn should_be_null(&self, column_position: i32) -> bool {
        ((self.null_bit_map >> column_position) & 1) == 1
    }

    pub fn null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    pub fn get_dv_version(&self) -> &str {
        &self.dv_version
    }

    pub fn get_dv_create_date(&self) -> &Date {
        &self.dv_create_date
    }

    pub fn get_dv_create_time(&self) -> i32 {
        self.dv_create_time
    }

    pub fn get_dv_cmdline_args(&self) -> &str {
        &self.dv_cmdline_args
    }
}

/// Seconds-since-midnight rendered as `HH:MM:SS`.
pub(crate) struct TimeOfDay(pub(crate) i32);

impl fmt::Display for TimeOfDay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hour = self.0 / 3600;
        let minute = (self.0 / 60) % 60;
        let second = self.0 % 60;
        write!(f, "{hour:02}:{minute:02}:{second:02}")
    }
}

/// DAT field helper for this row's columns.
impl DbgenVersionRow {
    pub(crate) fn field<T>(&self, value: T, column_position: i32) -> DatField<T> {
        DatField::new(value, self.should_be_null(column_position))
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces one
/// `|`-terminated field per column.
impl fmt::Display for DbgenVersionRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}|",
            self.field(&self.dv_version, 0),
            self.field(self.dv_create_date, 1),
            self.field(TimeOfDay(self.dv_create_time), 2),
            self.field(&self.dv_cmdline_args, 3),
        )
    }
}
