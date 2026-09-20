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

//! Store row structure and formatting

use crate::generator::{GeneratorColumn, StoreGeneratorColumn};
use crate::row::table_row::DatField;
use crate::types::{Address, Date, Decimal};
use std::fmt;

/// Store row
#[derive(Clone)]
pub struct StoreRow {
    null_bit_map: i64,
    pub(crate) store_sk: i64,
    pub(crate) store_id: String,
    pub(crate) rec_start_date_id: i64,
    pub(crate) rec_end_date_id: i64,
    pub(crate) closed_date_id: i64,
    pub(crate) store_name: String,
    pub(crate) employees: i32,
    pub(crate) floor_space: i32,
    pub(crate) hours: String,
    pub(crate) store_manager: String,
    pub(crate) market_id: i32,
    pub(crate) d_tax_percentage: Decimal,
    pub(crate) geography_class: String,
    pub(crate) market_desc: String,
    pub(crate) market_manager: String,
    pub(crate) division_id: i64,
    pub(crate) division_name: String,
    pub(crate) company_id: i64,
    pub(crate) company_name: String,
    pub(crate) address: Address,
}

impl StoreRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        null_bit_map: i64,
        store_sk: i64,
        store_id: String,
        rec_start_date_id: i64,
        rec_end_date_id: i64,
        closed_date_id: i64,
        store_name: String,
        employees: i32,
        floor_space: i32,
        hours: String,
        store_manager: String,
        market_id: i32,
        d_tax_percentage: Decimal,
        geography_class: String,
        market_desc: String,
        market_manager: String,
        division_id: i64,
        division_name: String,
        company_id: i64,
        company_name: String,
        address: Address,
    ) -> Self {
        StoreRow {
            null_bit_map,
            store_sk,
            store_id,
            rec_start_date_id,
            rec_end_date_id,
            closed_date_id,
            store_name,
            employees,
            floor_space,
            hours,
            store_manager,
            market_id,
            d_tax_percentage,
            geography_class,
            market_desc,
            market_manager,
            division_id,
            division_name,
            company_id,
            company_name,
            address,
        }
    }

    pub(crate) fn is_null(&self, column: &StoreGeneratorColumn) -> bool {
        let bit_position = column.get_global_column_number()
            - StoreGeneratorColumn::WStoreSk.get_global_column_number();
        (self.null_bit_map & (1 << bit_position)) != 0
    }

    // Getters for SCD fields
    pub fn get_closed_date_id(&self) -> i64 {
        self.closed_date_id
    }

    pub fn get_store_name(&self) -> &str {
        &self.store_name
    }

    pub fn get_employees(&self) -> i32 {
        self.employees
    }

    pub fn get_floor_space(&self) -> i32 {
        self.floor_space
    }

    pub fn get_hours(&self) -> &str {
        &self.hours
    }

    pub fn get_store_manager(&self) -> &str {
        &self.store_manager
    }

    pub fn get_market_id(&self) -> i32 {
        self.market_id
    }

    pub fn get_d_tax_percentage(&self) -> Decimal {
        self.d_tax_percentage
    }

    pub fn get_market_desc(&self) -> &str {
        &self.market_desc
    }

    pub fn get_market_manager(&self) -> &str {
        &self.market_manager
    }

    pub fn get_address(&self) -> &Address {
        &self.address
    }

    pub fn null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    pub fn get_store_sk(&self) -> i64 {
        self.store_sk
    }

    pub fn get_store_id(&self) -> &str {
        &self.store_id
    }

    pub fn get_rec_start_date_id(&self) -> i64 {
        self.rec_start_date_id
    }

    pub fn get_rec_end_date_id(&self) -> i64 {
        self.rec_end_date_id
    }

    pub fn get_geography_class(&self) -> &str {
        &self.geography_class
    }

    pub fn get_division_id(&self) -> i64 {
        self.division_id
    }

    pub fn get_division_name(&self) -> &str {
        &self.division_name
    }

    pub fn get_company_id(&self) -> i64 {
        self.company_id
    }

    pub fn get_company_name(&self) -> &str {
        &self.company_name
    }
}

impl StoreRow {
    /// DAT field for a surrogate key: NULL when the null bit is set or the
    /// key is negative.
    pub(crate) fn key_field(&self, value: i64, column: &StoreGeneratorColumn) -> DatField<i64> {
        DatField::new(value, self.is_null(column) || value < 0)
    }

    /// DAT field for a regular value: NULL when the null bit is set.
    pub(crate) fn field<T>(&self, value: T, column: &StoreGeneratorColumn) -> DatField<T> {
        DatField::new(value, self.is_null(column))
    }

    /// DAT field for an SCD date: NULL when the null bit is set or the
    /// julian day is negative.
    pub(crate) fn date_field(
        &self,
        julian_days: i64,
        column: &StoreGeneratorColumn,
    ) -> DatField<Date> {
        DatField::from(
            (!(self.is_null(column) || julian_days < 0))
                .then(|| Date::from_julian_days(julian_days as i32)),
        )
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces one
/// `|`-terminated field per column.
impl fmt::Display for StoreRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use StoreGeneratorColumn::*;

        write!(
            f,
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|",
            self.key_field(self.store_sk, &WStoreSk),
            self.field(&self.store_id, &WStoreId),
            self.date_field(self.rec_start_date_id, &WStoreRecStartDateId),
            self.date_field(self.rec_end_date_id, &WStoreRecEndDateId),
            self.key_field(self.closed_date_id, &WStoreClosedDateId),
            self.field(&self.store_name, &WStoreName),
            self.field(self.employees, &WStoreEmployees),
            self.field(self.floor_space, &WStoreFloorSpace),
            self.field(&self.hours, &WStoreHours),
            self.field(&self.store_manager, &WStoreManager),
            self.field(self.market_id, &WStoreMarketId),
            self.field(&self.geography_class, &WStoreGeographyClass),
            self.field(&self.market_desc, &WStoreMarketDesc),
            self.field(&self.market_manager, &WStoreMarketManager),
            self.key_field(self.division_id, &WStoreDivisionId),
            self.field(&self.division_name, &WStoreDivisionName),
            self.key_field(self.company_id, &WStoreCompanyId),
            self.field(&self.company_name, &WStoreCompanyName),
            self.field(self.address.get_street_number(), &WStoreAddressStreetNum),
            self.field(self.address.get_street_name(), &WStoreAddressStreetName1),
            self.field(self.address.get_street_type(), &WStoreAddressStreetType),
            self.field(self.address.get_suite_number(), &WStoreAddressSuiteNum),
            self.field(self.address.get_city(), &WStoreAddressCity),
            self.field(self.address.get_county().unwrap_or(""), &WStoreAddressCounty),
            self.field(self.address.get_state(), &WStoreAddressState),
            DatField::zip(self.address.get_zip(), self.is_null(&WStoreAddressZip)),
            self.field(self.address.get_country(), &WStoreAddressCountry),
            self.field(self.address.get_gmt_offset(), &WStoreAddressGmtOffset),
            self.field(self.d_tax_percentage, &WStoreTaxPercentage),
        )
    }
}
