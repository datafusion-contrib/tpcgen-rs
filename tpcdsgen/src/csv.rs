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

//! CSV formatting support for TPC-DS
//!
//! Each table row type gets a `<Table>Csv` wrapper whose `Display` impl emits
//! one CSV line
//!
//! # Character encoding
//!
//! CSV output is always UTF-8 (while The DAT output can optionally be encoded
//! as ISO-8859-1 in [`CompatMode::Trino`] (see [`crate::output::DatWriter`]);
//!
//! [`GeneratedRowCsv`] wraps the [`GeneratedRow`] enum for callers that work
//! with rows generically, and [`csv_header`] returns the header line for a
//! [`Table`].
//!
//! [`CompatMode::Trino`]: crate::config::CompatMode::Trino

use crate::config::Table;
use crate::row::dbgen_version_row::TimeOfDay;
use crate::row::table_row::{CsvQuoted, CsvQuotedNullLiteral, DatField, NullLiteralField};
use crate::row::{
    CallCenterRow, CatalogPageRow, CatalogReturnsRow, CatalogSalesRow, CustomerAddressRow,
    CustomerDemographicsRow, CustomerRow, DateDimRow, DbgenVersionRow, GeneratedRow,
    HouseholdDemographicsRow, IncomeBandRow, InventoryRow, ItemRow, PromotionRow, ReasonRow,
    ShipModeRow, StoreReturnsRow, StoreRow, StoreSalesRow, TimeDimRow, WarehouseRow, WebPageRow,
    WebReturnsRow, WebSalesRow, WebSiteRow,
};
use std::fmt::{self, Display};

/// Writes [`CallCenterRow`]s in CSV format.
///
/// # Example
/// ```
/// # use tpcdsgen::config::Session;
/// # use tpcdsgen::csv::CallCenterCsv;
/// # use tpcdsgen::row::{GeneratedRow, RowGenerator, CallCenterRowGenerator};
/// # use std::fmt::Write;
/// # let session = Session::default();
/// let mut generator = CallCenterRowGenerator::new();
/// // Output the first 3 rows in CSV format
/// let mut csv = String::new();
/// writeln!(&mut csv, "{}", CallCenterCsv::header()).unwrap(); // write header
/// # for row_number in 1..=3 {
/// #   let result = generator.generate_row_and_child_rows(row_number, &session, None, None).unwrap();
/// #   for row in result.get_rows() {
/// #     let GeneratedRow::CallCenter(row) = row else { continue };
/// // write line using CSV formatter
/// writeln!(&mut csv, "{}", CallCenterCsv::new(row)).unwrap();
/// #   }
/// #   generator.consume_remaining_seeds_for_row();
/// # }
/// assert_eq!(
///   csv,
///   "cc_call_center_sk,cc_call_center_id,cc_rec_start_date,cc_rec_end_date,cc_closed_date_sk,cc_open_date_sk,cc_name,cc_class,cc_employees,cc_sq_ft,cc_hours,cc_manager,cc_mkt_id,cc_mkt_class,cc_mkt_desc,cc_market_manager,cc_division,cc_division_name,cc_company,cc_company_name,cc_street_number,cc_street_name,cc_street_type,cc_suite_number,cc_city,cc_county,cc_state,cc_zip,cc_country,cc_gmt_offset,cc_tax_percentage\n\
///    1,AAAAAAAABAAAAAAA,1998-01-01,,,2450952,NY Metro,large,2,1138,8AM-4PM,Bob Belcher,6,\"More than other authori\",\"Shared others could not count fully dollars. New members ca\",Julius Tran,3,pri,6,cally,730,Ash Hill,Boulevard,Suite 0,Midway,Williamson County,TN,31904,United States,-5,0.11\n\
///    2,AAAAAAAACAAAAAAA,1998-01-01,2000-12-31,,2450806,Mid Atlantic,medium,6,2268,8AM-8AM,Felipe Perkins,2,\"A bit narrow forms matter animals. Consist\",\"Largely blank years put substantially deaf, new others. Question\",Julius Durham,5,anti,1,ought,984,Center Hill,Way,Suite 70,Midway,Williamson County,TN,31904,United States,-5,0.12\n\
///    3,AAAAAAAACAAAAAAA,2001-01-01,,,2450806,Mid Atlantic,medium,6,4134,8AM-4PM,Mark Hightower,2,\"Wrong troops shall work sometimes in a opti\",\"Largely blank years put substantially deaf, new others. Question\",Julius Durham,1,ought,2,able,984,Center Hill,Way,Suite 70,Midway,Williamson County,TN,31904,United States,-5,0.01\n"
/// );
/// ```
pub struct CallCenterCsv<'a> {
    inner: &'a CallCenterRow,
    delimiter: char,
}

impl<'a> CallCenterCsv<'a> {
    /// Create a wrapper that formats `inner` with the default `,` delimiter
    pub fn new(inner: &'a CallCenterRow) -> Self {
        Self {
            inner,
            delimiter: ',',
        }
    }

    /// Create a wrapper that formats `inner` with a custom delimiter
    pub fn with_delimiter(inner: &'a CallCenterRow, delimiter: char) -> Self {
        Self { inner, delimiter }
    }

    /// Returns the CSV header for the call_center table
    pub fn header() -> &'static str {
        "cc_call_center_sk,cc_call_center_id,cc_rec_start_date,cc_rec_end_date,cc_closed_date_sk,cc_open_date_sk,cc_name,cc_class,cc_employees,cc_sq_ft,cc_hours,cc_manager,cc_mkt_id,cc_mkt_class,cc_mkt_desc,cc_market_manager,cc_division,cc_division_name,cc_company,cc_company_name,cc_street_number,cc_street_name,cc_street_type,cc_suite_number,cc_city,cc_county,cc_state,cc_zip,cc_country,cc_gmt_offset,cc_tax_percentage"
    }

    /// Returns the CSV header with a custom delimiter
    pub fn header_with_delimiter(delimiter: char) -> String {
        join_header(Self::header(), delimiter)
    }
}

impl Display for CallCenterCsv<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = self.delimiter;
        let row = self.inner;
        write!(
            f,
            "{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}",
            row.nulled(row.cc_call_center_sk, 0),
            row.nulled(&row.cc_call_center_id, 1),
            row.date_field(row.cc_rec_start_date_id, 2),
            row.date_field(row.cc_rec_end_date_id, 3),
            row.key_field(row.cc_closed_date_id, 4),
            row.key_field(row.cc_open_date_id, 5),
            row.nulled(&row.cc_name, 6),
            row.nulled(&row.cc_class, 7),
            row.nulled(row.cc_employees, 8),
            row.nulled(row.cc_sq_ft, 9),
            row.nulled(&row.cc_hours, 10),
            row.nulled(&row.cc_manager, 11),
            row.nulled(row.cc_market_id, 12),
            CsvQuotedNullLiteral::new(&row.cc_market_class, row.is_null(13)),
            CsvQuotedNullLiteral::new(&row.cc_market_desc, row.is_null(14)),
            row.nulled(&row.cc_market_manager, 15),
            row.nulled(row.cc_division_id, 16),
            row.nulled(&row.cc_division_name, 17),
            row.nulled(row.cc_company, 18),
            row.nulled(&row.cc_company_name, 19),
            row.nulled(row.cc_address.get_street_number(), 20),
            row.nulled(row.cc_address.get_street_name(), 21),
            row.nulled(row.cc_address.get_street_type(), 22),
            row.nulled(row.cc_address.get_suite_number(), 23),
            row.nulled(row.cc_address.get_city(), 24),
            row.nulled(row.cc_address.get_county().unwrap_or(""), 25),
            row.nulled(row.cc_address.get_state(), 26),
            // Note: unlike other tables the call_center zip is not zero-padded,
            // matching format_numeric in get_values.
            row.nulled(row.cc_address.get_zip(), 27),
            row.nulled(row.cc_address.get_country(), 28),
            row.nulled(row.cc_address.get_gmt_offset(), 29),
            row.nulled(row.cc_tax_percentage, 30),
        )
    }
}
/// Writes [`CatalogPageRow`]s in CSV format.
///
/// # Example
/// ```
/// # use tpcdsgen::config::Session;
/// # use tpcdsgen::csv::CatalogPageCsv;
/// # use tpcdsgen::row::{GeneratedRow, RowGenerator, CatalogPageRowGenerator};
/// # use std::fmt::Write;
/// # let session = Session::default();
/// let mut generator = CatalogPageRowGenerator::new();
/// // Output the first 3 rows in CSV format
/// let mut csv = String::new();
/// writeln!(&mut csv, "{}", CatalogPageCsv::header()).unwrap(); // write header
/// # for row_number in 1..=3 {
/// #   let result = generator.generate_row_and_child_rows(row_number, &session, None, None).unwrap();
/// #   for row in result.get_rows() {
/// #     let GeneratedRow::CatalogPage(row) = row else { continue };
/// // write line using CSV formatter
/// writeln!(&mut csv, "{}", CatalogPageCsv::new(row)).unwrap();
/// #   }
/// #   generator.consume_remaining_seeds_for_row();
/// # }
/// assert_eq!(
///   csv,
///   "cp_catalog_page_sk,cp_catalog_page_id,cp_start_date_sk,cp_end_date_sk,cp_department,cp_catalog_number,cp_catalog_page_number,cp_description,cp_type\n\
///    1,AAAAAAAABAAAAAAA,2450815,2450996,DEPARTMENT,1,1,\"In general basic characters welcome. Clearly lively friends conv\",bi-annual\n\
///    2,AAAAAAAACAAAAAAA,2450815,2450996,DEPARTMENT,1,2,\"English areas will leave prisoners. Too public countries ought to become beneath the years. \",bi-annual\n\
///    3,AAAAAAAADAAAAAAA,2450815,2450996,DEPARTMENT,1,3,\"Times could not address disabled indians. Effectively public ports c\",bi-annual\n"
/// );
/// ```
pub struct CatalogPageCsv<'a> {
    inner: &'a CatalogPageRow,
    delimiter: char,
}

impl<'a> CatalogPageCsv<'a> {
    /// Create a wrapper that formats `inner` with the default `,` delimiter
    pub fn new(inner: &'a CatalogPageRow) -> Self {
        Self {
            inner,
            delimiter: ',',
        }
    }

    /// Create a wrapper that formats `inner` with a custom delimiter
    pub fn with_delimiter(inner: &'a CatalogPageRow, delimiter: char) -> Self {
        Self { inner, delimiter }
    }

    /// Returns the CSV header for the catalog_page table
    pub fn header() -> &'static str {
        "cp_catalog_page_sk,cp_catalog_page_id,cp_start_date_sk,cp_end_date_sk,cp_department,cp_catalog_number,cp_catalog_page_number,cp_description,cp_type"
    }

    /// Returns the CSV header with a custom delimiter
    pub fn header_with_delimiter(delimiter: char) -> String {
        join_header(Self::header(), delimiter)
    }
}

impl Display for CatalogPageCsv<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::generator::CatalogPageGeneratorColumn::*;

        let d = self.delimiter;
        let row = self.inner;

        write!(
            f,
            "{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}",
            row.key_field(row.cp_catalog_page_sk, &CpCatalogPageSk),
            row.field(&row.cp_catalog_page_id, &CpCatalogPageId),
            row.key_field(row.cp_start_date_id, &CpStartDateId),
            row.key_field(row.cp_end_date_id, &CpEndDateId),
            row.field(&row.cp_department, &CpDepartment),
            row.field(row.cp_catalog_number, &CpCatalogNumber),
            row.field(row.cp_catalog_page_number, &CpCatalogPageNumber),
            CsvQuoted::new(&row.cp_description, row.is_null(&CpDescription)),
            row.field(&row.cp_type, &CpType),
        )
    }
}
/// Writes [`CatalogReturnsRow`]s in CSV format.
///
/// Not every catalog sales row has a matching return, so the loop below runs
/// over more row numbers than the three returns rows it produces.
///
/// # Example
/// ```
/// # use tpcdsgen::config::Session;
/// # use tpcdsgen::csv::CatalogReturnsCsv;
/// # use tpcdsgen::row::{GeneratedRow, RowGenerator, CatalogSalesRowGenerator};
/// # use std::fmt::Write;
/// # let session = Session::default();
/// let mut generator = CatalogSalesRowGenerator::new();
/// // Output the first 3 rows in CSV format
/// let mut csv = String::new();
/// writeln!(&mut csv, "{}", CatalogReturnsCsv::header()).unwrap(); // write header
/// # for row_number in 1..=30 {
/// #   let result = generator.generate_row_and_child_rows(row_number, &session, None, None).unwrap();
/// #   for row in result.get_rows() {
/// #     let GeneratedRow::CatalogReturns(row) = row else { continue };
/// // write line using CSV formatter
/// writeln!(&mut csv, "{}", CatalogReturnsCsv::new(row)).unwrap();
/// #   }
/// #   if result.should_end_row() {
/// #     generator.consume_remaining_seeds_for_row();
/// #   }
/// # }
/// assert_eq!(
///   csv,
///   "cr_returned_date_sk,cr_returned_time_sk,cr_item_sk,cr_refunded_customer_sk,cr_refunded_cdemo_sk,cr_refunded_hdemo_sk,cr_refunded_addr_sk,cr_returning_customer_sk,cr_returning_cdemo_sk,cr_returning_hdemo_sk,cr_returning_addr_sk,cr_call_center_sk,cr_catalog_page_sk,cr_ship_mode_sk,cr_warehouse_sk,cr_reason_sk,cr_order_number,cr_return_quantity,cr_return_amount,cr_return_tax,cr_return_amt_inc_tax,cr_fee,cr_return_ship_cost,cr_refunded_cash,cr_reversed_charge,cr_store_credit,cr_net_loss\n\
///    2450926,45816,17368,14601,797995,6189,9583,14601,797995,4703,9583,1,106,2,2,30,5,47,3888.31,233.29,4121.60,91.23,1348.90,3577.24,186.64,124.43,1673.42\n\
///    2450946,74710,6295,14601,797995,6189,9583,82809,665550,991,14832,1,17,2,5,6,5,49,2490.18,99.60,2589.78,52.54,1867.39,323.72,931.57,1234.89,2019.53\n\
///    2451065,71104,3391,25383,3755,2480,5652,2311,700704,5571,12485,4,7,13,2,1,26,12,64.32,4.50,68.82,22.97,78.60,1.28,55.47,7.57,106.07\n"
/// );
/// ```
pub struct CatalogReturnsCsv<'a> {
    inner: &'a CatalogReturnsRow,
    delimiter: char,
}

impl<'a> CatalogReturnsCsv<'a> {
    /// Create a wrapper that formats `inner` with the default `,` delimiter
    pub fn new(inner: &'a CatalogReturnsRow) -> Self {
        Self {
            inner,
            delimiter: ',',
        }
    }

    /// Create a wrapper that formats `inner` with a custom delimiter
    pub fn with_delimiter(inner: &'a CatalogReturnsRow, delimiter: char) -> Self {
        Self { inner, delimiter }
    }

    /// Returns the CSV header for the catalog_returns table
    pub fn header() -> &'static str {
        "cr_returned_date_sk,cr_returned_time_sk,cr_item_sk,cr_refunded_customer_sk,cr_refunded_cdemo_sk,cr_refunded_hdemo_sk,cr_refunded_addr_sk,cr_returning_customer_sk,cr_returning_cdemo_sk,cr_returning_hdemo_sk,cr_returning_addr_sk,cr_call_center_sk,cr_catalog_page_sk,cr_ship_mode_sk,cr_warehouse_sk,cr_reason_sk,cr_order_number,cr_return_quantity,cr_return_amount,cr_return_tax,cr_return_amt_inc_tax,cr_fee,cr_return_ship_cost,cr_refunded_cash,cr_reversed_charge,cr_store_credit,cr_net_loss"
    }

    /// Returns the CSV header with a custom delimiter
    pub fn header_with_delimiter(delimiter: char) -> String {
        join_header(Self::header(), delimiter)
    }
}

impl Display for CatalogReturnsCsv<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::generator::CatalogReturnsGeneratorColumn::*;

        let d = self.delimiter;
        let row = self.inner;

        write!(
            f,
            "{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}",
            row.key_field(row.cr_returned_date_sk, &CrReturnedDateSk),
            row.key_field(row.cr_returned_time_sk, &CrReturnedTimeSk),
            row.key_field(row.cr_item_sk, &CrItemSk),
            row.key_field(row.cr_refunded_customer_sk, &CrRefundedCustomerSk),
            row.key_field(row.cr_refunded_cdemo_sk, &CrRefundedCdemoSk),
            row.key_field(row.cr_refunded_hdemo_sk, &CrRefundedHdemoSk),
            row.key_field(row.cr_refunded_addr_sk, &CrRefundedAddrSk),
            row.key_field(row.cr_returning_customer_sk, &CrReturningCustomerSk),
            row.key_field(row.cr_returning_cdemo_sk, &CrReturningCdemoSk),
            row.key_field(row.cr_returning_hdemo_sk, &CrReturningHdemoSk),
            row.key_field(row.cr_returning_addr_sk, &CrReturningAddrSk),
            row.key_field(row.cr_call_center_sk, &CrCallCenterSk),
            row.key_field(row.cr_catalog_page_sk, &CrCatalogPageSk),
            row.key_field(row.cr_ship_mode_sk, &CrShipModeSk),
            row.key_field(row.cr_warehouse_sk, &CrWarehouseSk),
            row.key_field(row.cr_reason_sk, &CrReasonSk),
            row.field(row.cr_order_number, &CrOrderNumber),
            row.field(row.cr_pricing.get_quantity(), &CrPricingQuantity),
            row.field(row.cr_pricing.get_net_paid(), &CrPricingNetPaid),
            row.field(row.cr_pricing.get_ext_tax(), &CrPricingExtTax),
            row.field(
                row.cr_pricing.get_net_paid_including_tax(),
                &CrPricingNetPaidIncTax
            ),
            row.field(row.cr_pricing.get_fee(), &CrPricingFee),
            row.field(row.cr_pricing.get_ext_ship_cost(), &CrPricingExtShipCost),
            row.field(row.cr_pricing.get_refunded_cash(), &CrPricingRefundedCash),
            row.field(
                row.cr_pricing.get_reversed_charge(),
                &CrPricingReversedCharge
            ),
            row.field(row.cr_pricing.get_store_credit(), &CrPricingStoreCredit),
            row.field(row.cr_pricing.get_net_loss(), &CrPricingNetLoss),
        )
    }
}
/// Writes [`CatalogSalesRow`]s in CSV format.
///
/// # Example
/// ```
/// # use tpcdsgen::config::Session;
/// # use tpcdsgen::csv::CatalogSalesCsv;
/// # use tpcdsgen::row::{GeneratedRow, RowGenerator, CatalogSalesRowGenerator};
/// # use std::fmt::Write;
/// # let session = Session::default();
/// let mut generator = CatalogSalesRowGenerator::new();
/// // Output the first 3 rows in CSV format
/// let mut csv = String::new();
/// writeln!(&mut csv, "{}", CatalogSalesCsv::header()).unwrap(); // write header
/// # for row_number in 1..=3 {
/// #   let result = generator.generate_row_and_child_rows(row_number, &session, None, None).unwrap();
/// #   for row in result.get_rows() {
/// #     let GeneratedRow::CatalogSales(row) = row else { continue };
/// // write line using CSV formatter
/// writeln!(&mut csv, "{}", CatalogSalesCsv::new(row)).unwrap();
/// #   }
/// #   if result.should_end_row() {
/// #     generator.consume_remaining_seeds_for_row();
/// #   }
/// # }
/// assert_eq!(
///   csv,
///   "cs_sold_date_sk,cs_sold_time_sk,cs_ship_date_sk,cs_bill_customer_sk,cs_bill_cdemo_sk,cs_bill_hdemo_sk,cs_bill_addr_sk,cs_ship_customer_sk,cs_ship_cdemo_sk,cs_ship_hdemo_sk,cs_ship_addr_sk,cs_call_center_sk,cs_catalog_page_sk,cs_ship_mode_sk,cs_warehouse_sk,cs_item_sk,cs_promo_sk,cs_order_number,cs_quantity,cs_wholesale_cost,cs_list_price,cs_sales_price,cs_ext_discount_amt,cs_ext_sales_price,cs_ext_wholesale_cost,cs_ext_list_price,cs_ext_tax,cs_coupon_amt,cs_ext_ship_cost,cs_net_paid,cs_net_paid_inc_tax,cs_net_paid_inc_ship,cs_net_paid_inc_ship_tax,cs_net_profit\n\
///    2450815,38212,2450886,62153,1822764,5775,19986,62153,1822764,5775,19986,4,62,3,4,16930,196,1,47,27.70,44.32,42.99,62.51,2020.53,1301.90,2083.04,101.02,0.00,1041.52,2020.53,2121.55,3062.05,3163.07,718.63\n\
///    2450815,38212,2450846,62153,1822764,5775,19986,62153,1822764,5775,19986,4,31,8,2,6020,270,1,20,87.55,260.89,153.92,2139.40,3078.40,1751.00,5217.80,71.41,1292.92,1356.60,1785.48,1856.89,3142.08,3213.49,34.48\n\
///    2450815,38212,2450868,62153,1822764,5775,19986,62153,1822764,5775,19986,4,76,2,2,16198,97,1,19,69.86,88.72,29.27,1129.55,556.13,1327.34,1685.68,33.36,0.00,168.53,556.13,589.49,724.66,758.02,-771.21\n"
/// );
/// ```
pub struct CatalogSalesCsv<'a> {
    inner: &'a CatalogSalesRow,
    delimiter: char,
}

impl<'a> CatalogSalesCsv<'a> {
    /// Create a wrapper that formats `inner` with the default `,` delimiter
    pub fn new(inner: &'a CatalogSalesRow) -> Self {
        Self {
            inner,
            delimiter: ',',
        }
    }

    /// Create a wrapper that formats `inner` with a custom delimiter
    pub fn with_delimiter(inner: &'a CatalogSalesRow, delimiter: char) -> Self {
        Self { inner, delimiter }
    }

    /// Returns the CSV header for the catalog_sales table
    pub fn header() -> &'static str {
        "cs_sold_date_sk,cs_sold_time_sk,cs_ship_date_sk,cs_bill_customer_sk,cs_bill_cdemo_sk,cs_bill_hdemo_sk,cs_bill_addr_sk,cs_ship_customer_sk,cs_ship_cdemo_sk,cs_ship_hdemo_sk,cs_ship_addr_sk,cs_call_center_sk,cs_catalog_page_sk,cs_ship_mode_sk,cs_warehouse_sk,cs_item_sk,cs_promo_sk,cs_order_number,cs_quantity,cs_wholesale_cost,cs_list_price,cs_sales_price,cs_ext_discount_amt,cs_ext_sales_price,cs_ext_wholesale_cost,cs_ext_list_price,cs_ext_tax,cs_coupon_amt,cs_ext_ship_cost,cs_net_paid,cs_net_paid_inc_tax,cs_net_paid_inc_ship,cs_net_paid_inc_ship_tax,cs_net_profit"
    }

    /// Returns the CSV header with a custom delimiter
    pub fn header_with_delimiter(delimiter: char) -> String {
        join_header(Self::header(), delimiter)
    }
}

impl Display for CatalogSalesCsv<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::generator::CatalogSalesGeneratorColumn::*;

        let d = self.delimiter;
        let row = self.inner;

        write!(
            f,
            "{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}",
            row.key_field(row.cs_sold_date_sk, &CsSoldDateSk),
            row.key_field(row.cs_sold_time_sk, &CsSoldTimeSk),
            row.key_field(row.cs_ship_date_sk, &CsShipDateSk),
            row.key_field(row.cs_bill_customer_sk, &CsBillCustomerSk),
            row.key_field(row.cs_bill_cdemo_sk, &CsBillCdemoSk),
            row.key_field(row.cs_bill_hdemo_sk, &CsBillHdemoSk),
            row.key_field(row.cs_bill_addr_sk, &CsBillAddrSk),
            row.key_field(row.cs_ship_customer_sk, &CsShipCustomerSk),
            row.key_field(row.cs_ship_cdemo_sk, &CsShipCdemoSk),
            row.key_field(row.cs_ship_hdemo_sk, &CsShipHdemoSk),
            row.key_field(row.cs_ship_addr_sk, &CsShipAddrSk),
            row.key_field(row.cs_call_center_sk, &CsCallCenterSk),
            row.key_field(row.cs_catalog_page_sk, &CsCatalogPageSk),
            row.key_field(row.cs_ship_mode_sk, &CsShipModeSk),
            row.field(row.cs_warehouse_sk, &CsWarehouseSk),
            row.key_field(row.cs_sold_item_sk, &CsSoldItemSk),
            row.key_field(row.cs_promo_sk, &CsPromoSk),
            row.field(row.cs_order_number, &CsOrderNumber),
            row.field(row.cs_pricing.get_quantity(), &CsPricingQuantity),
            row.field(
                row.cs_pricing.get_wholesale_cost(),
                &CsPricingWholesaleCost
            ),
            row.field(row.cs_pricing.get_list_price(), &CsPricingListPrice),
            row.field(row.cs_pricing.get_sales_price(), &CsPricingSalesPrice),
            row.field(
                row.cs_pricing.get_ext_discount_amount(),
                &CsPricingExtDiscountAmount
            ),
            row.field(
                row.cs_pricing.get_ext_sales_price(),
                &CsPricingExtSalesPrice
            ),
            row.field(
                row.cs_pricing.get_ext_wholesale_cost(),
                &CsPricingExtWholesaleCost
            ),
            row.field(
                row.cs_pricing.get_ext_list_price(),
                &CsPricingExtListPrice
            ),
            row.field(row.cs_pricing.get_ext_tax(), &CsPricingExtTax),
            row.field(row.cs_pricing.get_coupon_amount(), &CsPricingCouponAmt),
            row.field(row.cs_pricing.get_ext_ship_cost(), &CsPricingExtShipCost),
            row.field(row.cs_pricing.get_net_paid(), &CsPricingNetPaid),
            row.field(
                row.cs_pricing.get_net_paid_including_tax(),
                &CsPricingNetPaidIncTax
            ),
            row.field(
                row.cs_pricing.get_net_paid_including_shipping(),
                &CsPricingNetPaidIncShip
            ),
            row.field(
                row.cs_pricing.get_net_paid_including_shipping_and_tax(),
                &CsPricingNetPaidIncShipTax
            ),
            row.field(row.cs_pricing.get_net_profit(), &CsPricingNetProfit),
        )
    }
}
/// Writes [`CustomerRow`]s in CSV format.
///
/// # Example
/// ```
/// # use tpcdsgen::config::Session;
/// # use tpcdsgen::csv::CustomerCsv;
/// # use tpcdsgen::row::{GeneratedRow, RowGenerator, CustomerRowGenerator};
/// # use std::fmt::Write;
/// # let session = Session::default();
/// let mut generator = CustomerRowGenerator::new();
/// // Output the first 3 rows in CSV format
/// let mut csv = String::new();
/// writeln!(&mut csv, "{}", CustomerCsv::header()).unwrap(); // write header
/// # for row_number in 1..=3 {
/// #   let result = generator.generate_row_and_child_rows(row_number, &session, None, None).unwrap();
/// #   for row in result.get_rows() {
/// #     let GeneratedRow::Customer(row) = row else { continue };
/// // write line using CSV formatter
/// writeln!(&mut csv, "{}", CustomerCsv::new(row)).unwrap();
/// #   }
/// #   generator.consume_remaining_seeds_for_row();
/// # }
/// assert_eq!(
///   csv,
///   "c_customer_sk,c_customer_id,c_current_cdemo_sk,c_current_hdemo_sk,c_current_addr_sk,c_first_shipto_date_sk,c_first_sales_date_sk,c_salutation,c_first_name,c_last_name,c_preferred_cust_flag,c_birth_day,c_birth_month,c_birth_year,c_birth_country,c_login,c_email_address,c_last_review_date_sk\n\
///    1,AAAAAAAABAAAAAAA,980124,7135,32946,2452238,2452208,Mr.,Javier,Lewis,Y,9,12,1936,\"CHILE\",,Javier.Lewis@VFAxlnZEvOx.org,2452508\n\
///    2,AAAAAAAACAAAAAAA,819667,1461,31655,2452318,2452288,Dr.,Amy,Moses,Y,9,4,1966,\"TOGO\",,Amy.Moses@Ovk9KjHH.com,2452318\n\
///    3,AAAAAAAADAAAAAAA,1473522,6247,48572,2449130,2449100,Miss,Latisha,Hamilton,N,18,9,1979,\"NIUE\",,Latisha.Hamilton@V.com,2452313\n"
/// );
/// ```
pub struct CustomerCsv<'a> {
    inner: &'a CustomerRow,
    delimiter: char,
}

impl<'a> CustomerCsv<'a> {
    /// Create a wrapper that formats `inner` with the default `,` delimiter
    pub fn new(inner: &'a CustomerRow) -> Self {
        Self {
            inner,
            delimiter: ',',
        }
    }

    /// Create a wrapper that formats `inner` with a custom delimiter
    pub fn with_delimiter(inner: &'a CustomerRow, delimiter: char) -> Self {
        Self { inner, delimiter }
    }

    /// Returns the CSV header for the customer table
    pub fn header() -> &'static str {
        "c_customer_sk,c_customer_id,c_current_cdemo_sk,c_current_hdemo_sk,c_current_addr_sk,c_first_shipto_date_sk,c_first_sales_date_sk,c_salutation,c_first_name,c_last_name,c_preferred_cust_flag,c_birth_day,c_birth_month,c_birth_year,c_birth_country,c_login,c_email_address,c_last_review_date_sk"
    }

    /// Returns the CSV header with a custom delimiter
    pub fn header_with_delimiter(delimiter: char) -> String {
        join_header(Self::header(), delimiter)
    }
}

impl Display for CustomerCsv<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::generator::CustomerGeneratorColumn::*;

        let d = self.delimiter;
        let row = self.inner;

        write!(
            f,
            "{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}",
            row.key_field(row.c_customer_sk, CCustomerSk),
            row.field(&row.c_customer_id, CCustomerId),
            row.key_field(row.c_current_cdemo_sk, CCurrentCdemoSk),
            row.key_field(row.c_current_hdemo_sk, CCurrentHdemoSk),
            row.key_field(row.c_current_addr_sk, CCurrentAddrSk),
            row.field(row.c_first_shipto_date_id, CFirstShiptoDateId),
            row.field(row.c_first_sales_date_id, CFirstSalesDateId),
            row.field(&row.c_salutation, CSalutation),
            row.field(&row.c_first_name, CFirstName),
            row.field(&row.c_last_name, CLastName),
            DatField::yes_no(row.c_preferred_cust_flag, row.is_null(CPreferredCustFlag)),
            row.field(row.c_birth_day, CBirthDay),
            row.field(row.c_birth_month, CBirthMonth),
            row.field(row.c_birth_year, CBirthYear),
            CsvQuoted::new(&row.c_birth_country, row.is_null(CBirthCountry)),
            // c_login is emitted without a null check, like get_values()
            row.c_login.as_deref().unwrap_or_default(),
            row.field(&row.c_email_address, CEmailAddress),
            row.field(row.c_last_review_date, CLastReviewDate),
        )
    }
}
/// Writes [`CustomerAddressRow`]s in CSV format.
///
/// # Example
/// ```
/// # use tpcdsgen::config::Session;
/// # use tpcdsgen::csv::CustomerAddressCsv;
/// # use tpcdsgen::row::{GeneratedRow, RowGenerator, CustomerAddressRowGenerator};
/// # use std::fmt::Write;
/// # let session = Session::default();
/// let mut generator = CustomerAddressRowGenerator::new();
/// // Output the first 3 rows in CSV format
/// let mut csv = String::new();
/// writeln!(&mut csv, "{}", CustomerAddressCsv::header()).unwrap(); // write header
/// # for row_number in 1..=3 {
/// #   let result = generator.generate_row_and_child_rows(row_number, &session, None, None).unwrap();
/// #   for row in result.get_rows() {
/// #     let GeneratedRow::CustomerAddress(row) = row else { continue };
/// // write line using CSV formatter
/// writeln!(&mut csv, "{}", CustomerAddressCsv::new(row)).unwrap();
/// #   }
/// #   generator.consume_remaining_seeds_for_row();
/// # }
/// assert_eq!(
///   csv,
///   "ca_address_sk,ca_address_id,ca_street_number,ca_street_name,ca_street_type,ca_suite_number,ca_city,ca_county,ca_state,ca_zip,ca_country,ca_gmt_offset,ca_location_type\n\
///    1,AAAAAAAABAAAAAAA,18,Jackson ,Parkway,Suite 280,Fairfield,Maricopa County,AZ,86192,United States,-7,condo\n\
///    2,AAAAAAAACAAAAAAA,362,Washington 6th,RD,Suite 80,Fairview,Taos County,NM,85709,United States,-7,condo\n\
///    3,AAAAAAAADAAAAAAA,585,Dogwood Washington,Circle,Suite Q,Pleasant Valley,York County,PA,12477,United States,-5,single family\n"
/// );
/// ```
pub struct CustomerAddressCsv<'a> {
    inner: &'a CustomerAddressRow,
    delimiter: char,
}

impl<'a> CustomerAddressCsv<'a> {
    /// Create a wrapper that formats `inner` with the default `,` delimiter
    pub fn new(inner: &'a CustomerAddressRow) -> Self {
        Self {
            inner,
            delimiter: ',',
        }
    }

    /// Create a wrapper that formats `inner` with a custom delimiter
    pub fn with_delimiter(inner: &'a CustomerAddressRow, delimiter: char) -> Self {
        Self { inner, delimiter }
    }

    /// Returns the CSV header for the customer_address table
    pub fn header() -> &'static str {
        "ca_address_sk,ca_address_id,ca_street_number,ca_street_name,ca_street_type,ca_suite_number,ca_city,ca_county,ca_state,ca_zip,ca_country,ca_gmt_offset,ca_location_type"
    }

    /// Returns the CSV header with a custom delimiter
    pub fn header_with_delimiter(delimiter: char) -> String {
        join_header(Self::header(), delimiter)
    }
}

impl Display for CustomerAddressCsv<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = self.delimiter;
        let row = self.inner;
        write!(
            f,
            "{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}",
            row.field(row.ca_addr_sk, 0),
            row.field(&row.ca_addr_id, 1),
            row.field(row.ca_address.get_street_number(), 2),
            row.field(row.ca_address.get_street_name(), 3),
            row.field(row.ca_address.get_street_type(), 4),
            row.field(row.ca_address.get_suite_number(), 5),
            row.field(row.ca_address.get_city(), 6),
            row.field(row.ca_address.get_county().unwrap_or(""), 7),
            row.field(row.ca_address.get_state(), 8),
            DatField::zip(row.ca_address.get_zip(), row.should_be_null(9)),
            row.field(row.ca_address.get_country(), 10),
            row.field(row.ca_address.get_gmt_offset(), 11),
            row.field(&row.ca_location_type, 12),
        )
    }
}
/// Writes [`CustomerDemographicsRow`]s in CSV format.
///
/// # Example
/// ```
/// # use tpcdsgen::config::Session;
/// # use tpcdsgen::csv::CustomerDemographicsCsv;
/// # use tpcdsgen::row::{CustomerDemographicsRowGenerator, GeneratedRow, RowGenerator};
/// # use std::fmt::Write;
/// # let session = Session::default();
/// let mut generator = CustomerDemographicsRowGenerator::new();
/// // Output the first 3 rows in CSV format
/// let mut csv = String::new();
/// writeln!(&mut csv, "{}", CustomerDemographicsCsv::header()).unwrap(); // write header
/// # for row_number in 1..=3 {
/// #   let result = generator.generate_row_and_child_rows(row_number, &session, None, None).unwrap();
/// #   for row in result.get_rows() {
/// #     let GeneratedRow::CustomerDemographics(row) = row else { continue };
/// // write line using CSV formatter
/// writeln!(&mut csv, "{}", CustomerDemographicsCsv::new(row)).unwrap();
/// #   }
/// #   generator.consume_remaining_seeds_for_row();
/// # }
/// assert_eq!(
///   csv,
///   "cd_demo_sk,cd_gender,cd_marital_status,cd_education_status,cd_purchase_estimate,cd_credit_rating,cd_dep_count,cd_dep_employed_count,cd_dep_college_count\n\
///    1,M,M,Primary,500,Good,0,0,0\n\
///    2,F,M,Primary,500,Good,0,0,0\n\
///    3,M,S,Primary,500,Good,0,0,0\n"
/// );
/// ```
pub struct CustomerDemographicsCsv<'a> {
    inner: &'a CustomerDemographicsRow,
    delimiter: char,
}

impl<'a> CustomerDemographicsCsv<'a> {
    /// Create a wrapper that formats `inner` with the default `,` delimiter
    pub fn new(inner: &'a CustomerDemographicsRow) -> Self {
        Self {
            inner,
            delimiter: ',',
        }
    }

    /// Create a wrapper that formats `inner` with a custom delimiter
    pub fn with_delimiter(inner: &'a CustomerDemographicsRow, delimiter: char) -> Self {
        Self { inner, delimiter }
    }

    /// Returns the CSV header for the customer_demographics table
    pub fn header() -> &'static str {
        "cd_demo_sk,cd_gender,cd_marital_status,cd_education_status,cd_purchase_estimate,cd_credit_rating,cd_dep_count,cd_dep_employed_count,cd_dep_college_count"
    }

    /// Returns the CSV header with a custom delimiter
    pub fn header_with_delimiter(delimiter: char) -> String {
        join_header(Self::header(), delimiter)
    }
}

impl Display for CustomerDemographicsCsv<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = self.delimiter;
        let row = self.inner;
        write!(
            f,
            "{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}",
            row.field(row.cd_demo_sk, 0),
            row.field(&row.cd_gender, 1),
            row.field(&row.cd_marital_status, 2),
            row.field(&row.cd_education_status, 3),
            row.field(row.cd_purchase_estimate, 4),
            row.field(&row.cd_credit_rating, 5),
            row.field(row.cd_dep_count, 6),
            row.field(row.cd_dep_employed_count, 7),
            row.field(row.cd_dep_college_count, 8),
        )
    }
}
/// Writes [`DateDimRow`]s in CSV format.
///
/// # Example
/// ```
/// # use tpcdsgen::config::Session;
/// # use tpcdsgen::csv::DateDimCsv;
/// # use tpcdsgen::row::{DateDimRowGenerator, GeneratedRow, RowGenerator};
/// # use std::fmt::Write;
/// # let session = Session::default();
/// let mut generator = DateDimRowGenerator::new();
/// // Output the first 3 rows in CSV format
/// let mut csv = String::new();
/// writeln!(&mut csv, "{}", DateDimCsv::header()).unwrap(); // write header
/// # for row_number in 1..=3 {
/// #   let result = generator.generate_row_and_child_rows(row_number, &session, None, None).unwrap();
/// #   for row in result.get_rows() {
/// #     let GeneratedRow::DateDim(row) = row else { continue };
/// // write line using CSV formatter
/// writeln!(&mut csv, "{}", DateDimCsv::new(row)).unwrap();
/// #   }
/// #   generator.consume_remaining_seeds_for_row();
/// # }
/// assert_eq!(
///   csv,
///   "d_date_sk,d_date_id,d_date,d_month_seq,d_week_seq,d_quarter_seq,d_year,d_dow,d_moy,d_dom,d_qoy,d_fy_year,d_fy_quarter_seq,d_fy_week_seq,d_day_name,d_quarter_name,d_holiday,d_weekend,d_following_holiday,d_first_dom,d_last_dom,d_same_day_ly,d_same_day_lq,d_current_day,d_current_week,d_current_month,d_current_quarter,d_current_year\n\
///    2415022,AAAAAAAAOKJNECAA,1900-01-02,0,1,1,1900,1,1,2,1,1900,1,1,Monday,1900Q1,N,N,Y,2415021,2415020,2414657,2414930,N,N,N,N,N\n\
///    2415023,AAAAAAAAPKJNECAA,1900-01-03,0,1,1,1900,2,1,3,1,1900,1,1,Tuesday,1900Q1,N,N,N,2415021,2415020,2414658,2414931,N,N,N,N,N\n\
///    2415024,AAAAAAAAALJNECAA,1900-01-04,0,1,1,1900,3,1,4,1,1900,1,1,Wednesday,1900Q1,N,N,N,2415021,2415020,2414659,2414932,N,N,N,N,N\n"
/// );
/// ```
pub struct DateDimCsv<'a> {
    inner: &'a DateDimRow,
    delimiter: char,
}

impl<'a> DateDimCsv<'a> {
    /// Create a wrapper that formats `inner` with the default `,` delimiter
    pub fn new(inner: &'a DateDimRow) -> Self {
        Self {
            inner,
            delimiter: ',',
        }
    }

    /// Create a wrapper that formats `inner` with a custom delimiter
    pub fn with_delimiter(inner: &'a DateDimRow, delimiter: char) -> Self {
        Self { inner, delimiter }
    }

    /// Returns the CSV header for the date_dim table
    pub fn header() -> &'static str {
        "d_date_sk,d_date_id,d_date,d_month_seq,d_week_seq,d_quarter_seq,d_year,d_dow,d_moy,d_dom,d_qoy,d_fy_year,d_fy_quarter_seq,d_fy_week_seq,d_day_name,d_quarter_name,d_holiday,d_weekend,d_following_holiday,d_first_dom,d_last_dom,d_same_day_ly,d_same_day_lq,d_current_day,d_current_week,d_current_month,d_current_quarter,d_current_year"
    }

    /// Returns the CSV header with a custom delimiter
    pub fn header_with_delimiter(delimiter: char) -> String {
        join_header(Self::header(), delimiter)
    }
}

impl Display for DateDimCsv<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = self.delimiter;
        let row = self.inner;
        write!(
            f,
            "{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}",
            row.field(row.d_date_sk, 0),
            row.field(&row.d_date_id, 1),
            row.field(&row.d_date, 2),
            row.field(row.d_month_seq, 3),
            row.field(row.d_week_seq, 4),
            row.field(row.d_quarter_seq, 5),
            row.field(row.d_year, 6),
            row.field(row.d_dow, 7),
            row.field(row.d_moy, 8),
            row.field(row.d_dom, 9),
            row.field(row.d_qoy, 10),
            row.field(row.d_fy_year, 11),
            row.field(row.d_fy_quarter_seq, 12),
            row.field(row.d_fy_week_seq, 13),
            row.field(&row.d_day_name, 14),
            row.field(&row.d_quarter_name, 15),
            row.field(DateDimRow::format_boolean(row.d_holiday), 16),
            row.field(DateDimRow::format_boolean(row.d_weekend), 17),
            row.field(DateDimRow::format_boolean(row.d_following_holiday), 18),
            row.field(row.d_first_dom, 19),
            row.field(row.d_last_dom, 20),
            row.field(row.d_same_day_ly, 21),
            row.field(row.d_same_day_lq, 22),
            row.field(DateDimRow::format_boolean(row.d_current_day), 23),
            row.field(DateDimRow::format_boolean(row.d_current_week), 24),
            row.field(DateDimRow::format_boolean(row.d_current_month), 25),
            row.field(DateDimRow::format_boolean(row.d_current_quarter), 26),
            row.field(DateDimRow::format_boolean(row.d_current_year), 27),
        )
    }
}
/// Writes [`DbgenVersionRow`]s in CSV format.
///
/// # Example
///
/// Note the `dbgen_version` row records the time the data was generated and
/// the command line used, neither of which is reproducible, so this example
/// checks only the stable parts of the line.
/// ```
/// # use tpcdsgen::config::Session;
/// # use tpcdsgen::csv::DbgenVersionCsv;
/// # use tpcdsgen::row::{DbgenVersionRowGenerator, GeneratedRow, RowGenerator};
/// # use std::fmt::Write;
/// # let session = Session::default();
/// let mut generator = DbgenVersionRowGenerator::new();
/// // Output the first row in CSV format
/// let mut csv = String::new();
/// writeln!(&mut csv, "{}", DbgenVersionCsv::header()).unwrap(); // write header
/// # for row_number in 1..=1 {
/// #   let result = generator.generate_row_and_child_rows(row_number, &session, None, None).unwrap();
/// #   for row in result.get_rows() {
/// #     let GeneratedRow::DbgenVersion(row) = row else { continue };
/// // write line using CSV formatter
/// writeln!(&mut csv, "{}", DbgenVersionCsv::new(row)).unwrap();
/// #   }
/// #   generator.consume_remaining_seeds_for_row();
/// # }
/// let mut lines = csv.lines();
/// assert_eq!(
///   lines.next().unwrap(),
///   "dv_version,dv_create_date,dv_create_time,dv_cmdline_args"
/// );
/// // the version is stable, but the creation date/time and command line are not
/// let data = lines.next().unwrap();
/// assert!(data.starts_with("2.0.0,"), "unexpected line: {data}");
/// assert_eq!(data.split(',').count(), 4, "unexpected line: {data}");
/// assert!(lines.next().is_none());
/// ```
pub struct DbgenVersionCsv<'a> {
    inner: &'a DbgenVersionRow,
    delimiter: char,
}

impl<'a> DbgenVersionCsv<'a> {
    /// Create a wrapper that formats `inner` with the default `,` delimiter
    pub fn new(inner: &'a DbgenVersionRow) -> Self {
        Self {
            inner,
            delimiter: ',',
        }
    }

    /// Create a wrapper that formats `inner` with a custom delimiter
    pub fn with_delimiter(inner: &'a DbgenVersionRow, delimiter: char) -> Self {
        Self { inner, delimiter }
    }

    /// Returns the CSV header for the dbgen_version table
    pub fn header() -> &'static str {
        "dv_version,dv_create_date,dv_create_time,dv_cmdline_args"
    }

    /// Returns the CSV header with a custom delimiter
    pub fn header_with_delimiter(delimiter: char) -> String {
        join_header(Self::header(), delimiter)
    }
}

impl Display for DbgenVersionCsv<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = self.delimiter;
        let row = self.inner;
        write!(
            f,
            "{}{d}{}{d}{}{d}{}",
            row.field(&row.dv_version, 0),
            row.field(row.dv_create_date, 1),
            row.field(TimeOfDay(row.dv_create_time), 2),
            row.field(&row.dv_cmdline_args, 3),
        )
    }
}
/// Writes [`HouseholdDemographicsRow`]s in CSV format.
///
/// # Example
/// ```
/// # use tpcdsgen::config::Session;
/// # use tpcdsgen::csv::HouseholdDemographicsCsv;
/// # use tpcdsgen::row::{GeneratedRow, HouseholdDemographicsRowGenerator, RowGenerator};
/// # use std::fmt::Write;
/// # let session = Session::default();
/// let mut generator = HouseholdDemographicsRowGenerator::new();
/// // Output the first 3 rows in CSV format
/// let mut csv = String::new();
/// writeln!(&mut csv, "{}", HouseholdDemographicsCsv::header()).unwrap(); // write header
/// # for row_number in 1..=3 {
/// #   let result = generator.generate_row_and_child_rows(row_number, &session, None, None).unwrap();
/// #   for row in result.get_rows() {
/// #     let GeneratedRow::HouseholdDemographics(row) = row else { continue };
/// // write line using CSV formatter
/// writeln!(&mut csv, "{}", HouseholdDemographicsCsv::new(row)).unwrap();
/// #   }
/// #   generator.consume_remaining_seeds_for_row();
/// # }
/// assert_eq!(
///   csv,
///   "hd_demo_sk,hd_income_band_sk,hd_buy_potential,hd_dep_count,hd_vehicle_count\n\
///    1,2,0-500,0,0\n\
///    2,3,0-500,0,0\n\
///    3,4,0-500,0,0\n"
/// );
/// ```
pub struct HouseholdDemographicsCsv<'a> {
    inner: &'a HouseholdDemographicsRow,
    delimiter: char,
}

impl<'a> HouseholdDemographicsCsv<'a> {
    /// Create a wrapper that formats `inner` with the default `,` delimiter
    pub fn new(inner: &'a HouseholdDemographicsRow) -> Self {
        Self {
            inner,
            delimiter: ',',
        }
    }

    /// Create a wrapper that formats `inner` with a custom delimiter
    pub fn with_delimiter(inner: &'a HouseholdDemographicsRow, delimiter: char) -> Self {
        Self { inner, delimiter }
    }

    /// Returns the CSV header for the household_demographics table
    pub fn header() -> &'static str {
        "hd_demo_sk,hd_income_band_sk,hd_buy_potential,hd_dep_count,hd_vehicle_count"
    }

    /// Returns the CSV header with a custom delimiter
    pub fn header_with_delimiter(delimiter: char) -> String {
        join_header(Self::header(), delimiter)
    }
}

impl Display for HouseholdDemographicsCsv<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = self.delimiter;
        let row = self.inner;
        write!(
            f,
            "{}{d}{}{d}{}{d}{}{d}{}",
            NullLiteralField::new(row.hd_demo_sk, row.is_null(0)),
            NullLiteralField::new(row.hd_income_band_sk, row.is_null(1)),
            NullLiteralField::new(&row.hd_buy_potential, row.is_null(2)),
            NullLiteralField::new(row.hd_dep_count, row.is_null(3)),
            NullLiteralField::new(row.hd_vehicle_count, row.is_null(4)),
        )
    }
}
/// Writes [`IncomeBandRow`]s in CSV format.
///
/// # Example
/// ```
/// # use tpcdsgen::config::Session;
/// # use tpcdsgen::csv::IncomeBandCsv;
/// # use tpcdsgen::row::{GeneratedRow, IncomeBandRowGenerator, RowGenerator};
/// # use std::fmt::Write;
/// # let session = Session::default();
/// let mut generator = IncomeBandRowGenerator::new();
/// // Output the first 3 rows in CSV format
/// let mut csv = String::new();
/// writeln!(&mut csv, "{}", IncomeBandCsv::header()).unwrap(); // write header
/// # for row_number in 1..=3 {
/// #   let result = generator.generate_row_and_child_rows(row_number, &session, None, None).unwrap();
/// #   for row in result.get_rows() {
/// #     let GeneratedRow::IncomeBand(row) = row else { continue };
/// // write line using CSV formatter
/// writeln!(&mut csv, "{}", IncomeBandCsv::new(row)).unwrap();
/// #   }
/// #   generator.consume_remaining_seeds_for_row();
/// # }
/// assert_eq!(
///   csv,
///   "ib_income_band_sk,ib_lower_bound,ib_upper_bound\n\
///    1,0,10000\n\
///    2,10001,20000\n\
///    3,20001,30000\n"
/// );
/// ```
pub struct IncomeBandCsv<'a> {
    inner: &'a IncomeBandRow,
    delimiter: char,
}

impl<'a> IncomeBandCsv<'a> {
    /// Create a wrapper that formats `inner` with the default `,` delimiter
    pub fn new(inner: &'a IncomeBandRow) -> Self {
        Self {
            inner,
            delimiter: ',',
        }
    }

    /// Create a wrapper that formats `inner` with a custom delimiter
    pub fn with_delimiter(inner: &'a IncomeBandRow, delimiter: char) -> Self {
        Self { inner, delimiter }
    }

    /// Returns the CSV header for the income_band table
    pub fn header() -> &'static str {
        "ib_income_band_sk,ib_lower_bound,ib_upper_bound"
    }

    /// Returns the CSV header with a custom delimiter
    pub fn header_with_delimiter(delimiter: char) -> String {
        join_header(Self::header(), delimiter)
    }
}

impl Display for IncomeBandCsv<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = self.delimiter;
        let row = self.inner;
        write!(
            f,
            "{}{d}{}{d}{}",
            row.field(row.ib_income_band_sk, 0),
            row.field(row.ib_lower_bound, 1),
            row.field(row.ib_upper_bound, 2),
        )
    }
}
/// Writes [`InventoryRow`]s in CSV format.
///
/// # Example
/// ```
/// # use tpcdsgen::config::Session;
/// # use tpcdsgen::csv::InventoryCsv;
/// # use tpcdsgen::row::{GeneratedRow, InventoryRowGenerator, RowGenerator};
/// # use std::fmt::Write;
/// # let session = Session::default();
/// let mut generator = InventoryRowGenerator::new();
/// // Output the first 3 rows in CSV format
/// let mut csv = String::new();
/// writeln!(&mut csv, "{}", InventoryCsv::header()).unwrap(); // write header
/// # for row_number in 1..=3 {
/// #   let result = generator.generate_row_and_child_rows(row_number, &session, None, None).unwrap();
/// #   for row in result.get_rows() {
/// #     let GeneratedRow::Inventory(row) = row else { continue };
/// // write line using CSV formatter
/// writeln!(&mut csv, "{}", InventoryCsv::new(row)).unwrap();
/// #   }
/// #   generator.consume_remaining_seeds_for_row();
/// # }
/// assert_eq!(
///   csv,
///   "inv_date_sk,inv_item_sk,inv_warehouse_sk,inv_quantity_on_hand\n\
///    2450815,1,1,211\n\
///    2450815,2,1,235\n\
///    2450815,4,1,859\n"
/// );
/// ```
pub struct InventoryCsv<'a> {
    inner: &'a InventoryRow,
    delimiter: char,
}

impl<'a> InventoryCsv<'a> {
    /// Create a wrapper that formats `inner` with the default `,` delimiter
    pub fn new(inner: &'a InventoryRow) -> Self {
        Self {
            inner,
            delimiter: ',',
        }
    }

    /// Create a wrapper that formats `inner` with a custom delimiter
    pub fn with_delimiter(inner: &'a InventoryRow, delimiter: char) -> Self {
        Self { inner, delimiter }
    }

    /// Returns the CSV header for the inventory table
    pub fn header() -> &'static str {
        "inv_date_sk,inv_item_sk,inv_warehouse_sk,inv_quantity_on_hand"
    }

    /// Returns the CSV header with a custom delimiter
    pub fn header_with_delimiter(delimiter: char) -> String {
        join_header(Self::header(), delimiter)
    }
}

impl Display for InventoryCsv<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::generator::InventoryGeneratorColumn::*;

        let d = self.delimiter;
        let row = self.inner;
        write!(
            f,
            "{}{d}{}{d}{}{d}{}",
            DatField::new(row.inv_date_sk, row.is_null_at(InvDateSk)),
            DatField::new(row.inv_item_sk, row.is_null_at(InvItemSk)),
            DatField::new(row.inv_warehouse_sk, row.is_null_at(InvWarehouseSk)),
            DatField::new(row.inv_quantity_on_hand, row.is_null_at(InvQuantityOnHand)),
        )
    }
}
/// Writes [`ItemRow`]s in CSV format.
///
/// # Example
/// ```
/// # use tpcdsgen::config::Session;
/// # use tpcdsgen::csv::ItemCsv;
/// # use tpcdsgen::row::{GeneratedRow, RowGenerator, ItemRowGenerator};
/// # use std::fmt::Write;
/// # let session = Session::default();
/// let mut generator = ItemRowGenerator::new();
/// // Output the first 3 rows in CSV format
/// let mut csv = String::new();
/// writeln!(&mut csv, "{}", ItemCsv::header()).unwrap(); // write header
/// # for row_number in 1..=3 {
/// #   let result = generator.generate_row_and_child_rows(row_number, &session, None, None).unwrap();
/// #   for row in result.get_rows() {
/// #     let GeneratedRow::Item(row) = row else { continue };
/// // write line using CSV formatter
/// writeln!(&mut csv, "{}", ItemCsv::new(row)).unwrap();
/// #   }
/// #   generator.consume_remaining_seeds_for_row();
/// # }
/// assert_eq!(
///   csv,
///   "i_item_sk,i_item_id,i_rec_start_date,i_rec_end_date,i_item_desc,i_current_price,i_wholesale_cost,i_brand_id,i_brand,i_class_id,i_class,i_category_id,i_category,i_manufact_id,i_manufact,i_size,i_formulation,i_color,i_units,i_container,i_manager_id,i_product_name\n\
///    1,AAAAAAAABAAAAAAA,1997-10-27,,\"Powers will not get influences. Electoral ports should show low, annual chains. Now young visitors may pose now however final pages. Bitterly right children suit increasing, leading el\",27.02,23.23,5003002,exportischolar #2,3,pop,5,Music,52,ableanti,N/A,3663peru009490160959,spring,Tsp,Unknown,6,ought\n\
///    2,AAAAAAAACAAAAAAA,1997-10-27,2000-10-26,\"False opportunities would run alone with a views. Early approaches would show inc, european intentions; important, main passages shall know urban, \",1.12,0.38,1001001,amalgamalg #1,1,dresses,1,Women,294,esen stable,petite,516steel060826230906,rosy,Bunch,Unknown,98,able\n\
///    3,AAAAAAAACAAAAAAA,2000-10-27,,\"False opportunities would run alone with a views. Early approaches would show inc, european intentions; important, main passages shall know urban, \",7.11,0.38,1001001,brandbrand #4,7,decor,7,Home,294,esen stable,N/A,516steel060826230906,sienna,Cup,Unknown,18,pri\n"
/// );
/// ```
pub struct ItemCsv<'a> {
    inner: &'a ItemRow,
    delimiter: char,
}

impl<'a> ItemCsv<'a> {
    /// Create a wrapper that formats `inner` with the default `,` delimiter
    pub fn new(inner: &'a ItemRow) -> Self {
        Self {
            inner,
            delimiter: ',',
        }
    }

    /// Create a wrapper that formats `inner` with a custom delimiter
    pub fn with_delimiter(inner: &'a ItemRow, delimiter: char) -> Self {
        Self { inner, delimiter }
    }

    /// Returns the CSV header for the item table
    pub fn header() -> &'static str {
        "i_item_sk,i_item_id,i_rec_start_date,i_rec_end_date,i_item_desc,i_current_price,i_wholesale_cost,i_brand_id,i_brand,i_class_id,i_class,i_category_id,i_category,i_manufact_id,i_manufact,i_size,i_formulation,i_color,i_units,i_container,i_manager_id,i_product_name"
    }

    /// Returns the CSV header with a custom delimiter
    pub fn header_with_delimiter(delimiter: char) -> String {
        join_header(Self::header(), delimiter)
    }
}

impl Display for ItemCsv<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::generator::ItemGeneratorColumn::*;

        let d = self.delimiter;
        let row = self.inner;
        write!(
            f,
            "{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}",
            row.field(row.i_item_sk, &IItemSk),
            row.field(&row.i_item_id, &IItemId),
            row.date_field(row.i_rec_start_date_id, &IRecStartDateId),
            row.date_field(row.i_rec_end_date_id, &IRecEndDateId),
            CsvQuoted::new(&row.i_item_desc, row.is_null(&IItemDesc)),
            row.field(row.i_current_price, &ICurrentPrice),
            row.field(row.i_wholesale_cost, &IWholesaleCost),
            row.field(row.i_brand_id, &IBrandId),
            row.field(&row.i_brand, &IBrand),
            row.field(row.i_class_id, &IClassId),
            row.field(&row.i_class, &IClass),
            row.field(row.i_category_id, &ICategoryId),
            row.field(&row.i_category, &ICategory),
            row.field(row.i_manufact_id, &IManufactId),
            row.field(&row.i_manufact, &IManufact),
            row.field(&row.i_size, &ISize),
            row.field(&row.i_formulation, &IFormulation),
            row.field(&row.i_color, &IColor),
            row.field(&row.i_units, &IUnits),
            row.field(&row.i_container, &IContainer),
            row.field(row.i_manager_id, &IManagerId),
            row.field(&row.i_product_name, &IProductName),
        )
    }
}
/// Writes [`PromotionRow`]s in CSV format.
///
/// # Example
/// ```
/// # use tpcdsgen::config::Session;
/// # use tpcdsgen::csv::PromotionCsv;
/// # use tpcdsgen::row::{GeneratedRow, RowGenerator, PromotionRowGenerator};
/// # use std::fmt::Write;
/// # let session = Session::default();
/// let mut generator = PromotionRowGenerator::new();
/// // Output the first 3 rows in CSV format
/// let mut csv = String::new();
/// writeln!(&mut csv, "{}", PromotionCsv::header()).unwrap(); // write header
/// # for row_number in 1..=3 {
/// #   let result = generator.generate_row_and_child_rows(row_number, &session, None, None).unwrap();
/// #   for row in result.get_rows() {
/// #     let GeneratedRow::Promotion(row) = row else { continue };
/// // write line using CSV formatter
/// writeln!(&mut csv, "{}", PromotionCsv::new(row)).unwrap();
/// #   }
/// #   generator.consume_remaining_seeds_for_row();
/// # }
/// assert_eq!(
///   csv,
///   "p_promo_sk,p_promo_id,p_start_date_sk,p_end_date_sk,p_item_sk,p_cost,p_response_target,p_promo_name,p_channel_dmail,p_channel_email,p_channel_catalog,p_channel_tv,p_channel_radio,p_channel_press,p_channel_event,p_channel_demo,p_channel_details,p_purpose,p_discount_active\n\
///    1,AAAAAAAABAAAAAAA,2450164,2450185,10022,1000.00,1,ought,Y,N,N,N,N,N,N,N,\"Men will not say merely. Old, available \",Unknown,N\n\
///    2,AAAAAAAACAAAAAAA,2450118,2450150,2410,1000.00,1,able,Y,N,N,N,N,N,N,N,\"So willing buildings coul\",Unknown,N\n\
///    3,AAAAAAAADAAAAAAA,2450675,2450712,10843,1000.00,1,pri,Y,N,N,N,N,N,N,N,\"Companies shall not pr\",Unknown,N\n"
/// );
/// ```
pub struct PromotionCsv<'a> {
    inner: &'a PromotionRow,
    delimiter: char,
}

impl<'a> PromotionCsv<'a> {
    /// Create a wrapper that formats `inner` with the default `,` delimiter
    pub fn new(inner: &'a PromotionRow) -> Self {
        Self {
            inner,
            delimiter: ',',
        }
    }

    /// Create a wrapper that formats `inner` with a custom delimiter
    pub fn with_delimiter(inner: &'a PromotionRow, delimiter: char) -> Self {
        Self { inner, delimiter }
    }

    /// Returns the CSV header for the promotion table
    pub fn header() -> &'static str {
        "p_promo_sk,p_promo_id,p_start_date_sk,p_end_date_sk,p_item_sk,p_cost,p_response_target,p_promo_name,p_channel_dmail,p_channel_email,p_channel_catalog,p_channel_tv,p_channel_radio,p_channel_press,p_channel_event,p_channel_demo,p_channel_details,p_purpose,p_discount_active"
    }

    /// Returns the CSV header with a custom delimiter
    pub fn header_with_delimiter(delimiter: char) -> String {
        join_header(Self::header(), delimiter)
    }
}

impl Display for PromotionCsv<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::generator::PromotionGeneratorColumn::*;

        let d = self.delimiter;
        let row = self.inner;
        write!(
            f,
            "{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}",
            row.key_field(row.p_promo_sk, PPromoSk),
            row.field(&row.p_promo_id, PPromoId),
            row.key_field(row.p_start_date_id, PStartDateId),
            row.key_field(row.p_end_date_id, PEndDateId),
            row.key_field(row.p_item_sk, PItemSk),
            row.field(row.p_cost, PCost),
            row.field(row.p_response_target, PResponseTarget),
            row.field(&row.p_promo_name, PPromoName),
            DatField::yes_no(row.p_channel_dmail, row.is_null_at(PChannelDmail)),
            DatField::yes_no(row.p_channel_email, row.is_null_at(PChannelEmail)),
            DatField::yes_no(row.p_channel_catalog, row.is_null_at(PChannelCatalog)),
            DatField::yes_no(row.p_channel_tv, row.is_null_at(PChannelTv)),
            DatField::yes_no(row.p_channel_radio, row.is_null_at(PChannelRadio)),
            DatField::yes_no(row.p_channel_press, row.is_null_at(PChannelPress)),
            DatField::yes_no(row.p_channel_event, row.is_null_at(PChannelEvent)),
            DatField::yes_no(row.p_channel_demo, row.is_null_at(PChannelDemo)),
            CsvQuoted::new(&row.p_channel_details, row.is_null_at(PChannelDetails)),
            row.field(&row.p_purpose, PPurpose),
            DatField::yes_no(row.p_discount_active, row.is_null_at(PDiscountActive)),
        )
    }
}
/// Writes [`ReasonRow`]s in CSV format.
///
/// # Example
/// ```
/// # use tpcdsgen::config::Session;
/// # use tpcdsgen::csv::ReasonCsv;
/// # use tpcdsgen::row::{GeneratedRow, RowGenerator, ReasonRowGenerator};
/// # use std::fmt::Write;
/// # let session = Session::default();
/// let mut generator = ReasonRowGenerator::new();
/// // Output the first 3 rows in CSV format
/// let mut csv = String::new();
/// writeln!(&mut csv, "{}", ReasonCsv::header()).unwrap(); // write header
/// # for row_number in 1..=3 {
/// #   let result = generator.generate_row_and_child_rows(row_number, &session, None, None).unwrap();
/// #   for row in result.get_rows() {
/// #     let GeneratedRow::Reason(row) = row else { continue };
/// // write line using CSV formatter
/// writeln!(&mut csv, "{}", ReasonCsv::new(row)).unwrap();
/// #   }
/// #   generator.consume_remaining_seeds_for_row();
/// # }
/// assert_eq!(
///   csv,
///   "r_reason_sk,r_reason_id,r_reason_desc\n\
///    1,AAAAAAAABAAAAAAA,Package was damaged\n\
///    2,AAAAAAAACAAAAAAA,Stopped working\n\
///    3,AAAAAAAADAAAAAAA,Did not get it on time\n"
/// );
/// ```
pub struct ReasonCsv<'a> {
    inner: &'a ReasonRow,
    delimiter: char,
}

impl<'a> ReasonCsv<'a> {
    /// Create a wrapper that formats `inner` with the default `,` delimiter
    pub fn new(inner: &'a ReasonRow) -> Self {
        Self {
            inner,
            delimiter: ',',
        }
    }

    /// Create a wrapper that formats `inner` with a custom delimiter
    pub fn with_delimiter(inner: &'a ReasonRow, delimiter: char) -> Self {
        Self { inner, delimiter }
    }

    /// Returns the CSV header for the reason table
    pub fn header() -> &'static str {
        "r_reason_sk,r_reason_id,r_reason_desc"
    }

    /// Returns the CSV header with a custom delimiter
    pub fn header_with_delimiter(delimiter: char) -> String {
        join_header(Self::header(), delimiter)
    }
}

impl Display for ReasonCsv<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = self.delimiter;
        let row = self.inner;
        write!(
            f,
            "{}{d}{}{d}{}",
            row.field(row.r_reason_sk, 0),
            row.field(&row.r_reason_id, 1),
            row.field(&row.r_reason_desc, 2),
        )
    }
}
/// Writes [`ShipModeRow`]s in CSV format.
///
/// # Example
/// ```
/// # use tpcdsgen::config::Session;
/// # use tpcdsgen::csv::ShipModeCsv;
/// # use tpcdsgen::row::{GeneratedRow, RowGenerator, ShipModeRowGenerator};
/// # use std::fmt::Write;
/// # let session = Session::default();
/// let mut generator = ShipModeRowGenerator::new();
/// // Output the first 3 rows in CSV format
/// let mut csv = String::new();
/// writeln!(&mut csv, "{}", ShipModeCsv::header()).unwrap(); // write header
/// # for row_number in 1..=3 {
/// #   let result = generator.generate_row_and_child_rows(row_number, &session, None, None).unwrap();
/// #   for row in result.get_rows() {
/// #     let GeneratedRow::ShipMode(row) = row else { continue };
/// // write line using CSV formatter
/// writeln!(&mut csv, "{}", ShipModeCsv::new(row)).unwrap();
/// #   }
/// #   generator.consume_remaining_seeds_for_row();
/// # }
/// assert_eq!(
///   csv,
///   "sm_ship_mode_sk,sm_ship_mode_id,sm_type,sm_code,sm_carrier,sm_contract\n\
///    1,AAAAAAAABAAAAAAA,EXPRESS,AIR,UPS,YvxVaJI10\n\
///    2,AAAAAAAACAAAAAAA,NEXT DAY,AIR,FEDEX,ldhM8IvpzHgdbBgDfI\n\
///    3,AAAAAAAADAAAAAAA,OVERNIGHT,AIR,AIRBORNE,6Hzzp4JkzjqD8MGXLCDa\n"
/// );
/// ```
pub struct ShipModeCsv<'a> {
    inner: &'a ShipModeRow,
    delimiter: char,
}

impl<'a> ShipModeCsv<'a> {
    /// Create a wrapper that formats `inner` with the default `,` delimiter
    pub fn new(inner: &'a ShipModeRow) -> Self {
        Self {
            inner,
            delimiter: ',',
        }
    }

    /// Create a wrapper that formats `inner` with a custom delimiter
    pub fn with_delimiter(inner: &'a ShipModeRow, delimiter: char) -> Self {
        Self { inner, delimiter }
    }

    /// Returns the CSV header for the ship_mode table
    pub fn header() -> &'static str {
        "sm_ship_mode_sk,sm_ship_mode_id,sm_type,sm_code,sm_carrier,sm_contract"
    }

    /// Returns the CSV header with a custom delimiter
    pub fn header_with_delimiter(delimiter: char) -> String {
        join_header(Self::header(), delimiter)
    }
}

impl Display for ShipModeCsv<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = self.delimiter;
        let row = self.inner;
        write!(
            f,
            "{}{d}{}{d}{}{d}{}{d}{}{d}{}",
            row.field(row.sm_ship_mode_sk, 0),
            row.field(&row.sm_ship_mode_id, 1),
            row.field(&row.sm_type, 2),
            row.field(&row.sm_code, 3),
            row.field(&row.sm_carrier, 4),
            row.field(&row.sm_contract, 5),
        )
    }
}
/// Writes [`StoreRow`]s in CSV format.
///
/// # Example
/// ```
/// # use tpcdsgen::config::Session;
/// # use tpcdsgen::csv::StoreCsv;
/// # use tpcdsgen::row::{GeneratedRow, RowGenerator, StoreRowGenerator};
/// # use std::fmt::Write;
/// # let session = Session::default();
/// let mut generator = StoreRowGenerator::new();
/// // Output the first 3 rows in CSV format
/// let mut csv = String::new();
/// writeln!(&mut csv, "{}", StoreCsv::header()).unwrap(); // write header
/// # for row_number in 1..=3 {
/// #   let result = generator.generate_row_and_child_rows(row_number, &session, None, None).unwrap();
/// #   for row in result.get_rows() {
/// #     let GeneratedRow::Store(row) = row else { continue };
/// // write line using CSV formatter
/// writeln!(&mut csv, "{}", StoreCsv::new(row)).unwrap();
/// #   }
/// #   generator.consume_remaining_seeds_for_row();
/// # }
/// assert_eq!(
///   csv,
///   "s_store_sk,s_store_id,s_rec_start_date,s_rec_end_date,s_closed_date_sk,s_store_name,s_number_employees,s_floor_space,s_hours,s_manager,s_market_id,s_geography_class,s_market_desc,s_market_manager,s_division_id,s_division_name,s_company_id,s_company_name,s_street_number,s_street_name,s_street_type,s_suite_number,s_city,s_county,s_state,s_zip,s_country,s_gmt_offset,s_tax_precentage\n\
///    1,AAAAAAAABAAAAAAA,1997-03-13,,2451189,ought,245,5250760,8AM-4PM,William Ward,2,Unknown,\"Enough high areas stop expectations. Elaborate, local is\",Charles Bartley,1,Unknown,1,Unknown,767,Spring ,Wy,Suite 250,Midway,Williamson County,TN,31904,United States,-5,0.03\n\
///    2,AAAAAAAACAAAAAAA,1997-03-13,2000-03-12,,able,236,5285950,8AM-4PM,Scott Smith,8,Unknown,\"Parliamentary candidates wait then heavy, keen mil\",David Lamontagne,1,Unknown,1,Unknown,255,Sycamore ,Dr.,Suite 410,Midway,Williamson County,TN,31904,United States,-5,0.03\n\
///    3,AAAAAAAACAAAAAAA,2000-03-13,,,able,236,7557959,8AM-4PM,Scott Smith,7,Unknown,\"Impossible, true arms can treat constant, complete w\",David Lamontagne,1,Unknown,1,Unknown,877,Park Laurel,Road,Suite T,Midway,Williamson County,TN,31904,United States,-5,0.03\n"
/// );
/// ```
pub struct StoreCsv<'a> {
    inner: &'a StoreRow,
    delimiter: char,
}

impl<'a> StoreCsv<'a> {
    /// Create a wrapper that formats `inner` with the default `,` delimiter
    pub fn new(inner: &'a StoreRow) -> Self {
        Self {
            inner,
            delimiter: ',',
        }
    }

    /// Create a wrapper that formats `inner` with a custom delimiter
    pub fn with_delimiter(inner: &'a StoreRow, delimiter: char) -> Self {
        Self { inner, delimiter }
    }

    /// Returns the CSV header for the store table
    pub fn header() -> &'static str {
        "s_store_sk,s_store_id,s_rec_start_date,s_rec_end_date,s_closed_date_sk,s_store_name,s_number_employees,s_floor_space,s_hours,s_manager,s_market_id,s_geography_class,s_market_desc,s_market_manager,s_division_id,s_division_name,s_company_id,s_company_name,s_street_number,s_street_name,s_street_type,s_suite_number,s_city,s_county,s_state,s_zip,s_country,s_gmt_offset,s_tax_precentage"
    }

    /// Returns the CSV header with a custom delimiter
    pub fn header_with_delimiter(delimiter: char) -> String {
        join_header(Self::header(), delimiter)
    }
}

impl Display for StoreCsv<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::generator::StoreGeneratorColumn::*;

        let d = self.delimiter;
        let row = self.inner;
        write!(
            f,
            "{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}",
            row.key_field(row.store_sk, &WStoreSk),
            row.field(&row.store_id, &WStoreId),
            row.date_field(row.rec_start_date_id, &WStoreRecStartDateId),
            row.date_field(row.rec_end_date_id, &WStoreRecEndDateId),
            row.key_field(row.closed_date_id, &WStoreClosedDateId),
            row.field(&row.store_name, &WStoreName),
            row.field(row.employees, &WStoreEmployees),
            row.field(row.floor_space, &WStoreFloorSpace),
            row.field(&row.hours, &WStoreHours),
            row.field(&row.store_manager, &WStoreManager),
            row.field(row.market_id, &WStoreMarketId),
            row.field(&row.geography_class, &WStoreGeographyClass),
            CsvQuoted::new(&row.market_desc, row.is_null(&WStoreMarketDesc)),
            row.field(&row.market_manager, &WStoreMarketManager),
            row.key_field(row.division_id, &WStoreDivisionId),
            row.field(&row.division_name, &WStoreDivisionName),
            row.key_field(row.company_id, &WStoreCompanyId),
            row.field(&row.company_name, &WStoreCompanyName),
            row.field(row.address.get_street_number(), &WStoreAddressStreetNum),
            row.field(row.address.get_street_name(), &WStoreAddressStreetName1),
            row.field(row.address.get_street_type(), &WStoreAddressStreetType),
            row.field(row.address.get_suite_number(), &WStoreAddressSuiteNum),
            row.field(row.address.get_city(), &WStoreAddressCity),
            row.field(row.address.get_county().unwrap_or(""), &WStoreAddressCounty),
            row.field(row.address.get_state(), &WStoreAddressState),
            DatField::zip(row.address.get_zip(), row.is_null(&WStoreAddressZip)),
            row.field(row.address.get_country(), &WStoreAddressCountry),
            row.field(row.address.get_gmt_offset(), &WStoreAddressGmtOffset),
            row.field(row.d_tax_percentage, &WStoreTaxPercentage),
        )
    }
}
/// Writes [`StoreReturnsRow`]s in CSV format.
///
/// # Example
/// ```
/// # use tpcdsgen::config::Session;
/// # use tpcdsgen::csv::StoreReturnsCsv;
/// # use tpcdsgen::row::{GeneratedRow, RowGenerator, StoreSalesRowGenerator};
/// # use std::fmt::Write;
/// # let session = Session::default();
/// // store_returns rows are generated as child rows of the store_sales generator
/// let mut generator = StoreSalesRowGenerator::new();
/// // Output the returns rows produced by the first 2 store_sales row numbers
/// let mut csv = String::new();
/// writeln!(&mut csv, "{}", StoreReturnsCsv::header()).unwrap(); // write header
/// # let mut row_number = 1;
/// # while row_number <= 2 {
/// #   let result = generator.generate_row_and_child_rows(row_number, &session, None, None).unwrap();
/// #   for row in result.get_rows() {
/// #     let GeneratedRow::StoreReturns(row) = row else { continue };
/// // write line using CSV formatter
/// writeln!(&mut csv, "{}", StoreReturnsCsv::new(row)).unwrap();
/// #   }
/// #   if result.should_end_row() {
/// #     generator.consume_remaining_seeds_for_row();
/// #     row_number += 1;
/// #   }
/// # }
/// assert_eq!(
///   csv,
///   "sr_returned_date_sk,sr_return_time_sk,sr_item_sk,sr_customer_sk,sr_cdemo_sk,sr_hdemo_sk,sr_addr_sk,sr_store_sk,sr_reason_sk,sr_ticket_number,sr_return_quantity,sr_return_amt,sr_return_tax,sr_return_amt_inc_tax,sr_fee,sr_return_ship_cost,sr_refunded_cash,sr_reversed_charge,sr_store_credit,sr_net_loss\n\
///    2451984,46418,4553,67006,793022,4033,17397,7,19,1,51,37.23,3.35,40.58,55.28,714.00,0.74,17.51,18.98,772.63\n\
///    2451822,47480,10993,67006,1082163,7157,49751,1,29,1,43,4009.32,120.27,4129.59,28.23,0.00,3448.01,5.61,555.70,148.50\n\
///    2451653,37700,7654,68284,289468,3067,21819,10,35,2,7,249.48,2.49,251.97,11.50,6.23,234.51,8.83,6.14,20.22\n"
/// );
/// ```
pub struct StoreReturnsCsv<'a> {
    inner: &'a StoreReturnsRow,
    delimiter: char,
}

impl<'a> StoreReturnsCsv<'a> {
    /// Create a wrapper that formats `inner` with the default `,` delimiter
    pub fn new(inner: &'a StoreReturnsRow) -> Self {
        Self {
            inner,
            delimiter: ',',
        }
    }

    /// Create a wrapper that formats `inner` with a custom delimiter
    pub fn with_delimiter(inner: &'a StoreReturnsRow, delimiter: char) -> Self {
        Self { inner, delimiter }
    }

    /// Returns the CSV header for the store_returns table
    pub fn header() -> &'static str {
        "sr_returned_date_sk,sr_return_time_sk,sr_item_sk,sr_customer_sk,sr_cdemo_sk,sr_hdemo_sk,sr_addr_sk,sr_store_sk,sr_reason_sk,sr_ticket_number,sr_return_quantity,sr_return_amt,sr_return_tax,sr_return_amt_inc_tax,sr_fee,sr_return_ship_cost,sr_refunded_cash,sr_reversed_charge,sr_store_credit,sr_net_loss"
    }

    /// Returns the CSV header with a custom delimiter
    pub fn header_with_delimiter(delimiter: char) -> String {
        join_header(Self::header(), delimiter)
    }
}

impl Display for StoreReturnsCsv<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::generator::StoreReturnsGeneratorColumn::*;

        let d = self.delimiter;
        let row = self.inner;
        write!(
            f,
            "{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}",
            DatField::key(row.sr_returned_date_sk, row.is_null_at(SrReturnedDateSk)),
            DatField::key(row.sr_returned_time_sk, row.is_null_at(SrReturnedTimeSk)),
            DatField::key(row.sr_item_sk, row.is_null_at(SrItemSk)),
            DatField::key(row.sr_customer_sk, row.is_null_at(SrCustomerSk)),
            DatField::key(row.sr_cdemo_sk, row.is_null_at(SrCdemoSk)),
            DatField::key(row.sr_hdemo_sk, row.is_null_at(SrHdemoSk)),
            DatField::key(row.sr_addr_sk, row.is_null_at(SrAddrSk)),
            DatField::key(row.sr_store_sk, row.is_null_at(SrStoreSk)),
            DatField::key(row.sr_reason_sk, row.is_null_at(SrReasonSk)),
            DatField::key(row.sr_ticket_number, row.is_null_at(SrTicketNumber)),
            DatField::new(
                row.sr_pricing.get_quantity(),
                row.is_null_at(SrPricingQuantity)
            ),
            DatField::new(
                row.sr_pricing.get_net_paid(),
                row.is_null_at(SrPricingNetPaid)
            ),
            DatField::new(
                row.sr_pricing.get_ext_tax(),
                row.is_null_at(SrPricingExtTax)
            ),
            DatField::new(
                row.sr_pricing.get_net_paid_including_tax(),
                row.is_null_at(SrPricingNetPaidIncTax)
            ),
            DatField::new(row.sr_pricing.get_fee(), row.is_null_at(SrPricingFee)),
            DatField::new(
                row.sr_pricing.get_ext_ship_cost(),
                row.is_null_at(SrPricingExtShipCost)
            ),
            DatField::new(
                row.sr_pricing.get_refunded_cash(),
                row.is_null_at(SrPricingRefundedCash)
            ),
            DatField::new(
                row.sr_pricing.get_reversed_charge(),
                row.is_null_at(SrPricingReversedCharge)
            ),
            DatField::new(
                row.sr_pricing.get_store_credit(),
                row.is_null_at(SrPricingStoreCredit)
            ),
            DatField::new(
                row.sr_pricing.get_net_loss(),
                row.is_null_at(SrPricingNetLoss)
            ),
        )
    }
}
/// Writes [`StoreSalesRow`]s in CSV format.
///
/// # Example
/// ```
/// # use tpcdsgen::config::Session;
/// # use tpcdsgen::csv::StoreSalesCsv;
/// # use tpcdsgen::row::{GeneratedRow, RowGenerator, StoreSalesRowGenerator};
/// # use std::fmt::Write;
/// # let session = Session::default();
/// let mut generator = StoreSalesRowGenerator::new();
/// // Output the first 3 rows in CSV format
/// let mut csv = String::new();
/// writeln!(&mut csv, "{}", StoreSalesCsv::header()).unwrap(); // write header
/// # for row_number in 1..=3 {
/// #   let result = generator.generate_row_and_child_rows(row_number, &session, None, None).unwrap();
/// #   for row in result.get_rows() {
/// #     let GeneratedRow::StoreSales(row) = row else { continue };
/// // write line using CSV formatter
/// writeln!(&mut csv, "{}", StoreSalesCsv::new(row)).unwrap();
/// #   }
/// #   if result.should_end_row() {
/// #     generator.consume_remaining_seeds_for_row();
/// #   }
/// # }
/// assert_eq!(
///   csv,
///   "ss_sold_date_sk,ss_sold_time_sk,ss_item_sk,ss_customer_sk,ss_cdemo_sk,ss_hdemo_sk,ss_addr_sk,ss_store_sk,ss_promo_sk,ss_ticket_number,ss_quantity,ss_wholesale_cost,ss_list_price,ss_sales_price,ss_ext_discount_amt,ss_ext_sales_price,ss_ext_wholesale_cost,ss_ext_list_price,ss_ext_tax,ss_coupon_amt,ss_net_paid,ss_net_paid_inc_tax,ss_net_profit\n\
///    2451813,65495,3617,67006,591617,3428,24839,10,161,1,79,11.41,18.71,2.80,99.54,221.20,901.39,1478.09,6.08,99.54,121.66,127.74,-779.73\n\
///    2451813,65495,13283,67006,591617,3428,24839,10,154,1,37,63.63,101.17,41.47,46.03,1534.39,2354.31,3743.29,59.53,46.03,1488.36,1547.89,-865.95\n\
///    2451813,65495,13631,67006,591617,3428,24839,10,172,1,99,80.52,137.68,83.98,0.00,8314.02,7971.48,13630.32,0.00,0.00,8314.02,8314.02,342.54\n"
/// );
/// ```
pub struct StoreSalesCsv<'a> {
    inner: &'a StoreSalesRow,
    delimiter: char,
}

impl<'a> StoreSalesCsv<'a> {
    /// Create a wrapper that formats `inner` with the default `,` delimiter
    pub fn new(inner: &'a StoreSalesRow) -> Self {
        Self {
            inner,
            delimiter: ',',
        }
    }

    /// Create a wrapper that formats `inner` with a custom delimiter
    pub fn with_delimiter(inner: &'a StoreSalesRow, delimiter: char) -> Self {
        Self { inner, delimiter }
    }

    /// Returns the CSV header for the store_sales table
    pub fn header() -> &'static str {
        "ss_sold_date_sk,ss_sold_time_sk,ss_item_sk,ss_customer_sk,ss_cdemo_sk,ss_hdemo_sk,ss_addr_sk,ss_store_sk,ss_promo_sk,ss_ticket_number,ss_quantity,ss_wholesale_cost,ss_list_price,ss_sales_price,ss_ext_discount_amt,ss_ext_sales_price,ss_ext_wholesale_cost,ss_ext_list_price,ss_ext_tax,ss_coupon_amt,ss_net_paid,ss_net_paid_inc_tax,ss_net_profit"
    }

    /// Returns the CSV header with a custom delimiter
    pub fn header_with_delimiter(delimiter: char) -> String {
        join_header(Self::header(), delimiter)
    }
}

impl Display for StoreSalesCsv<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::generator::StoreSalesGeneratorColumn::*;
        use crate::row::table_row::DatField;

        let d = self.delimiter;
        let row = self.inner;

        // Note: Java has coupon_amount twice at positions 15 and 20 (bug in original)
        // We replicate this for byte-for-byte compatibility
        write!(
            f,
            "{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}",
            DatField::key(row.ss_sold_date_sk, row.is_null_at(SsSoldDateSk)),
            DatField::key(row.ss_sold_time_sk, row.is_null_at(SsSoldTimeSk)),
            DatField::key(row.ss_sold_item_sk, row.is_null_at(SsSoldItemSk)),
            DatField::key(row.ss_sold_customer_sk, row.is_null_at(SsSoldCustomerSk)),
            DatField::key(row.ss_sold_cdemo_sk, row.is_null_at(SsSoldCdemoSk)),
            DatField::key(row.ss_sold_hdemo_sk, row.is_null_at(SsSoldHdemoSk)),
            DatField::key(row.ss_sold_addr_sk, row.is_null_at(SsSoldAddrSk)),
            DatField::key(row.ss_sold_store_sk, row.is_null_at(SsSoldStoreSk)),
            DatField::key(row.ss_sold_promo_sk, row.is_null_at(SsSoldPromoSk)),
            DatField::key(row.ss_ticket_number, row.is_null_at(SsTicketNumber)),
            DatField::new(
                row.ss_pricing.get_quantity(),
                row.is_null_at(SsPricingQuantity)
            ),
            DatField::new(
                row.ss_pricing.get_wholesale_cost(),
                row.is_null_at(SsPricingWholesaleCost)
            ),
            DatField::new(
                row.ss_pricing.get_list_price(),
                row.is_null_at(SsPricingListPrice)
            ),
            DatField::new(
                row.ss_pricing.get_sales_price(),
                row.is_null_at(SsPricingSalesPrice)
            ),
            DatField::new(
                row.ss_pricing.get_coupon_amount(),
                row.is_null_at(SsPricingCouponAmt)
            ),
            DatField::new(
                row.ss_pricing.get_ext_sales_price(),
                row.is_null_at(SsPricingExtSalesPrice)
            ),
            DatField::new(
                row.ss_pricing.get_ext_wholesale_cost(),
                row.is_null_at(SsPricingExtWholesaleCost)
            ),
            DatField::new(
                row.ss_pricing.get_ext_list_price(),
                row.is_null_at(SsPricingExtListPrice)
            ),
            DatField::new(
                row.ss_pricing.get_ext_tax(),
                row.is_null_at(SsPricingExtTax)
            ),
            DatField::new(
                row.ss_pricing.get_coupon_amount(),
                row.is_null_at(SsPricingCouponAmt)
            ),
            DatField::new(
                row.ss_pricing.get_net_paid(),
                row.is_null_at(SsPricingNetPaid)
            ),
            DatField::new(
                row.ss_pricing.get_net_paid_including_tax(),
                row.is_null_at(SsPricingNetPaidIncTax)
            ),
            DatField::new(
                row.ss_pricing.get_net_profit(),
                row.is_null_at(SsPricingNetProfit)
            ),
        )
    }
}
/// Writes [`TimeDimRow`]s in CSV format.
///
/// # Example
/// ```
/// # use tpcdsgen::config::Session;
/// # use tpcdsgen::csv::TimeDimCsv;
/// # use tpcdsgen::row::{GeneratedRow, RowGenerator, TimeDimRowGenerator};
/// # use std::fmt::Write;
/// # let session = Session::default();
/// let mut generator = TimeDimRowGenerator::new();
/// // Output the first 3 rows in CSV format
/// let mut csv = String::new();
/// writeln!(&mut csv, "{}", TimeDimCsv::header()).unwrap(); // write header
/// # for row_number in 1..=3 {
/// #   let result = generator.generate_row_and_child_rows(row_number, &session, None, None).unwrap();
/// #   for row in result.get_rows() {
/// #     let GeneratedRow::TimeDim(row) = row else { continue };
/// // write line using CSV formatter
/// writeln!(&mut csv, "{}", TimeDimCsv::new(row)).unwrap();
/// #   }
/// #   generator.consume_remaining_seeds_for_row();
/// # }
/// assert_eq!(
///   csv,
///   "t_time_sk,t_time_id,t_time,t_hour,t_minute,t_second,t_am_pm,t_shift,t_sub_shift,t_meal_time\n\
///    0,AAAAAAAABAAAAAAA,0,0,0,0,AM,third,night,\n\
///    1,AAAAAAAACAAAAAAA,1,0,0,1,AM,third,night,\n\
///    2,AAAAAAAADAAAAAAA,2,0,0,2,AM,third,night,\n"
/// );
/// ```
pub struct TimeDimCsv<'a> {
    inner: &'a TimeDimRow,
    delimiter: char,
}

impl<'a> TimeDimCsv<'a> {
    /// Create a wrapper that formats `inner` with the default `,` delimiter
    pub fn new(inner: &'a TimeDimRow) -> Self {
        Self {
            inner,
            delimiter: ',',
        }
    }

    /// Create a wrapper that formats `inner` with a custom delimiter
    pub fn with_delimiter(inner: &'a TimeDimRow, delimiter: char) -> Self {
        Self { inner, delimiter }
    }

    /// Returns the CSV header for the time_dim table
    pub fn header() -> &'static str {
        "t_time_sk,t_time_id,t_time,t_hour,t_minute,t_second,t_am_pm,t_shift,t_sub_shift,t_meal_time"
    }

    /// Returns the CSV header with a custom delimiter
    pub fn header_with_delimiter(delimiter: char) -> String {
        join_header(Self::header(), delimiter)
    }
}

impl Display for TimeDimCsv<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = self.delimiter;
        let row = self.inner;
        write!(
            f,
            "{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}",
            row.field(row.t_time_sk, 0),
            row.field(&row.t_time_id, 1),
            row.field(row.t_time, 2),
            row.field(row.t_hour, 3),
            row.field(row.t_minute, 4),
            row.field(row.t_second, 5),
            row.field(&row.t_am_pm, 6),
            row.field(&row.t_shift, 7),
            row.field(&row.t_sub_shift, 8),
            row.field(&row.t_meal_time, 9),
        )
    }
}
/// Writes [`WarehouseRow`]s in CSV format.
///
/// # Example
/// ```
/// # use tpcdsgen::config::Session;
/// # use tpcdsgen::csv::WarehouseCsv;
/// # use tpcdsgen::row::{GeneratedRow, RowGenerator, WarehouseRowGenerator};
/// # use std::fmt::Write;
/// # let session = Session::default();
/// let mut generator = WarehouseRowGenerator::new();
/// // Output the first 3 rows in CSV format
/// let mut csv = String::new();
/// writeln!(&mut csv, "{}", WarehouseCsv::header()).unwrap(); // write header
/// # for row_number in 1..=3 {
/// #   let result = generator.generate_row_and_child_rows(row_number, &session, None, None).unwrap();
/// #   for row in result.get_rows() {
/// #     let GeneratedRow::Warehouse(row) = row else { continue };
/// // write line using CSV formatter
/// writeln!(&mut csv, "{}", WarehouseCsv::new(row)).unwrap();
/// #   }
/// #   generator.consume_remaining_seeds_for_row();
/// # }
/// assert_eq!(
///   csv,
///   "w_warehouse_sk,w_warehouse_id,w_warehouse_name,w_warehouse_sq_ft,w_street_number,w_street_name,w_street_type,w_suite_number,w_city,w_county,w_state,w_zip,w_country,w_gmt_offset\n\
///    1,AAAAAAAABAAAAAAA,\"Conventional childr\",977787,651,6th ,Parkway,Suite 470,Fairview,Williamson County,TN,35709,United States,-5\n\
///    2,AAAAAAAACAAAAAAA,\"Important issues liv\",138504,600,View First,Avenue,Suite P,Fairview,Williamson County,TN,35709,United States,-5\n\
///    3,AAAAAAAADAAAAAAA,\"Doors canno\",294242,534,Ash Laurel,Dr.,Suite 0,Fairview,Williamson County,TN,35709,United States,-5\n"
/// );
/// ```
pub struct WarehouseCsv<'a> {
    inner: &'a WarehouseRow,
    delimiter: char,
}

impl<'a> WarehouseCsv<'a> {
    /// Create a wrapper that formats `inner` with the default `,` delimiter
    pub fn new(inner: &'a WarehouseRow) -> Self {
        Self {
            inner,
            delimiter: ',',
        }
    }

    /// Create a wrapper that formats `inner` with a custom delimiter
    pub fn with_delimiter(inner: &'a WarehouseRow, delimiter: char) -> Self {
        Self { inner, delimiter }
    }

    /// Returns the CSV header for the warehouse table
    pub fn header() -> &'static str {
        "w_warehouse_sk,w_warehouse_id,w_warehouse_name,w_warehouse_sq_ft,w_street_number,w_street_name,w_street_type,w_suite_number,w_city,w_county,w_state,w_zip,w_country,w_gmt_offset"
    }

    /// Returns the CSV header with a custom delimiter
    pub fn header_with_delimiter(delimiter: char) -> String {
        join_header(Self::header(), delimiter)
    }
}

impl Display for WarehouseCsv<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::row::table_row::{CsvQuoted, DatField};

        let d = self.delimiter;
        let row = self.inner;
        write!(
            f,
            "{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}",
            row.field(row.w_warehouse_sk, 0),
            row.field(&row.w_warehouse_id, 1),
            CsvQuoted::new(&row.w_warehouse_name, row.should_be_null(2)),
            row.field(row.w_warehouse_sq_ft, 3),
            row.field(row.w_address.get_street_number(), 4),
            row.field(row.w_address.get_street_name(), 5),
            row.field(row.w_address.get_street_type(), 6),
            row.field(row.w_address.get_suite_number(), 7),
            row.field(row.w_address.get_city(), 8),
            row.field(row.w_address.get_county().unwrap_or(""), 9),
            row.field(row.w_address.get_state(), 10),
            DatField::zip(row.w_address.get_zip(), row.should_be_null(11)),
            row.field(row.w_address.get_country(), 12),
            row.field(row.w_address.get_gmt_offset(), 13),
        )
    }
}
/// Writes [`WebPageRow`]s in CSV format.
///
/// # Example
/// ```
/// # use tpcdsgen::config::Session;
/// # use tpcdsgen::csv::WebPageCsv;
/// # use tpcdsgen::row::{GeneratedRow, RowGenerator, WebPageRowGenerator};
/// # use std::fmt::Write;
/// # let session = Session::default();
/// let mut generator = WebPageRowGenerator::new();
/// // Output the first 3 rows in CSV format
/// let mut csv = String::new();
/// writeln!(&mut csv, "{}", WebPageCsv::header()).unwrap(); // write header
/// # for row_number in 1..=3 {
/// #   let result = generator.generate_row_and_child_rows(row_number, &session, None, None).unwrap();
/// #   for row in result.get_rows() {
/// #     let GeneratedRow::WebPage(row) = row else { continue };
/// // write line using CSV formatter
/// writeln!(&mut csv, "{}", WebPageCsv::new(row)).unwrap();
/// #   }
/// #   generator.consume_remaining_seeds_for_row();
/// # }
/// assert_eq!(
///   csv,
///   "wp_web_page_sk,wp_web_page_id,wp_rec_start_date,wp_rec_end_date,wp_creation_date_sk,wp_access_date_sk,wp_autogen_flag,wp_customer_sk,wp_url,wp_type,wp_char_count,wp_link_count,wp_image_count,wp_max_ad_count\n\
///    1,AAAAAAAABAAAAAAA,1997-09-03,,2450810,2452620,Y,98539,http://www.foo.com,welcome,2531,8,3,4\n\
///    2,AAAAAAAACAAAAAAA,1997-09-03,2000-09-02,2450814,2452580,N,,http://www.foo.com,protected,1564,4,3,1\n\
///    3,AAAAAAAACAAAAAAA,2000-09-03,,2450814,2452611,N,,http://www.foo.com,feedback,1564,4,3,4\n"
/// );
/// ```
pub struct WebPageCsv<'a> {
    inner: &'a WebPageRow,
    delimiter: char,
}

impl<'a> WebPageCsv<'a> {
    /// Create a wrapper that formats `inner` with the default `,` delimiter
    pub fn new(inner: &'a WebPageRow) -> Self {
        Self {
            inner,
            delimiter: ',',
        }
    }

    /// Create a wrapper that formats `inner` with a custom delimiter
    pub fn with_delimiter(inner: &'a WebPageRow, delimiter: char) -> Self {
        Self { inner, delimiter }
    }

    /// Returns the CSV header for the web_page table
    pub fn header() -> &'static str {
        "wp_web_page_sk,wp_web_page_id,wp_rec_start_date,wp_rec_end_date,wp_creation_date_sk,wp_access_date_sk,wp_autogen_flag,wp_customer_sk,wp_url,wp_type,wp_char_count,wp_link_count,wp_image_count,wp_max_ad_count"
    }

    /// Returns the CSV header with a custom delimiter
    pub fn header_with_delimiter(delimiter: char) -> String {
        join_header(Self::header(), delimiter)
    }
}

impl Display for WebPageCsv<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::row::table_row::DatField;

        let d = self.delimiter;
        let row = self.inner;
        write!(
            f,
            "{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}",
            row.key_field(row.wp_page_sk, 0),
            row.field(&row.wp_page_id, 1),
            row.date_field(row.wp_rec_start_date_id, 2),
            row.date_field(row.wp_rec_end_date_id, 3),
            row.key_field(row.wp_creation_date_sk, 4),
            row.key_field(row.wp_access_date_sk, 5),
            DatField::yes_no(row.wp_autogen_flag, row.should_be_null(6)),
            row.key_field(row.wp_customer_sk, 7),
            row.field(&row.wp_url, 8),
            row.field(&row.wp_type, 9),
            row.field(row.wp_char_count, 10),
            row.field(row.wp_link_count, 11),
            row.field(row.wp_image_count, 12),
            row.field(row.wp_max_ad_count, 13),
        )
    }
}
/// Writes [`WebReturnsRow`]s in CSV format.
///
/// # Example
/// ```
/// # use tpcdsgen::config::Session;
/// # use tpcdsgen::csv::WebReturnsCsv;
/// # use tpcdsgen::row::{GeneratedRow, RowGenerator, WebSalesRowGenerator};
/// # use std::fmt::Write;
/// # let session = Session::default();
/// let mut generator = WebSalesRowGenerator::new();
/// // Output the first 3 rows in CSV format
/// let mut csv = String::new();
/// writeln!(&mut csv, "{}", WebReturnsCsv::header()).unwrap(); // write header
/// # for row_number in 1..=15 {
/// #   let result = generator.generate_row_and_child_rows(row_number, &session, None, None).unwrap();
/// #   for row in result.get_rows() {
/// #     let GeneratedRow::WebReturns(row) = row else { continue };
/// // write line using CSV formatter
/// writeln!(&mut csv, "{}", WebReturnsCsv::new(row)).unwrap();
/// #   }
/// #   if result.should_end_row() {
/// #     generator.consume_remaining_seeds_for_row();
/// #   }
/// # }
/// assert_eq!(
///   csv,
///   "wr_returned_date_sk,wr_returned_time_sk,wr_item_sk,wr_refunded_customer_sk,wr_refunded_cdemo_sk,wr_refunded_hdemo_sk,wr_refunded_addr_sk,wr_returning_customer_sk,wr_returning_cdemo_sk,wr_returning_hdemo_sk,wr_returning_addr_sk,wr_web_page_sk,wr_reason_sk,wr_order_number,wr_return_quantity,wr_return_amt,wr_return_tax,wr_return_amt_inc_tax,wr_fee,wr_return_ship_cost,wr_refunded_cash,wr_reversed_charge,wr_account_credit,wr_net_loss\n\
///    2451653,7022,10402,46224,1011635,3446,4057,46224,1011635,3446,4057,56,23,1,10,698.20,13.96,712.16,18.63,820.30,300.22,382.06,15.92,852.89\n\
///    2451627,64915,15464,3811,18405,199,48793,3811,18405,199,48793,13,9,1,47,1248.79,49.95,1298.74,61.81,709.23,262.24,128.25,858.30,820.99\n\
///    2452798,,9559,,31639,,18790,,31639,2038,18790,,11,10,11,,25.52,,,16.72,,16.36,165.47,\n"
/// );
/// ```
pub struct WebReturnsCsv<'a> {
    inner: &'a WebReturnsRow,
    delimiter: char,
}

impl<'a> WebReturnsCsv<'a> {
    /// Create a wrapper that formats `inner` with the default `,` delimiter
    pub fn new(inner: &'a WebReturnsRow) -> Self {
        Self {
            inner,
            delimiter: ',',
        }
    }

    /// Create a wrapper that formats `inner` with a custom delimiter
    pub fn with_delimiter(inner: &'a WebReturnsRow, delimiter: char) -> Self {
        Self { inner, delimiter }
    }

    /// Returns the CSV header for the web_returns table
    pub fn header() -> &'static str {
        "wr_returned_date_sk,wr_returned_time_sk,wr_item_sk,wr_refunded_customer_sk,wr_refunded_cdemo_sk,wr_refunded_hdemo_sk,wr_refunded_addr_sk,wr_returning_customer_sk,wr_returning_cdemo_sk,wr_returning_hdemo_sk,wr_returning_addr_sk,wr_web_page_sk,wr_reason_sk,wr_order_number,wr_return_quantity,wr_return_amt,wr_return_tax,wr_return_amt_inc_tax,wr_fee,wr_return_ship_cost,wr_refunded_cash,wr_reversed_charge,wr_account_credit,wr_net_loss"
    }

    /// Returns the CSV header with a custom delimiter
    pub fn header_with_delimiter(delimiter: char) -> String {
        join_header(Self::header(), delimiter)
    }
}

impl Display for WebReturnsCsv<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::generator::WebReturnsGeneratorColumn::*;

        let d = self.delimiter;
        let row = self.inner;

        write!(
            f,
            "{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}",
            row.field(row.wr_returned_date_sk, WrReturnedDateSk),
            row.field(row.wr_returned_time_sk, WrReturnedTimeSk),
            row.field(row.wr_item_sk, WrItemSk),
            row.field(row.wr_refunded_customer_sk, WrRefundedCustomerSk),
            row.field(row.wr_refunded_cdemo_sk, WrRefundedCdemoSk),
            row.field(row.wr_refunded_hdemo_sk, WrRefundedHdemoSk),
            row.field(row.wr_refunded_addr_sk, WrRefundedAddrSk),
            row.field(row.wr_returning_customer_sk, WrReturningCustomerSk),
            row.field(row.wr_returning_cdemo_sk, WrReturningCdemoSk),
            row.field(row.wr_returning_hdemo_sk, WrReturningHdemoSk),
            row.field(row.wr_returning_addr_sk, WrReturningAddrSk),
            row.field(row.wr_web_page_sk, WrWebPageSk),
            row.field(row.wr_reason_sk, WrReasonSk),
            row.field(row.wr_order_number, WrOrderNumber),
            row.field(row.wr_pricing.get_quantity(), WrPricingQuantity),
            row.field(row.wr_pricing.get_net_paid(), WrPricingNetPaid),
            row.field(row.wr_pricing.get_ext_tax(), WrPricingExtTax),
            row.field(
                row.wr_pricing.get_net_paid_including_tax(),
                WrPricingNetPaidIncTax
            ),
            row.field(row.wr_pricing.get_fee(), WrPricingFee),
            row.field(row.wr_pricing.get_ext_ship_cost(), WrPricingExtShipCost),
            row.field(row.wr_pricing.get_refunded_cash(), WrPricingRefundedCash),
            row.field(
                row.wr_pricing.get_reversed_charge(),
                WrPricingReversedCharge
            ),
            row.field(row.wr_pricing.get_store_credit(), WrPricingStoreCredit),
            row.field(row.wr_pricing.get_net_loss(), WrPricingNetLoss),
        )
    }
}
/// Writes [`WebSalesRow`]s in CSV format.
///
/// # Example
/// ```
/// # use tpcdsgen::config::Session;
/// # use tpcdsgen::csv::WebSalesCsv;
/// # use tpcdsgen::row::{GeneratedRow, RowGenerator, WebSalesRowGenerator};
/// # use std::fmt::Write;
/// # let session = Session::default();
/// let mut generator = WebSalesRowGenerator::new();
/// // Output the first 3 rows in CSV format
/// let mut csv = String::new();
/// writeln!(&mut csv, "{}", WebSalesCsv::header()).unwrap(); // write header
/// # for row_number in 1..=3 {
/// #   let result = generator.generate_row_and_child_rows(row_number, &session, None, None).unwrap();
/// #   for row in result.get_rows() {
/// #     let GeneratedRow::WebSales(row) = row else { continue };
/// // write line using CSV formatter
/// writeln!(&mut csv, "{}", WebSalesCsv::new(row)).unwrap();
/// #   }
/// #   if result.should_end_row() {
/// #     generator.consume_remaining_seeds_for_row();
/// #   }
/// # }
/// assert_eq!(
///   csv,
///   "ws_sold_date_sk,ws_sold_time_sk,ws_ship_date_sk,ws_item_sk,ws_bill_customer_sk,ws_bill_cdemo_sk,ws_bill_hdemo_sk,ws_bill_addr_sk,ws_ship_customer_sk,ws_ship_cdemo_sk,ws_ship_hdemo_sk,ws_ship_addr_sk,ws_web_page_sk,ws_web_site_sk,ws_ship_mode_sk,ws_warehouse_sk,ws_promo_sk,ws_order_number,ws_quantity,ws_wholesale_cost,ws_list_price,ws_sales_price,ws_ext_discount_amt,ws_ext_sales_price,ws_ext_wholesale_cost,ws_ext_list_price,ws_ext_tax,ws_coupon_amt,ws_ext_ship_cost,ws_net_paid,ws_net_paid_inc_tax,ws_net_paid_inc_ship,ws_net_paid_inc_ship_tax,ws_net_profit\n\
///    2451383,73313,2451482,4591,83074,596485,1096,40907,85919,41329,1140,1351,43,4,4,5,6,1,57,33.59,59.45,38.04,1220.37,2168.28,1914.63,3388.65,50.95,1149.18,575.70,1019.10,1070.05,1594.80,1645.75,-895.53\n\
///    2451383,73313,2451411,3566,83074,596485,1096,40907,85919,41329,1140,1351,28,7,3,2,271,1,38,29.83,48.92,26.41,855.38,1003.58,1133.54,1858.96,30.10,0.00,910.86,1003.58,1033.68,1914.44,1944.54,-129.96\n\
///    2451383,73313,2451413,7286,83074,596485,1096,40907,85919,41329,1140,1351,58,28,10,5,300,1,32,49.72,107.89,97.10,345.28,3107.20,1591.04,3452.48,124.28,0.00,828.48,3107.20,3231.48,3935.68,4059.96,1516.16\n"
/// );
/// ```
pub struct WebSalesCsv<'a> {
    inner: &'a WebSalesRow,
    delimiter: char,
}

impl<'a> WebSalesCsv<'a> {
    /// Create a wrapper that formats `inner` with the default `,` delimiter
    pub fn new(inner: &'a WebSalesRow) -> Self {
        Self {
            inner,
            delimiter: ',',
        }
    }

    /// Create a wrapper that formats `inner` with a custom delimiter
    pub fn with_delimiter(inner: &'a WebSalesRow, delimiter: char) -> Self {
        Self { inner, delimiter }
    }

    /// Returns the CSV header for the web_sales table
    pub fn header() -> &'static str {
        "ws_sold_date_sk,ws_sold_time_sk,ws_ship_date_sk,ws_item_sk,ws_bill_customer_sk,ws_bill_cdemo_sk,ws_bill_hdemo_sk,ws_bill_addr_sk,ws_ship_customer_sk,ws_ship_cdemo_sk,ws_ship_hdemo_sk,ws_ship_addr_sk,ws_web_page_sk,ws_web_site_sk,ws_ship_mode_sk,ws_warehouse_sk,ws_promo_sk,ws_order_number,ws_quantity,ws_wholesale_cost,ws_list_price,ws_sales_price,ws_ext_discount_amt,ws_ext_sales_price,ws_ext_wholesale_cost,ws_ext_list_price,ws_ext_tax,ws_coupon_amt,ws_ext_ship_cost,ws_net_paid,ws_net_paid_inc_tax,ws_net_paid_inc_ship,ws_net_paid_inc_ship_tax,ws_net_profit"
    }

    /// Returns the CSV header with a custom delimiter
    pub fn header_with_delimiter(delimiter: char) -> String {
        join_header(Self::header(), delimiter)
    }
}

impl Display for WebSalesCsv<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::generator::WebSalesGeneratorColumn::*;

        let d = self.delimiter;
        let row = self.inner;

        write!(
            f,
            "{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}",
            row.field(row.ws_sold_date_sk, WsSoldDateSk),
            row.field(row.ws_sold_time_sk, WsSoldTimeSk),
            row.field(row.ws_ship_date_sk, WsShipDateSk),
            row.field(row.ws_item_sk, WsItemSk),
            row.field(row.ws_bill_customer_sk, WsBillCustomerSk),
            row.field(row.ws_bill_cdemo_sk, WsBillCdemoSk),
            row.field(row.ws_bill_hdemo_sk, WsBillHdemoSk),
            row.field(row.ws_bill_addr_sk, WsBillAddrSk),
            row.field(row.ws_ship_customer_sk, WsShipCustomerSk),
            row.field(row.ws_ship_cdemo_sk, WsShipCdemoSk),
            row.field(row.ws_ship_hdemo_sk, WsShipHdemoSk),
            row.field(row.ws_ship_addr_sk, WsShipAddrSk),
            row.field(row.ws_web_page_sk, WsWebPageSk),
            row.field(row.ws_web_site_sk, WsWebSiteSk),
            row.field(row.ws_ship_mode_sk, WsShipModeSk),
            row.field(row.ws_warehouse_sk, WsWarehouseSk),
            row.field(row.ws_promo_sk, WsPromoSk),
            row.field(row.ws_order_number, WsOrderNumber),
            row.field(row.ws_pricing.get_quantity(), WsPricingQuantity),
            row.field(row.ws_pricing.get_wholesale_cost(), WsPricingWholesaleCost),
            row.field(row.ws_pricing.get_list_price(), WsPricingListPrice),
            row.field(row.ws_pricing.get_sales_price(), WsPricingSalesPrice),
            row.field(row.ws_pricing.get_ext_discount_amount(), WsPricingExtDiscountAmt),
            row.field(row.ws_pricing.get_ext_sales_price(), WsPricingExtSalesPrice),
            row.field(row.ws_pricing.get_ext_wholesale_cost(), WsPricingExtWholesaleCost),
            row.field(row.ws_pricing.get_ext_list_price(), WsPricingExtListPrice),
            row.field(row.ws_pricing.get_ext_tax(), WsPricingExtTax),
            row.field(row.ws_pricing.get_coupon_amount(), WsPricingCouponAmt),
            row.field(row.ws_pricing.get_ext_ship_cost(), WsPricingExtShipCost),
            row.field(row.ws_pricing.get_net_paid(), WsPricingNetPaid),
            row.field(row.ws_pricing.get_net_paid_including_tax(), WsPricingNetPaidIncTax),
            row.field(row.ws_pricing.get_net_paid_including_shipping(), WsPricingNetPaidIncShip),
            row.field(row.ws_pricing.get_net_paid_including_shipping_and_tax(), WsPricingNetPaidIncShipTax),
            row.field(row.ws_pricing.get_net_profit(), WsPricingNetProfit),
        )
    }
}
/// Writes [`WebSiteRow`]s in CSV format.
///
/// # Example
/// ```
/// # use tpcdsgen::config::Session;
/// # use tpcdsgen::csv::WebSiteCsv;
/// # use tpcdsgen::row::{GeneratedRow, RowGenerator, WebSiteRowGenerator};
/// # use std::fmt::Write;
/// # let session = Session::default();
/// let mut generator = WebSiteRowGenerator::new();
/// // Output the first 3 rows in CSV format
/// let mut csv = String::new();
/// writeln!(&mut csv, "{}", WebSiteCsv::header()).unwrap(); // write header
/// # for row_number in 1..=3 {
/// #   let result = generator.generate_row_and_child_rows(row_number, &session, None, None).unwrap();
/// #   for row in result.get_rows() {
/// #     let GeneratedRow::WebSite(row) = row else { continue };
/// // write line using CSV formatter
/// writeln!(&mut csv, "{}", WebSiteCsv::new(row)).unwrap();
/// #   }
/// #   generator.consume_remaining_seeds_for_row();
/// # }
/// assert_eq!(
///   csv,
///   "web_site_sk,web_site_id,web_rec_start_date,web_rec_end_date,web_name,web_open_date_sk,web_close_date_sk,web_class,web_manager,web_mkt_id,web_mkt_class,web_mkt_desc,web_market_manager,web_company_id,web_company_name,web_street_number,web_street_name,web_street_type,web_suite_number,web_city,web_county,web_state,web_zip,web_country,web_gmt_offset,web_tax_percentage\n\
///    1,AAAAAAAABAAAAAAA,1997-08-16,,site_0,2450807,,Unknown,Ronald Shaffer,4,\"Grey lines ought to result indeed centres. Tod\",\"Well similar decisions used to keep hardly democratic, personal priorities.\",Joe George,6,cally,51,Dogwood Sunset,Ln,Suite 330,Midway,Williamson County,TN,31904,United States,-5,0.10\n\
///    2,AAAAAAAACAAAAAAA,1997-08-16,2000-08-15,site_0,2450798,2447148,Unknown,Tommy Jones,6,\"Completely excellent things ought to pro\",\"Lucky passengers know. Red details will not hang alive, international s\",David Myers,4,ese,358,Ridge Wilson,Cir.,Suite 150,Midway,Williamson County,TN,31904,United States,-5,0.00\n\
///    3,AAAAAAAACAAAAAAA,2000-08-16,,site_0,2450798,2447148,Unknown,Tommy Jones,3,\"Completely excellent things ought to pro\",\"Particular, common seasons shall not indicate fully more single decisions; \",David Myers,4,ese,753,7th ,Pkwy,Suite 210,Midway,Williamson County,TN,31904,United States,-5,0.02\n"
/// );
/// ```
pub struct WebSiteCsv<'a> {
    inner: &'a WebSiteRow,
    delimiter: char,
}

impl<'a> WebSiteCsv<'a> {
    /// Create a wrapper that formats `inner` with the default `,` delimiter
    pub fn new(inner: &'a WebSiteRow) -> Self {
        Self {
            inner,
            delimiter: ',',
        }
    }

    /// Create a wrapper that formats `inner` with a custom delimiter
    pub fn with_delimiter(inner: &'a WebSiteRow, delimiter: char) -> Self {
        Self { inner, delimiter }
    }

    /// Returns the CSV header for the web_site table
    pub fn header() -> &'static str {
        "web_site_sk,web_site_id,web_rec_start_date,web_rec_end_date,web_name,web_open_date_sk,web_close_date_sk,web_class,web_manager,web_mkt_id,web_mkt_class,web_mkt_desc,web_market_manager,web_company_id,web_company_name,web_street_number,web_street_name,web_street_type,web_suite_number,web_city,web_county,web_state,web_zip,web_country,web_gmt_offset,web_tax_percentage"
    }

    /// Returns the CSV header with a custom delimiter
    pub fn header_with_delimiter(delimiter: char) -> String {
        join_header(Self::header(), delimiter)
    }
}

impl Display for WebSiteCsv<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::generator::WebSiteGeneratorColumn::*;
        use crate::row::table_row::{CsvQuoted, DatField};

        let d = self.delimiter;
        let row = self.inner;

        write!(
            f,
            "{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}{d}{}",
            row.key_field(row.web_site_sk, WebSiteSk),
            row.field(&row.web_site_id, WebSiteId),
            row.date_field(row.web_rec_start_date_id, WebRecStartDateId),
            row.date_field(row.web_rec_end_date_id, WebRecEndDateId),
            row.field(&row.web_name, WebName),
            row.key_field(row.web_open_date, WebOpenDate),
            row.key_field(row.web_close_date, WebCloseDate),
            row.field(&row.web_class, WebClass),
            row.field(&row.web_manager, WebManager),
            row.field(row.web_market_id, WebMarketId),
            CsvQuoted::new(&row.web_market_class, row.is_null_at(WebMarketClass)),
            CsvQuoted::new(&row.web_market_desc, row.is_null_at(WebMarketDesc)),
            row.field(&row.web_market_manager, WebMarketManager),
            row.field(row.web_company_id, WebCompanyId),
            row.field(&row.web_company_name, WebCompanyName),
            row.field(row.web_address.get_street_number(), WebAddressStreetNum),
            row.field(row.web_address.get_street_name(), WebAddressStreetName1),
            row.field(row.web_address.get_street_type(), WebAddressStreetType),
            row.field(row.web_address.get_suite_number(), WebAddressSuiteNum),
            row.field(row.web_address.get_city(), WebAddressCity),
            row.field(
                row.web_address.get_county().unwrap_or(""),
                WebAddressCounty
            ),
            row.field(row.web_address.get_state(), WebAddressState),
            DatField::zip(row.web_address.get_zip(), row.is_null_at(WebAddressZip)),
            row.field(row.web_address.get_country(), WebAddressCountry),
            row.field(row.web_address.get_gmt_offset(), WebAddressGmtOffset),
            row.field(row.web_tax_percentage, WebTaxPercentage),
        )
    }
}

/// Writes any [`GeneratedRow`] in CSV format, delegating to the variant's
/// table-specific formatting.
pub struct GeneratedRowCsv<'a> {
    inner: &'a GeneratedRow,
    delimiter: char,
}

impl<'a> GeneratedRowCsv<'a> {
    /// Create a wrapper that formats `inner` with the default `,` delimiter
    pub fn new(inner: &'a GeneratedRow) -> Self {
        Self {
            inner,
            delimiter: ',',
        }
    }

    /// Create a wrapper that formats `inner` with a custom delimiter
    pub fn with_delimiter(inner: &'a GeneratedRow, delimiter: char) -> Self {
        Self { inner, delimiter }
    }
}

impl Display for GeneratedRowCsv<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = self.delimiter;
        match self.inner {
            GeneratedRow::CallCenter(row) => CallCenterCsv::with_delimiter(row, d).fmt(f),
            GeneratedRow::CatalogPage(row) => CatalogPageCsv::with_delimiter(row, d).fmt(f),
            GeneratedRow::CatalogReturns(row) => CatalogReturnsCsv::with_delimiter(row, d).fmt(f),
            GeneratedRow::CatalogSales(row) => CatalogSalesCsv::with_delimiter(row, d).fmt(f),
            GeneratedRow::Customer(row) => CustomerCsv::with_delimiter(row, d).fmt(f),
            GeneratedRow::CustomerAddress(row) => CustomerAddressCsv::with_delimiter(row, d).fmt(f),
            GeneratedRow::CustomerDemographics(row) => {
                CustomerDemographicsCsv::with_delimiter(row, d).fmt(f)
            }
            GeneratedRow::DateDim(row) => DateDimCsv::with_delimiter(row, d).fmt(f),
            GeneratedRow::DbgenVersion(row) => DbgenVersionCsv::with_delimiter(row, d).fmt(f),
            GeneratedRow::HouseholdDemographics(row) => {
                HouseholdDemographicsCsv::with_delimiter(row, d).fmt(f)
            }
            GeneratedRow::IncomeBand(row) => IncomeBandCsv::with_delimiter(row, d).fmt(f),
            GeneratedRow::Inventory(row) => InventoryCsv::with_delimiter(row, d).fmt(f),
            GeneratedRow::Item(row) => ItemCsv::with_delimiter(row, d).fmt(f),
            GeneratedRow::Promotion(row) => PromotionCsv::with_delimiter(row, d).fmt(f),
            GeneratedRow::Reason(row) => ReasonCsv::with_delimiter(row, d).fmt(f),
            GeneratedRow::ShipMode(row) => ShipModeCsv::with_delimiter(row, d).fmt(f),
            GeneratedRow::Store(row) => StoreCsv::with_delimiter(row, d).fmt(f),
            GeneratedRow::StoreReturns(row) => StoreReturnsCsv::with_delimiter(row, d).fmt(f),
            GeneratedRow::StoreSales(row) => StoreSalesCsv::with_delimiter(row, d).fmt(f),
            GeneratedRow::TimeDim(row) => TimeDimCsv::with_delimiter(row, d).fmt(f),
            GeneratedRow::Warehouse(row) => WarehouseCsv::with_delimiter(row, d).fmt(f),
            GeneratedRow::WebPage(row) => WebPageCsv::with_delimiter(row, d).fmt(f),
            GeneratedRow::WebReturns(row) => WebReturnsCsv::with_delimiter(row, d).fmt(f),
            GeneratedRow::WebSales(row) => WebSalesCsv::with_delimiter(row, d).fmt(f),
            GeneratedRow::WebSite(row) => WebSiteCsv::with_delimiter(row, d).fmt(f),
        }
    }
}

/// Returns the CSV header line for `table` with the given delimiter, or
/// `None` for source tables, which have no CSV output.
pub fn csv_header(table: Table, delimiter: char) -> Option<String> {
    let header = match table {
        Table::CallCenter => CallCenterCsv::header(),
        Table::CatalogPage => CatalogPageCsv::header(),
        Table::CatalogReturns => CatalogReturnsCsv::header(),
        Table::CatalogSales => CatalogSalesCsv::header(),
        Table::Customer => CustomerCsv::header(),
        Table::CustomerAddress => CustomerAddressCsv::header(),
        Table::CustomerDemographics => CustomerDemographicsCsv::header(),
        Table::DateDim => DateDimCsv::header(),
        Table::DbgenVersion => DbgenVersionCsv::header(),
        Table::HouseholdDemographics => HouseholdDemographicsCsv::header(),
        Table::IncomeBand => IncomeBandCsv::header(),
        Table::Inventory => InventoryCsv::header(),
        Table::Item => ItemCsv::header(),
        Table::Promotion => PromotionCsv::header(),
        Table::Reason => ReasonCsv::header(),
        Table::ShipMode => ShipModeCsv::header(),
        Table::Store => StoreCsv::header(),
        Table::StoreReturns => StoreReturnsCsv::header(),
        Table::StoreSales => StoreSalesCsv::header(),
        Table::TimeDim => TimeDimCsv::header(),
        Table::Warehouse => WarehouseCsv::header(),
        Table::WebPage => WebPageCsv::header(),
        Table::WebReturns => WebReturnsCsv::header(),
        Table::WebSales => WebSalesCsv::header(),
        Table::WebSite => WebSiteCsv::header(),
        _ => return None,
    };
    Some(join_header(header, delimiter))
}

/// Join the comma-separated column names of `header` with `delimiter`,
/// double-quoting any name that contains the delimiter (for example `_`).
fn join_header(header: &str, delimiter: char) -> String {
    let mut out = String::with_capacity(header.len() + 16);
    for (i, name) in header.split(',').enumerate() {
        if i > 0 {
            out.push(delimiter);
        }
        if name.contains(delimiter) {
            out.push('"');
            out.push_str(name);
            out.push('"');
        } else {
            out.push_str(name);
        }
    }
    out
}
