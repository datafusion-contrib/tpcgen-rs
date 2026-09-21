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

// The arrow version is selected by the `arrow_59` / `arrow_60` feature flags.
// The dependency is renamed so both versions can be declared at once; alias the
// selected one back to `arrow` so the rest of the crate (and downstream
// crates, via `tpcdsgen_arrow::arrow`) can use it under its normal name. If
// both features are enabled (`--all-features`, or feature unification with
// another crate) the newer version wins.
#[cfg(all(feature = "arrow_59", not(feature = "arrow_60")))]
pub extern crate arrow_59 as arrow;
#[cfg(feature = "arrow_60")]
pub extern crate arrow_60 as arrow;
#[cfg(not(any(feature = "arrow_59", feature = "arrow_60")))]
compile_error!("exactly one of the `arrow_59` or `arrow_60` features must be enabled");

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
