//! Generate TPC-DS data as Apache Arrow [`RecordBatch`](arrow::array::RecordBatch)es.
//!
//! This crate wraps the [`tpcdsgen`] row generators and produces typed Arrow
//! arrays directly — bypassing the intermediate string formatting step —
//! for significantly faster ingestion into Arrow-based engines.
//!
//! # Example
//! ```
//! use tpcdsgen::config::Session;
//! use tpcdsgen_arrow::ReasonArrow;
//!
//! let session = Session::default();
//! let mut gen = ReasonArrow::new(session).with_batch_size(100);
//! let batch = gen.next().unwrap().unwrap();
//! assert_eq!(batch.num_columns(), 3);
//! ```
//!
//! # Feature Flags
//!
//! This crate builds against [arrow-rs] 60 by default. One feature flag changes
//! that:
//! * `arrow_59` - build against [Arrow 59.x] instead of [Arrow 60.x]
//!
//! The selected version is re-exported as [`arrow`], so downstream crates can
//! name the matching types without guessing.
//!
//! [arrow-rs]: https://github.com/apache/arrow-rs
//! [Arrow 59.x]: https://docs.rs/arrow/59
//! [Arrow 60.x]: https://docs.rs/arrow/60

// Alias the selected arrow version as `arrow`.
#[cfg(feature = "arrow_59")]
pub extern crate arrow_59 as arrow;
#[cfg(not(feature = "arrow_59"))]
pub extern crate arrow_60 as arrow;

pub mod conversions;
mod tables;

pub(crate) use tpcdsgen::row::RowIter;

pub use tables::{
    CallCenterArrow, CatalogPageArrow, CatalogReturnsArrow, CatalogSalesArrow,
    CustomerAddressArrow, CustomerArrow, CustomerDemographicsArrow, DateDimArrow,
    DbgenVersionArrow, HouseholdDemographicsArrow, IncomeBandArrow, InventoryArrow, ItemArrow,
    PromotionArrow, ReasonArrow, ShipModeArrow, StoreArrow, StoreReturnsArrow, StoreSalesArrow,
    TimeDimArrow, WarehouseArrow, WebPageArrow, WebReturnsArrow, WebSalesArrow, WebSiteArrow,
};

/// Default number of rows per [`RecordBatch`](arrow::array::RecordBatch).
pub const DEFAULT_BATCH_SIZE: usize = 8_000;
