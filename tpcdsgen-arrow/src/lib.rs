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
