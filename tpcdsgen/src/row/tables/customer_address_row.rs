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

//! Customer address row (CustomerAddressRow)

use crate::row::table_row::DatField;
use crate::types::Address;
use std::fmt;

/// Customer address table row (CustomerAddressRow)
#[derive(Debug, Clone)]
pub struct CustomerAddressRow {
    pub(crate) null_bit_map: i64,
    pub(crate) ca_addr_sk: i64,
    pub(crate) ca_addr_id: String,
    pub(crate) ca_address: Address,
    pub(crate) ca_location_type: String,
}

impl CustomerAddressRow {
    pub fn new(
        null_bit_map: i64,
        ca_addr_sk: i64,
        ca_addr_id: String,
        ca_address: Address,
        ca_location_type: String,
    ) -> Self {
        CustomerAddressRow {
            null_bit_map,
            ca_addr_sk,
            ca_addr_id,
            ca_address,
            ca_location_type,
        }
    }

    /// Check if a column should be null based on the null bitmap (TableRowWithNulls logic)
    pub(crate) fn should_be_null(&self, column_position: i32) -> bool {
        ((self.null_bit_map >> column_position) & 1) == 1
    }

    pub fn null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    pub fn get_ca_addr_sk(&self) -> i64 {
        self.ca_addr_sk
    }

    pub fn get_ca_addr_id(&self) -> &str {
        &self.ca_addr_id
    }

    pub fn get_ca_address(&self) -> &Address {
        &self.ca_address
    }

    pub fn get_ca_location_type(&self) -> &str {
        &self.ca_location_type
    }
}

/// DAT field helper: NULL is driven purely by the null bit
/// (customer_address applies no key sentinel check, only the null bit).
impl CustomerAddressRow {
    pub(crate) fn field<T>(&self, value: T, column_position: i32) -> DatField<T> {
        DatField::new(value, self.should_be_null(column_position))
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces one
/// `|`-terminated field per column.
impl fmt::Display for CustomerAddressRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|",
            self.field(self.ca_addr_sk, 0),
            self.field(&self.ca_addr_id, 1),
            self.field(self.ca_address.get_street_number(), 2),
            self.field(self.ca_address.get_street_name(), 3),
            self.field(self.ca_address.get_street_type(), 4),
            self.field(self.ca_address.get_suite_number(), 5),
            self.field(self.ca_address.get_city(), 6),
            self.field(self.ca_address.get_county().unwrap_or(""), 7),
            self.field(self.ca_address.get_state(), 8),
            DatField::zip(self.ca_address.get_zip(), self.should_be_null(9)),
            self.field(self.ca_address.get_country(), 10),
            self.field(self.ca_address.get_gmt_offset(), 11),
            self.field(&self.ca_location_type, 12),
        )
    }
}
