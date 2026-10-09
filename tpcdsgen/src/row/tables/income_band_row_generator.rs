use crate::config::Session;
use crate::distribution::DemographicsDistributions;
use crate::generator::IncomeBandGeneratorColumn;
use crate::random::RandomValueGenerator;
use crate::row::{AbstractRowGenerator, IncomeBandRow};
use crate::table::Table;

/// Row generator for the INCOME_BAND table (IncomeBandRowGenerator)
pub struct IncomeBandRowGenerator {
    abstract_generator: AbstractRowGenerator,
}

impl IncomeBandRowGenerator {
    /// Generate source rows `1..=row_count`.
    pub fn new(_session: Session, row_count: u64) -> Self {
        Self {
            abstract_generator: AbstractRowGenerator::new(Table::IncomeBand, row_count),
        }
    }

    /// Generate an IncomeBandRow with realistic data following Java implementation
    fn generate_income_band_row(&mut self, row_number: u64) -> IncomeBandRow {
        // Create null bit map (createNullBitMap call)
        let nulls_stream = self
            .abstract_generator
            .get_random_number_stream(&IncomeBandGeneratorColumn::IbNulls);
        let threshold = RandomValueGenerator::generate_uniform_random_int(0, 9999, nulls_stream);
        let bit_map =
            RandomValueGenerator::generate_uniform_random_key(1, i32::MAX as i64, nulls_stream);

        // Calculate null_bit_map based on threshold and table's not-null bitmap (Nulls.createNullBitMap)
        let null_bit_map = if threshold < Table::IncomeBand.get_null_basis_points() {
            bit_map & !Table::IncomeBand.get_not_null_bit_map()
        } else {
            0
        };

        let ib_income_band_sk = row_number as i32;
        let ib_lower_bound = DemographicsDistributions::get_income_band_lower_bound_at_index(
            (row_number - 1) as usize,
        );
        let ib_upper_bound = DemographicsDistributions::get_income_band_upper_bound_at_index(
            (row_number - 1) as usize,
        );

        IncomeBandRow::new(
            null_bit_map,
            ib_income_band_sk,
            ib_lower_bound,
            ib_upper_bound,
        )
    }

    /// Start generating at `starting_row_number` (1-based), fast forwarding
    /// the random number streams to that row.
    pub fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64) {
        self.abstract_generator
            .skip_rows_until_starting_row_number(starting_row_number);
    }

    /// Restrict generation to source rows
    /// `starting_row_number..=ending_row_number` (1-based, inclusive).
    ///
    /// The ending row number is clamped to the table's row count.
    pub fn set_source_row_range(&mut self, starting_row_number: u64, ending_row_number: u64) {
        self.abstract_generator
            .set_source_row_range(starting_row_number, ending_row_number);
    }
}

impl Iterator for IncomeBandRowGenerator {
    type Item = IncomeBandRow;

    fn next(&mut self) -> Option<IncomeBandRow> {
        let row_number = self.abstract_generator.next_row_number()?;
        let row = self.generate_income_band_row(row_number);
        self.abstract_generator.finish_row();
        Some(row)
    }
}
