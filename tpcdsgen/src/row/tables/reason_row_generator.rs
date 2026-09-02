use crate::business_key_generator::make_business_key;
use crate::config::Session;
use crate::distribution::ReturnReasonsDistribution;
use crate::error::Result;
use crate::generator::ReasonGeneratorColumn;
use crate::random::RandomValueGenerator;
use crate::row::{AbstractRowGenerator, ReasonRow, RowGenerator, RowGeneratorResult};
use crate::table::Table;

/// Row generator for the REASON table (ReasonRowGenerator)
pub struct ReasonRowGenerator {
    abstract_generator: AbstractRowGenerator,
}

impl Default for ReasonRowGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ReasonRowGenerator {
    /// Create a new ReasonRowGenerator
    pub fn new() -> Self {
        Self {
            abstract_generator: AbstractRowGenerator::new(Table::Reason),
        }
    }

    /// Generate a ReasonRow with realistic data following Java implementation
    fn generate_reason_row(&mut self, row_number: u64, session: &Session) -> Result<ReasonRow> {
        let row_number_i64 = i64::try_from(row_number).expect("row number fits in i64");

        // Create null bit map (createNullBitMap call)
        let nulls_stream = self
            .abstract_generator
            .get_random_number_stream(&ReasonGeneratorColumn::RNulls);
        let threshold = RandomValueGenerator::generate_uniform_random_int(0, 9999, nulls_stream);
        let bit_map = RandomValueGenerator::generate_uniform_random_int(1, i32::MAX, nulls_stream);

        // Calculate null_bit_map based on threshold and table's not-null bitmap (Nulls.createNullBitMap)
        let null_bit_map = if threshold < Table::Reason.get_null_basis_points() {
            (bit_map as i64) & !Table::Reason.get_not_null_bit_map()
        } else {
            0
        };

        let r_reason_sk = row_number_i64;
        let r_reason_id = make_business_key(row_number);
        let r_reason_description = ReturnReasonsDistribution::get_return_reason_at_index(
            (row_number - 1) as usize,
            session.get_compat_mode(),
        )?;

        Ok(ReasonRow::new(
            null_bit_map,
            r_reason_sk,
            r_reason_id.to_string(),
            r_reason_description.to_string(),
        ))
    }
}

impl RowGenerator for ReasonRowGenerator {
    fn generate_row_and_child_rows(
        &mut self,
        row_number: u64,
        session: &Session,
        _parent_row_generator: Option<&mut dyn RowGenerator>,
        _child_row_generator: Option<&mut dyn RowGenerator>,
    ) -> Result<RowGeneratorResult> {
        let row = self.generate_reason_row(row_number, session)?;
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
