//! Generate TPC-DS data as Arrow RecordBatches
//!
//! This crate provides generators for TPC-DS tables that directly produce
//! Arrow [`RecordBatch`](arrow::array::RecordBatch)es. This is significantly faster than generating DAT or CSV
//! files and then parsing them into Arrow.
//!
//! # Example
//! ```
//! # use tpcdsgen::config::SessionBuilder;
//! # use tpcdsgen_arrow::StoreSalesArrow;
//! # use tpcdsgen_arrow::arrow::util::pretty::pretty_format_batches;
//! // Create a SF=1 session for the store_sales table
//! let session = SessionBuilder::new().with_scale_factor(1.0).build().unwrap();
//! let mut arrow_generator = StoreSalesArrow::new(session)
//!   .with_batch_size(10);
//! // The generator is a Rust iterator, producing RecordBatch
//! let batch = arrow_generator.next().unwrap().unwrap();
//! // compare the output by pretty printing it
//! let formatted_batches = pretty_format_batches(&[batch]).unwrap().to_string();
//! assert_eq!(formatted_batches.lines().collect::<Vec<_>>(), vec![
//!   "+-----------------+-----------------+------------+----------------+-------------+-------------+------------+-------------+-------------+------------------+-------------+-------------------+---------------+----------------+---------------------+--------------------+-----------------------+-------------------+------------+---------------+-------------+---------------------+---------------+",
//!   "| ss_sold_date_sk | ss_sold_time_sk | ss_item_sk | ss_customer_sk | ss_cdemo_sk | ss_hdemo_sk | ss_addr_sk | ss_store_sk | ss_promo_sk | ss_ticket_number | ss_quantity | ss_wholesale_cost | ss_list_price | ss_sales_price | ss_ext_discount_amt | ss_ext_sales_price | ss_ext_wholesale_cost | ss_ext_list_price | ss_ext_tax | ss_coupon_amt | ss_net_paid | ss_net_paid_inc_tax | ss_net_profit |",
//!   "+-----------------+-----------------+------------+----------------+-------------+-------------+------------+-------------+-------------+------------------+-------------+-------------------+---------------+----------------+---------------------+--------------------+-----------------------+-------------------+------------+---------------+-------------+---------------------+---------------+",
//!   "| 2451813         | 65495           | 3617       | 67006          | 591617      | 3428        | 24839      | 10          | 161         | 1                | 79          | 11.41             | 18.71         | 2.80           | 99.54               | 221.20             | 901.39                | 1478.09           | 6.08       | 99.54         | 121.66      | 127.74              | -779.73       |",
//!   "| 2451813         | 65495           | 13283      | 67006          | 591617      | 3428        | 24839      | 10          | 154         | 1                | 37          | 63.63             | 101.17        | 41.47          | 46.03               | 1534.39            | 2354.31               | 3743.29           | 59.53      | 46.03         | 1488.36     | 1547.89             | -865.95       |",
//!   "| 2451813         | 65495           | 13631      | 67006          | 591617      | 3428        | 24839      | 10          | 172         | 1                | 99          | 80.52             | 137.68        | 83.98          | 0.00                | 8314.02            | 7971.48               | 13630.32          | 0.00       | 0.00          | 8314.02     | 8314.02             | 342.54        |",
//!   "| 2451813         | 65495           | 5981       | 67006          | 591617      | 3428        | 24839      | 10          | 280         | 1                | 14          | 57.37             | 76.30         | 6.10           | 0.00                | 85.40              | 803.18                | 1068.20           | 0.00       | 0.00          | 85.40       | 85.40               | -717.78       |",
//!   "| 2451813         | 65495           | 4553       | 67006          | 591617      | 3428        | 24839      | 10          | 236         | 1                | 100         | 25.08             | 36.86         | 0.73           | 0.00                | 73.00              | 2508.00               | 3686.00           | 6.57       | 0.00          | 73.00       | 79.57               | -2435.00      |",
//!   "| 2451813         | 65495           | 10993      | 67006          | 591617      | 3428        | 24839      | 10          | 263         | 1                | 91          | 93.48             | 108.43        | 93.24          | 0.00                | 8484.84            | 8506.68               | 9867.13           | 254.54     | 0.00          | 8484.84     | 8739.38             | -21.84        |",
//!   "| 2451813         | 65495           | 49         | 67006          | 591617      | 3428        | 24839      | 10          | 70          | 1                | 5           | 10.68             | 15.91         | 6.68           | 0.00                | 33.40              | 53.40                 | 79.55             | 2.33       | 0.00          | 33.40       | 35.73               | -20.00        |",
//!   "| 2451813         | 65495           | 4583       | 67006          | 591617      | 3428        | 24839      | 10          | 267         | 1                | 72          | 84.72             | 111.83        | 61.50          | 0.00                | 4428.00            | 6099.84               | 8051.76           | 177.12     | 0.00          | 4428.00     | 4605.12             | -1671.84      |",
//!   "| 2451813         | 65495           | 13538      | 67006          | 591617      | 3428        | 24839      | 10          | 106         | 1                | 14          | 11.54             | 11.77         | 0.00           | 0.00                | 0.00               | 161.56                | 164.78            | 0.00       | 0.00          | 0.00        | 0.00                | -161.56       |",
//!   "| 2451813         | 65495           | 3248       | 67006          | 591617      | 3428        | 24839      | 10          | 189         | 1                | 58          | 4.57              | 5.34          | 3.52           | 0.00                | 204.16             | 265.06                | 309.72            | 0.00       | 0.00          | 204.16      | 204.16              | -60.90        |",
//!   "+-----------------+-----------------+------------+----------------+-------------+-------------+------------+-------------+-------------+------------------+-------------+-------------------+---------------+----------------+---------------------+--------------------+-----------------------+-------------------+------------+---------------+-------------+---------------------+---------------+"
//! ]);
//! ```
//!
//! # Feature Flags
//!
//! * `arrow_60` - build against [Arrow 60.x] (default)
//! * `arrow_59` - build against [Arrow 59.x]
//!
//! Pick the one that matches your project's arrow version. For arrow 59, also
//! disable the default features so that arrow 60 is not compiled as well:
//!
//! ```toml
//! # arrow 60
//! tpcdsgen-arrow = "..."
//! # arrow 59
//! tpcdsgen-arrow = { version = "...", default-features = false, features = ["arrow_59"] }
//! ```
//!
//! If both are enabled, arrow 59 is used. The selected version is re-exported
//! as [`arrow`].
//!
//! [Arrow 59.x]: https://docs.rs/arrow/59
//! [Arrow 60.x]: https://docs.rs/arrow/60

// Alias the selected arrow version as `arrow`.
#[cfg(feature = "arrow_59")]
pub extern crate arrow_59 as arrow;
#[cfg(all(feature = "arrow_60", not(feature = "arrow_59")))]
pub extern crate arrow_60 as arrow;
#[cfg(not(any(feature = "arrow_59", feature = "arrow_60")))]
compile_error!("one of the `arrow_60` (default) or `arrow_59` features must be enabled");

pub mod conversions;
mod tables;

pub use tables::{
    CallCenterArrow, CatalogPageArrow, CatalogReturnsArrow, CatalogSalesArrow,
    CustomerAddressArrow, CustomerArrow, CustomerDemographicsArrow, DateDimArrow,
    DbgenVersionArrow, HouseholdDemographicsArrow, IncomeBandArrow, InventoryArrow, ItemArrow,
    PromotionArrow, ReasonArrow, ShipModeArrow, StoreArrow, StoreReturnsArrow, StoreSalesArrow,
    TimeDimArrow, WarehouseArrow, WebPageArrow, WebReturnsArrow, WebSalesArrow, WebSiteArrow,
};

/// Default number of rows per [`RecordBatch`](arrow::array::RecordBatch).
pub const DEFAULT_BATCH_SIZE: usize = 8_000;
