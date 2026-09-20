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
