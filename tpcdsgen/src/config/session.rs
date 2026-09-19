use crate::config::{CompatMode, Scaling, Table};
use crate::error::{InvalidOptionError, Result};
use std::ops::RangeInclusive;

/// Threshold above which tables are not split across chunks. For tables with
/// fewer than this many rows, chunk 1 generates the whole table . Matches the
/// C `dsdgen`'s `kTotalRows < 1000000` check in `tools/parallel.c` [1] and
/// Trino's `Parallel.SMALL_TABLE_THRESHOLD` [2].
///
/// [1]: https://github.com/gregrahn/tpcds-kit/blob/5a3a81796992b725c2a8b216767e142609966752/tools/parallel.c#L75
/// [2]: https://github.com/trinodb/tpcds/blob/b594136818cc95bd6b34a352327611b329017281/src/main/java/io/trino/tpcds/Parallel.java#L28
const SMALL_TABLE_ROW_THRESHOLD: u64 = 1_000_000;

/// Split `total_rows` into `total_chunks` pieces and return the 1-based
/// `(first_row, row_count)` for `chunk_number`.
///
/// Ports the C `dsdgen`'s `split_work` (`tools/parallel.c`) [1] / Trino's
/// `Parallel.splitWork` [2].
///
/// [1]: https://github.com/gregrahn/tpcds-kit/blob/5a3a81796992b725c2a8b216767e142609966752/tools/parallel.c#L58-L107
/// [2]: https://github.com/trinodb/tpcds/blob/b594136818cc95bd6b34a352327611b329017281/src/main/java/io/trino/tpcds/Parallel.java#L24-L51
fn split_work(total_rows: u64, chunk_number: i32, total_chunks: i32) -> (u64, u64) {
    if total_rows < SMALL_TABLE_ROW_THRESHOLD {
        return if chunk_number == 1 {
            (1, total_rows)
        } else {
            (1, 0)
        };
    }

    let total_chunks = total_chunks as u64;
    let chunk_number = chunk_number as u64;
    let rowset_size = total_rows / total_chunks;
    let extra_rows = total_rows % total_chunks;

    let first_row = {
        let offset = 1 + (chunk_number - 1) * rowset_size;
        if extra_rows > 0 && chunk_number > 1 {
            offset + (chunk_number - 1).min(extra_rows)
        } else {
            offset
        }
    };

    let row_count = {
        if extra_rows > 0 && chunk_number <= extra_rows {
            rowset_size + 1
        } else {
            rowset_size
        }
    };

    (first_row, row_count)
}

/// Configuration for a TPC-DS data generation run.
///
/// A `Session` defines how TPC-DS data is generated.
#[derive(Debug, Clone)]
pub struct Session {
    scaling: Scaling,
    table: Option<Table>,
    no_sexism: bool,
    chunk_number: i32,
    total_chunks: i32,
    partitioned: bool,
    compat_mode: CompatMode,
    command_line_arguments: Option<String>,
}

impl Default for Session {
    fn default() -> Self {
        Session {
            scaling: Scaling::new_with_compat(Self::DEFAULT_SCALE, Self::DEFAULT_COMPAT),
            table: None,
            no_sexism: Self::DEFAULT_NO_SEXISM,
            chunk_number: Self::DEFAULT_CHUNK_NUMBER,
            total_chunks: Self::DEFAULT_TOTAL_CHUNKS,
            partitioned: Self::DEFAULT_PARTITIONED,
            compat_mode: Self::DEFAULT_COMPAT,
            command_line_arguments: None,
        }
    }
}

impl Session {
    pub const DEFAULT_SCALE: f64 = 1.0;
    pub const DEFAULT_NO_SEXISM: bool = false;
    pub const DEFAULT_CHUNK_NUMBER: i32 = 1;
    pub const DEFAULT_TOTAL_CHUNKS: i32 = 1;
    pub const DEFAULT_PARTITIONED: bool = false;
    pub const DEFAULT_COMPAT: CompatMode = CompatMode::Trino;

    /// Convert this session into a builder initialized with its current values.
    pub fn into_builder(self) -> SessionBuilder {
        SessionBuilder {
            scale: self.scaling.get_scale(),
            table: self.table,
            no_sexism: self.no_sexism,
            chunk_number: self.chunk_number,
            total_chunks: self.total_chunks,
            partitioned: self.partitioned,
            compat_mode: self.compat_mode,
            command_line_arguments: self.command_line_arguments,
        }
    }

    /// Return the [`Scaling`] settings used for row counts.
    pub fn get_scaling(&self) -> &Scaling {
        &self.scaling
    }

    /// Return `true` if this session should generate a single table.
    pub fn generate_only_one_table(&self) -> bool {
        self.table.is_some()
    }

    /// Return the single table selected for generation.
    ///
    /// # Panics
    ///
    /// Panics if no single table was configured. Call
    /// [`Session::generate_only_one_table`] before using this method.
    pub fn get_only_table_to_generate(&self) -> Table {
        self.table
            .unwrap_or_else(|| panic!("table not present - call generate_only_one_table() first"))
    }

    /// Return the optional single-table selection.
    pub fn get_table(&self) -> Option<Table> {
        self.table
    }

    /// Return whether generated manager names should match the reference
    /// implementation's original gendered data.
    pub fn is_sexist(&self) -> bool {
        !self.no_sexism
    }

    /// Return the one-based chunk number represented by this session.
    pub fn get_chunk_number(&self) -> i32 {
        self.chunk_number
    }

    /// Return the total number of chunks for table generation
    pub fn get_total_chunks(&self) -> i32 {
        self.total_chunks
    }

    /// Return `true` if `--parts` was requested.
    ///
    /// Note this also returns true for `--parts 1`
    pub fn is_partitioned(&self) -> bool {
        self.partitioned
    }

    /// Return the 1-based, inclusive range of `table`'s source rows generated
    /// by this session.
    ///
    /// Note there is a 1M row minimum for TPCDS so smaller tables may have only
    /// a single chunk.
    ///
    /// A chunk with no work returns an empty range, so callers must check
    /// [`RangeInclusive::is_empty`] before generating
    pub fn get_source_row_range(&self, table: Table) -> RangeInclusive<u64> {
        let total_rows = self.scaling.get_row_count(table.source_table());
        let (first_row, row_count) = split_work(total_rows, self.chunk_number, self.total_chunks);
        first_row..=(first_row + row_count - 1)
    }

    /// Return the reference implementation compatibility mode.
    pub fn get_compat_mode(&self) -> CompatMode {
        self.compat_mode
    }

    /// Return the actual command line arguments used to create this session, if known.
    pub fn command_line_arguments(&self) -> Option<&str> {
        self.command_line_arguments.as_deref()
    }
}

/// Builder for validated [`Session`] construction.
#[derive(Debug, Clone)]
pub struct SessionBuilder {
    scale: f64,
    table: Option<Table>,
    no_sexism: bool,
    chunk_number: i32,
    total_chunks: i32,
    partitioned: bool,
    compat_mode: CompatMode,
    command_line_arguments: Option<String>,
}

impl Default for SessionBuilder {
    fn default() -> Self {
        Self {
            scale: Session::DEFAULT_SCALE,
            table: None,
            no_sexism: Session::DEFAULT_NO_SEXISM,
            chunk_number: Session::DEFAULT_CHUNK_NUMBER,
            total_chunks: Session::DEFAULT_TOTAL_CHUNKS,
            partitioned: Session::DEFAULT_PARTITIONED,
            compat_mode: Session::DEFAULT_COMPAT,
            command_line_arguments: None,
        }
    }
}

impl SessionBuilder {
    /// Create a builder initialized with the default session values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the scale factor to generate.
    pub fn with_scale_factor(mut self, scale: f64) -> Self {
        self.scale = scale;
        self
    }

    /// Restrict generation to a single table.
    pub fn with_table(mut self, table: Table) -> Self {
        self.table = Some(table);
        self
    }

    /// Clear any single-table restriction.
    pub fn without_table(mut self) -> Self {
        self.table = None;
        self
    }

    /// Set whether gender-neutral manager names are enabled.
    pub fn with_no_sexism(mut self, no_sexism: bool) -> Self {
        self.no_sexism = no_sexism;
        self
    }

    /// Set the one-based chunk number represented by this session.
    pub fn with_chunk_number(mut self, chunk_number: i32) -> Self {
        self.chunk_number = chunk_number;
        self
    }

    /// Set the total number of chunks for table generation.
    pub fn with_total_chunks(mut self, total_chunks: i32) -> Self {
        self.total_chunks = total_chunks;
        self
    }

    /// Set whether `--parts` was requested. See [`Session::is_partitioned`].
    pub fn with_partitioned(mut self, partitioned: bool) -> Self {
        self.partitioned = partitioned;
        self
    }

    /// Set the reference implementation compatibility mode.
    pub fn with_compat_mode(mut self, compat_mode: CompatMode) -> Self {
        self.compat_mode = compat_mode;
        self
    }

    /// Set the actual command line arguments used to create the session.
    pub fn with_command_line_arguments(
        mut self,
        command_line_arguments: impl Into<String>,
    ) -> Self {
        self.command_line_arguments = Some(command_line_arguments.into());
        self
    }

    /// Clear any command line arguments associated with the session.
    pub fn without_command_line_arguments(mut self) -> Self {
        self.command_line_arguments = None;
        self
    }

    /// Build a validated [`Session`].
    pub fn build(self) -> Result<Session> {
        self.validate()?;

        Ok(Session {
            scaling: Scaling::new_with_compat(self.scale, self.compat_mode),
            table: self.table,
            no_sexism: self.no_sexism,
            chunk_number: self.chunk_number,
            total_chunks: self.total_chunks,
            partitioned: self.partitioned,
            compat_mode: self.compat_mode,
            command_line_arguments: self.command_line_arguments,
        })
    }

    fn validate(&self) -> Result<()> {
        if !(0.0..=100000.0).contains(&self.scale) {
            return Err(InvalidOptionError::with_message(
                "scale",
                &self.scale.to_string(),
                "Scale must be between 0 and 100000, inclusive",
            )
            .into());
        }

        if self.chunk_number < 1 {
            return Err(InvalidOptionError::with_message(
                "chunk_number",
                &self.chunk_number.to_string(),
                "Chunk number must be >= 1",
            )
            .into());
        }

        if self.total_chunks < 1 {
            return Err(InvalidOptionError::with_message(
                "total_chunks",
                &self.total_chunks.to_string(),
                "Total chunks must be >= 1",
            )
            .into());
        }

        if self.chunk_number > self.total_chunks {
            return Err(InvalidOptionError::with_message(
                "chunk_number",
                &self.chunk_number.to_string(),
                &format!(
                    "Chunk number must be <= total_chunks ({})",
                    self.total_chunks
                ),
            )
            .into());
        }

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::reversed_empty_ranges)]
mod tests {
    use super::*;

    #[test]
    fn test_default_session() {
        let session = Session::default();

        assert_eq!(session.get_scaling().get_scale(), 1.0);
        assert!(!session.generate_only_one_table());
        assert!(session.is_sexist());
        assert_eq!(session.get_chunk_number(), 1);
        assert_eq!(session.command_line_arguments(), None);
    }

    #[test]
    fn test_session_builder() {
        let session = SessionBuilder::new()
            .with_scale_factor(2.0)
            .with_table(Table::CatalogSales)
            .with_no_sexism(true)
            .with_chunk_number(2)
            .with_total_chunks(4)
            .with_compat_mode(CompatMode::C)
            .with_command_line_arguments("tpcgen tpcds --scale-factor 2")
            .build()
            .unwrap();

        assert_eq!(session.get_scaling().get_scale(), 2.0);
        assert_eq!(session.get_table(), Some(Table::CatalogSales));
        assert!(!session.is_sexist());
        assert_eq!(session.get_chunk_number(), 2);
        assert_eq!(session.get_total_chunks(), 4);
        assert_eq!(session.get_compat_mode(), CompatMode::C);
        assert_eq!(
            session.command_line_arguments(),
            Some("tpcgen tpcds --scale-factor 2")
        );
    }

    #[test]
    fn test_session_builder_validation() {
        assert!(SessionBuilder::new()
            .with_scale_factor(10.0)
            .build()
            .is_ok());

        assert!(SessionBuilder::new()
            .with_scale_factor(-1.0)
            .build()
            .is_err());

        assert!(SessionBuilder::new()
            .with_scale_factor(f64::NAN)
            .build()
            .is_err());

        assert!(SessionBuilder::new().with_chunk_number(0).build().is_err());
    }

    #[test]
    fn test_into_builder() {
        let session = Session::default();

        let session = session
            .into_builder()
            .with_table(Table::CatalogSales)
            .with_scale_factor(10.0)
            .with_chunk_number(2)
            .with_total_chunks(4)
            .with_no_sexism(true)
            .with_command_line_arguments("initial")
            .without_command_line_arguments()
            .build()
            .unwrap();

        assert!(session.generate_only_one_table());
        assert_eq!(session.get_only_table_to_generate(), Table::CatalogSales);
        assert_eq!(session.get_scaling().get_scale(), 10.0);
        assert_eq!(session.get_chunk_number(), 2);
        assert!(!session.is_sexist());
        assert_eq!(session.command_line_arguments(), None);
    }

    #[test]
    fn test_generate_only_one_table() {
        let session = Session::default();
        assert!(!session.generate_only_one_table());

        let session_with_table = session
            .into_builder()
            .with_table(Table::StoreSales)
            .build()
            .unwrap();
        assert!(session_with_table.generate_only_one_table());
        assert_eq!(
            session_with_table.get_only_table_to_generate(),
            Table::StoreSales
        );
    }

    #[test]
    #[should_panic(expected = "table not present")]
    fn test_get_only_table_when_none() {
        let session = Session::default();
        session.get_only_table_to_generate();
    }

    #[test]
    fn test_call_center_ranges() {
        let test = RowSizeTest {
            table: Table::CallCenter,
            scale_factor: 1.0,
            total_chunks: 3,
            expected_ranges: &[1..=6, 1..=0, 1..=0],
        };

        test.run();
    }

    #[test]
    fn test_catalog_page_ranges() {
        let test = RowSizeTest {
            table: Table::CatalogPage,
            scale_factor: 1.0,
            total_chunks: 3,
            expected_ranges: &[1..=11718, 1..=0, 1..=0],
        };

        test.run();
    }

    #[test]
    fn test_catalog_returns_ranges() {
        // catalog_returns is split over its paired catalog_sales source rows
        let test = RowSizeTest {
            table: Table::CatalogReturns,
            scale_factor: 1.0,
            total_chunks: 3,
            expected_ranges: &[1..=160000, 1..=0, 1..=0],
        };

        test.run();
    }

    #[test]
    fn test_catalog_sales_ranges() {
        let test = RowSizeTest {
            table: Table::CatalogSales,
            scale_factor: 1.0,
            total_chunks: 3,
            expected_ranges: &[1..=160000, 1..=0, 1..=0],
        };

        test.run();
    }

    #[test]
    fn test_customer_ranges() {
        let test = RowSizeTest {
            table: Table::Customer,
            scale_factor: 1.0,
            total_chunks: 3,
            expected_ranges: &[1..=100000, 1..=0, 1..=0],
        };

        test.run();
    }

    #[test]
    fn test_customer_address_ranges() {
        let test = RowSizeTest {
            table: Table::CustomerAddress,
            scale_factor: 1.0,
            total_chunks: 3,
            expected_ranges: &[1..=50000, 1..=0, 1..=0],
        };

        test.run();
    }

    #[test]
    fn test_customer_demographics_ranges() {
        // 1,920,800 rows clears the 1M threshold, so it really is split. 3 does
        // not divide it evenly: the 2 extra rows go to the first 2 chunks.
        let test = RowSizeTest {
            table: Table::CustomerDemographics,
            scale_factor: 1.0,
            total_chunks: 3,
            expected_ranges: &[1..=640267, 640268..=1280534, 1280535..=1920800],
        };

        test.run();
    }

    #[test]
    fn test_date_dim_ranges() {
        let test = RowSizeTest {
            table: Table::DateDim,
            scale_factor: 1.0,
            total_chunks: 3,
            expected_ranges: &[1..=73049, 1..=0, 1..=0],
        };

        test.run();
    }

    #[test]
    fn test_household_demographics_ranges() {
        let test = RowSizeTest {
            table: Table::HouseholdDemographics,
            scale_factor: 1.0,
            total_chunks: 3,
            expected_ranges: &[1..=7200, 1..=0, 1..=0],
        };

        test.run();
    }

    #[test]
    fn test_income_band_ranges() {
        let test = RowSizeTest {
            table: Table::IncomeBand,
            scale_factor: 1.0,
            total_chunks: 3,
            expected_ranges: &[1..=20, 1..=0, 1..=0],
        };

        test.run();
    }

    #[test]
    fn test_inventory_ranges() {
        // 11,745,000 rows clears the 1M threshold and divides evenly by 3
        let test = RowSizeTest {
            table: Table::Inventory,
            scale_factor: 1.0,
            total_chunks: 3,
            expected_ranges: &[1..=3915000, 3915001..=7830000, 7830001..=11745000],
        };

        test.run();
    }

    #[test]
    fn test_item_ranges() {
        let test = RowSizeTest {
            table: Table::Item,
            scale_factor: 1.0,
            total_chunks: 3,
            expected_ranges: &[1..=18000, 1..=0, 1..=0],
        };

        test.run();
    }

    #[test]
    fn test_promotion_ranges() {
        let test = RowSizeTest {
            table: Table::Promotion,
            scale_factor: 1.0,
            total_chunks: 3,
            expected_ranges: &[1..=300, 1..=0, 1..=0],
        };

        test.run();
    }

    #[test]
    fn test_reason_ranges() {
        let test = RowSizeTest {
            table: Table::Reason,
            scale_factor: 1.0,
            total_chunks: 3,
            expected_ranges: &[1..=35, 1..=0, 1..=0],
        };

        test.run();
    }

    #[test]
    fn test_ship_mode_ranges() {
        let test = RowSizeTest {
            table: Table::ShipMode,
            scale_factor: 1.0,
            total_chunks: 3,
            expected_ranges: &[1..=20, 1..=0, 1..=0],
        };

        test.run();
    }

    #[test]
    fn test_store_ranges() {
        let test = RowSizeTest {
            table: Table::Store,
            scale_factor: 1.0,
            total_chunks: 3,
            expected_ranges: &[1..=12, 1..=0, 1..=0],
        };

        test.run();
    }

    #[test]
    fn test_store_returns_ranges() {
        // store_returns is split over its paired store_sales source rows
        let test = RowSizeTest {
            table: Table::StoreReturns,
            scale_factor: 1.0,
            total_chunks: 3,
            expected_ranges: &[1..=240000, 1..=0, 1..=0],
        };

        test.run();
    }

    #[test]
    fn test_store_sales_ranges() {
        let test = RowSizeTest {
            table: Table::StoreSales,
            scale_factor: 1.0,
            total_chunks: 3,
            expected_ranges: &[1..=240000, 1..=0, 1..=0],
        };

        test.run();
    }

    #[test]
    fn test_time_dim_ranges() {
        let test = RowSizeTest {
            table: Table::TimeDim,
            scale_factor: 1.0,
            total_chunks: 3,
            expected_ranges: &[1..=86400, 1..=0, 1..=0],
        };

        test.run();
    }

    #[test]
    fn test_warehouse_ranges() {
        let test = RowSizeTest {
            table: Table::Warehouse,
            scale_factor: 1.0,
            total_chunks: 3,
            expected_ranges: &[1..=5, 1..=0, 1..=0],
        };

        test.run();
    }

    #[test]
    fn test_web_page_ranges() {
        let test = RowSizeTest {
            table: Table::WebPage,
            scale_factor: 1.0,
            total_chunks: 3,
            expected_ranges: &[1..=60, 1..=0, 1..=0],
        };

        test.run();
    }

    #[test]
    fn test_web_returns_ranges() {
        // web_returns is split over its paired web_sales source rows
        let test = RowSizeTest {
            table: Table::WebReturns,
            scale_factor: 1.0,
            total_chunks: 3,
            expected_ranges: &[1..=60000, 1..=0, 1..=0],
        };

        test.run();
    }

    #[test]
    fn test_web_sales_ranges() {
        let test = RowSizeTest {
            table: Table::WebSales,
            scale_factor: 1.0,
            total_chunks: 3,
            expected_ranges: &[1..=60000, 1..=0, 1..=0],
        };

        test.run();
    }

    #[test]
    fn test_web_site_ranges() {
        let test = RowSizeTest {
            table: Table::WebSite,
            scale_factor: 1.0,
            total_chunks: 3,
            expected_ranges: &[1..=30, 1..=0, 1..=0],
        };

        test.run();
    }

    #[test]
    fn test_dbgen_version_ranges() {
        let test = RowSizeTest {
            table: Table::DbgenVersion,
            scale_factor: 1.0,
            total_chunks: 3,
            expected_ranges: &[1..=1, 1..=0, 1..=0],
        };

        test.run();
    }

    // Larger tables at scale factor 1000, split 10 ways. Both have hundreds of
    // millions of source rows, well over the 1M threshold, so every chunk gets
    // an equal share.

    #[test]
    fn test_store_sales_ranges_sf1000() {
        let test = RowSizeTest {
            table: Table::StoreSales,
            scale_factor: 1000.0,
            total_chunks: 10,
            // 240,000,000 source rows / 10 chunks = 24,000,000 rows each
            expected_ranges: &[
                1..=24000000,
                24000001..=48000000,
                48000001..=72000000,
                72000001..=96000000,
                96000001..=120000000,
                120000001..=144000000,
                144000001..=168000000,
                168000001..=192000000,
                192000001..=216000000,
                216000001..=240000000,
            ],
        };

        test.run();
    }

    #[test]
    fn test_catalog_sales_ranges_sf1000() {
        let test = RowSizeTest {
            table: Table::CatalogSales,
            scale_factor: 1000.0,
            total_chunks: 10,
            // 160,000,000 source rows / 10 chunks = 16,000,000 rows each
            expected_ranges: &[
                1..=16000000,
                16000001..=32000000,
                32000001..=48000000,
                48000001..=64000000,
                64000001..=80000000,
                80000001..=96000000,
                96000001..=112000000,
                112000001..=128000000,
                128000001..=144000000,
                144000001..=160000000,
            ],
        };

        test.run();
    }

    /// Verify that the row ranges for a table given scale factor and chunk configuration.
    struct RowSizeTest {
        table: Table,
        scale_factor: f64,
        total_chunks: i32,
        /// Expected ranges. Note that range is inclusive:
        /// * `1..=6` means rows 1 through 6
        /// * `1..=0` means no rows
        expected_ranges: &'static [RangeInclusive<u64>],
    }

    impl RowSizeTest {
        fn run(self) {
            let Self {
                table,
                scale_factor,
                total_chunks,
                expected_ranges,
            } = self;

            let actual_ranges: Vec<_> = (1..=total_chunks)
                .map(|chunk_number| {
                    SessionBuilder::new()
                        .with_scale_factor(scale_factor)
                        .with_chunk_number(chunk_number)
                        .with_total_chunks(total_chunks)
                        .build()
                        .unwrap()
                        .get_source_row_range(table)
                })
                .collect();

            assert_eq!(
                expected_ranges, &actual_ranges,
                "Row ranges for table {table:?} do not match expected"
            );
        }
    }
}
