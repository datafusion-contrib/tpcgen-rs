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
//! This crate supports multiple versions of [arrow-rs] via feature flags.
//! Exactly one must be enabled:
//! * `arrow_60` - build against [Arrow 60.x] (default)
//! * `arrow_59` - build against [Arrow 59.x] (requires `default-features = false`)
//!
//! Enabling both at once is a compile error. The selected version is re-exported
//! as [`arrow`], so downstream crates can name the matching types without
//! guessing.
//!
//! [arrow-rs]: https://github.com/apache/arrow-rs
//! [Arrow 59.x]: https://docs.rs/arrow/59
//! [Arrow 60.x]: https://docs.rs/arrow/60

// Exactly one arrow version must be selected; see the feature flag docs above.
#[cfg(all(feature = "arrow_59", feature = "arrow_60"))]
compile_error!("the `arrow_59` and `arrow_60` features are mutually exclusive");
#[cfg(not(any(feature = "arrow_59", feature = "arrow_60")))]
compile_error!("one of the `arrow_59` or `arrow_60` features must be enabled");

// Alias the selected arrow version as `arrow`. The `arrow_59` arm also tests
// `arrow_60` so that enabling both reports only the error above, and not also a
// duplicate definition of `arrow`.
#[cfg(all(feature = "arrow_59", not(feature = "arrow_60")))]
pub extern crate arrow_59 as arrow;
#[cfg(feature = "arrow_60")]
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
