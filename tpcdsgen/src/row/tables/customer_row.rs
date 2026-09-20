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

//! Customer row definition (CustomerRow)

use crate::generator::CustomerGeneratorColumn;
use crate::row::table_row::DatField;
use std::fmt;

/// Customer row (CustomerRow)
#[derive(Debug, Clone)]
pub struct CustomerRow {
    pub(crate) null_bit_map: i64,
    pub(crate) c_customer_sk: i64,
    pub(crate) c_customer_id: String,
    pub(crate) c_current_cdemo_sk: i64,
    pub(crate) c_current_hdemo_sk: i64,
    pub(crate) c_current_addr_sk: i64,
    pub(crate) c_first_shipto_date_id: i32,
    pub(crate) c_first_sales_date_id: i32,
    pub(crate) c_salutation: String,
    pub(crate) c_first_name: String,
    pub(crate) c_last_name: String,
    pub(crate) c_preferred_cust_flag: bool,
    pub(crate) c_birth_day: i32,
    pub(crate) c_birth_month: i32,
    pub(crate) c_birth_year: i32,
    pub(crate) c_birth_country: String,
    pub(crate) c_login: Option<String>, // always null in the Java implementation
    pub(crate) c_email_address: String,
    pub(crate) c_last_review_date: i32,
}

impl CustomerRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        c_customer_sk: i64,
        c_customer_id: String,
        c_current_cdemo_sk: i64,
        c_current_hdemo_sk: i64,
        c_current_addr_sk: i64,
        c_first_shipto_date_id: i32,
        c_first_sales_date_id: i32,
        c_salutation: String,
        c_first_name: String,
        c_last_name: String,
        c_preferred_cust_flag: bool,
        c_birth_day: i32,
        c_birth_month: i32,
        c_birth_year: i32,
        c_birth_country: String,
        c_email_address: String,
        c_last_review_date: i32,
        null_bit_map: i64,
    ) -> Self {
        CustomerRow {
            null_bit_map,
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
            c_login: None, // never gets set to anything
            c_email_address,
            c_last_review_date,
        }
    }

    pub fn null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    pub fn get_c_customer_sk(&self) -> i64 {
        self.c_customer_sk
    }

    pub fn get_c_customer_id(&self) -> &str {
        &self.c_customer_id
    }

    pub fn get_c_current_cdemo_sk(&self) -> i64 {
        self.c_current_cdemo_sk
    }

    pub fn get_c_current_hdemo_sk(&self) -> i64 {
        self.c_current_hdemo_sk
    }

    pub fn get_c_current_addr_sk(&self) -> i64 {
        self.c_current_addr_sk
    }

    pub fn get_c_first_shipto_date_id(&self) -> i32 {
        self.c_first_shipto_date_id
    }

    pub fn get_c_first_sales_date_id(&self) -> i32 {
        self.c_first_sales_date_id
    }

    pub fn get_c_salutation(&self) -> &str {
        &self.c_salutation
    }

    pub fn get_c_first_name(&self) -> &str {
        &self.c_first_name
    }

    pub fn get_c_last_name(&self) -> &str {
        &self.c_last_name
    }

    pub fn get_c_preferred_cust_flag(&self) -> bool {
        self.c_preferred_cust_flag
    }

    pub fn get_c_birth_day(&self) -> i32 {
        self.c_birth_day
    }

    pub fn get_c_birth_month(&self) -> i32 {
        self.c_birth_month
    }

    pub fn get_c_birth_year(&self) -> i32 {
        self.c_birth_year
    }

    pub fn get_c_birth_country(&self) -> &str {
        &self.c_birth_country
    }

    pub fn get_c_login(&self) -> Option<&str> {
        self.c_login.as_deref()
    }

    pub fn get_c_email_address(&self) -> &str {
        &self.c_email_address
    }

    pub fn get_c_last_review_date(&self) -> i32 {
        self.c_last_review_date
    }

    /// Check if a column is null based on the null bit map
    pub(crate) fn is_null(&self, column: CustomerGeneratorColumn) -> bool {
        let position = column.get_global_column_number()
            - CustomerGeneratorColumn::CCustomerSk.get_global_column_number();
        (self.null_bit_map & (1 << position)) != 0
    }
}

impl CustomerRow {
    /// DAT field for a surrogate key: NULL when the null bit is set or the
    /// key is negative.
    pub(crate) fn key_field(&self, value: i64, column: CustomerGeneratorColumn) -> DatField<i64> {
        DatField::new(value, self.is_null(column) || value < 0)
    }

    /// DAT field for a regular value: NULL when the null bit is set.
    pub(crate) fn field<T>(&self, value: T, column: CustomerGeneratorColumn) -> DatField<T> {
        DatField::new(value, self.is_null(column))
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces one
/// `|`-terminated field per column.
impl fmt::Display for CustomerRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CustomerGeneratorColumn::*;

        write!(
            f,
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|",
            self.key_field(self.c_customer_sk, CCustomerSk),
            self.field(&self.c_customer_id, CCustomerId),
            self.key_field(self.c_current_cdemo_sk, CCurrentCdemoSk),
            self.key_field(self.c_current_hdemo_sk, CCurrentHdemoSk),
            self.key_field(self.c_current_addr_sk, CCurrentAddrSk),
            self.field(self.c_first_shipto_date_id, CFirstShiptoDateId),
            self.field(self.c_first_sales_date_id, CFirstSalesDateId),
            self.field(&self.c_salutation, CSalutation),
            self.field(&self.c_first_name, CFirstName),
            self.field(&self.c_last_name, CLastName),
            DatField::yes_no(self.c_preferred_cust_flag, self.is_null(CPreferredCustFlag)),
            self.field(self.c_birth_day, CBirthDay),
            self.field(self.c_birth_month, CBirthMonth),
            self.field(self.c_birth_year, CBirthYear),
            self.field(&self.c_birth_country, CBirthCountry),
            // c_login is emitted without a null check, matching Java
            self.c_login.as_deref().unwrap_or_default(),
            self.field(&self.c_email_address, CEmailAddress),
            self.field(self.c_last_review_date, CLastReviewDate),
        )
    }
}

use crate::generator::GeneratorColumn;
