use crate::config::Session;
use crate::distribution::DemographicsDistributions;
use crate::error::Result;
use crate::generator::IncomeBandGeneratorColumn;
use crate::random::RandomValueGenerator;
use crate::row::{AbstractRowGenerator, IncomeBandRow, RowGenerator, RowGeneratorResult};
use crate::table::Table;

/// Row generator for the INCOME_BAND table (IncomeBandRowGenerator)
pub struct IncomeBandRowGenerator {
    abstract_generator: AbstractRowGenerator,
}

impl Default for IncomeBandRowGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl IncomeBandRowGenerator {
    /// Create a new IncomeBandRowGenerator
    pub fn new() -> Self {
        Self {
            abstract_generator: AbstractRowGenerator::new(Table::IncomeBand),
        }
    }

    /// Generate an IncomeBandRow with realistic data following Java implementation
    fn generate_income_band_row(
        &mut self,
        row_number: u64,
        _session: &Session,
    ) -> Result<IncomeBandRow> {
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

        let ib_income_band_id = row_number as i32;
        let ib_lower_bound = DemographicsDistributions::get_income_band_lower_bound_at_index(
            (row_number - 1) as usize,
        )?;
        let ib_upper_bound = DemographicsDistributions::get_income_band_upper_bound_at_index(
            (row_number - 1) as usize,
        )?;

        Ok(IncomeBandRow::new(
            null_bit_map,
            ib_income_band_id,
            ib_lower_bound,
            ib_upper_bound,
        ))
    }
}

impl RowGenerator for IncomeBandRowGenerator {
    fn generate_row_and_child_rows(
        &mut self,
        row_number: u64,
        session: &Session,
        _parent_row_generator: Option<&mut dyn RowGenerator>,
        _child_row_generator: Option<&mut dyn RowGenerator>,
    ) -> Result<RowGeneratorResult> {
        let row = self.generate_income_band_row(row_number, session)?;
        Ok(RowGeneratorResult::new(row))
    }

    fn consume_remaining_seeds_for_row(&mut self) {
        self.abstract_generator.consume_remaining_seeds_for_row();
    }

    fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64) {
        self.abstract_generator
            .skip_rows_until_starting_row_number(starting_row_number);
    }
}
