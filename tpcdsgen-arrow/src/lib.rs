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
//! * `arrow_60` - build against [Arrow 60.x] (default)
//! * `arrow_59` - build against [Arrow 59.x]. Use with `default-features = false`
//!   so that arrow 60 is not compiled as well.
//!
//! Note that the arrow dependency is re-exported as [`arrow`] so downstream
//! crates can use it without an explicit dependence.
//!
//! [arrow-rs]: https://github.com/apache/arrow-rs
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
