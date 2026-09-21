//! Generate TPCH data as Arrow RecordBatches
//!
//! This crate provides generators for TPCH tables that directly produces
//! Arrow [`RecordBatch`](arrow::array::RecordBatch)es. This is significantly faster than generating TBL or CSV
//! files and then parsing them into Arrow.
//!
//! # Example
//! ```
//! # use tpchgen::generators::LineItemGenerator;
//! # use tpchgen_arrow::LineItemArrow;
//! # use tpchgen_arrow::arrow::util::pretty::pretty_format_batches;
//! // Create a SF=1 generator for the LineItem table
//! let generator = LineItemGenerator::new(1.0, 1, 1);
//! let mut arrow_generator = LineItemArrow::new(generator)
//!   .with_batch_size(10);
//! // The generator is a Rust iterator, producing RecordBatch
//! let batch = arrow_generator.next().unwrap().unwrap();
//! // compare the output by pretty printing it
//! let formatted_batches = pretty_format_batches(&[batch]).unwrap().to_string();
//! assert_eq!(formatted_batches.lines().collect::<Vec<_>>(), vec![
//!   "+------------+-----------+-----------+--------------+------------+-----------------+------------+-------+--------------+--------------+------------+--------------+---------------+-------------------+------------+-------------------------------------+",
//!   "| l_orderkey | l_partkey | l_suppkey | l_linenumber | l_quantity | l_extendedprice | l_discount | l_tax | l_returnflag | l_linestatus | l_shipdate | l_commitdate | l_receiptdate | l_shipinstruct    | l_shipmode | l_comment                           |",
//!   "+------------+-----------+-----------+--------------+------------+-----------------+------------+-------+--------------+--------------+------------+--------------+---------------+-------------------+------------+-------------------------------------+",
//!   "| 1          | 155190    | 7706      | 1            | 17.00      | 21168.23        | 0.04       | 0.02  | N            | O            | 1996-03-13 | 1996-02-12   | 1996-03-22    | DELIVER IN PERSON | TRUCK      | egular courts above the             |",
//!   "| 1          | 67310     | 7311      | 2            | 36.00      | 45983.16        | 0.09       | 0.06  | N            | O            | 1996-04-12 | 1996-02-28   | 1996-04-20    | TAKE BACK RETURN  | MAIL       | ly final dependencies: slyly bold   |",
//!   "| 1          | 63700     | 3701      | 3            | 8.00       | 13309.60        | 0.10       | 0.02  | N            | O            | 1996-01-29 | 1996-03-05   | 1996-01-31    | TAKE BACK RETURN  | REG AIR    | riously. regular, express dep       |",
//!   "| 1          | 2132      | 4633      | 4            | 28.00      | 28955.64        | 0.09       | 0.06  | N            | O            | 1996-04-21 | 1996-03-30   | 1996-05-16    | NONE              | AIR        | lites. fluffily even de             |",
//!   "| 1          | 24027     | 1534      | 5            | 24.00      | 22824.48        | 0.10       | 0.04  | N            | O            | 1996-03-30 | 1996-03-14   | 1996-04-01    | NONE              | FOB        |  pending foxes. slyly re            |",
//!   "| 1          | 15635     | 638       | 6            | 32.00      | 49620.16        | 0.07       | 0.02  | N            | O            | 1996-01-30 | 1996-02-07   | 1996-02-03    | DELIVER IN PERSON | MAIL       | arefully slyly ex                   |",
//!   "| 2          | 106170    | 1191      | 1            | 38.00      | 44694.46        | 0.00       | 0.05  | N            | O            | 1997-01-28 | 1997-01-14   | 1997-02-02    | TAKE BACK RETURN  | RAIL       | ven requests. deposits breach a     |",
//!   "| 3          | 4297      | 1798      | 1            | 45.00      | 54058.05        | 0.06       | 0.00  | R            | F            | 1994-02-02 | 1994-01-04   | 1994-02-23    | NONE              | AIR        | ongside of the furiously brave acco |",
//!   "| 3          | 19036     | 6540      | 2            | 49.00      | 46796.47        | 0.10       | 0.00  | R            | F            | 1993-11-09 | 1993-12-20   | 1993-11-24    | TAKE BACK RETURN  | RAIL       |  unusual accounts. eve              |",
//!   "| 3          | 128449    | 3474      | 3            | 27.00      | 39890.88        | 0.06       | 0.07  | A            | F            | 1994-01-16 | 1993-11-22   | 1994-01-23    | DELIVER IN PERSON | SHIP       | nal foxes wake.                     |",
//!   "+------------+-----------+-----------+--------------+------------+-----------------+------------+-------+--------------+--------------+------------+--------------+---------------+-------------------+------------+-------------------------------------+"
//! ]);
//! ```
//!
//! # Feature Flags
//! This crate supports multiple versions of the Arrow crate via feature flags.
//! * `arrow_60` - Use [Arrow 60.x] (default)
//! * `arrow_59` - Use [Arrow 59.x]
//!
//! Feature flags are additive and `arrow_60` is a default feature, so selecting
//! `arrow_59` requires `default-features = false`. If both are enabled the
//! newer version wins.
//!
//! The selected version is re-exported as [`arrow`] for
//! downstream crates to use.
//!
//! [Arrow 59.x]: https://crates.io/crates/arrow/59.0.0
//! [Arrow 60.x]: https://crates.io/crates/arrow/60.0.0

// alias the selected arrow version as `arrow`
#[cfg(all(feature = "arrow_59", not(feature = "arrow_60")))]
pub extern crate arrow_59 as arrow;
#[cfg(feature = "arrow_60")]
pub extern crate arrow_60 as arrow;
#[cfg(not(any(feature = "arrow_59", feature = "arrow_60")))]
compile_error!("exactly one of the `arrow_59` or `arrow_60` features must be enabled");

pub mod conversions;
mod customer;
mod lineitem;
mod nation;
mod order;
mod part;
mod partsupp;
mod region;
mod supplier;

pub use customer::CustomerArrow;
pub use lineitem::LineItemArrow;
pub use nation::NationArrow;
pub use order::OrderArrow;
pub use part::PartArrow;
pub use partsupp::PartSuppArrow;
pub use region::RegionArrow;
pub use supplier::SupplierArrow;

/// The default number of rows in each Batch
pub const DEFAULT_BATCH_SIZE: usize = 8 * 1000;
