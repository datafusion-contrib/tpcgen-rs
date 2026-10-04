use crate::business_key_generator::make_business_key;
use crate::config::Session;
use crate::distribution::ReturnReasonsDistribution;
use crate::error::Result;
use crate::generator::ReasonGeneratorColumn;
use crate::random::RandomValueGenerator;
use crate::row::{AbstractRowGenerator, ReasonRow};
use crate::table::Table;

/// Row generator for the REASON table (ReasonRowGenerator)
pub struct ReasonRowGenerator {
    abstract_generator: AbstractRowGenerator,
    session: Session,
}

impl ReasonRowGenerator {
    /// Generate source rows `1..=row_count`.
    pub fn new(session: Session, row_count: u64) -> Self {
        Self {
            abstract_generator: AbstractRowGenerator::new(Table::Reason, row_count),
            session,
        }
    }

    /// Generate a ReasonRow with realistic data following Java implementation
    fn generate_reason_row(&mut self, row_number: u64) -> Result<ReasonRow> {
        let session = &self.session;
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
        let r_reason_desc = ReturnReasonsDistribution::get_return_reason_at_index(
            (row_number - 1) as usize,
            session.get_compat_mode(),
        )?;

        Ok(ReasonRow::new(
            null_bit_map,
            r_reason_sk,
            r_reason_id.to_string(),
            r_reason_desc.to_string(),
        ))
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

impl Iterator for ReasonRowGenerator {
    type Item = ReasonRow;

    fn next(&mut self) -> Option<ReasonRow> {
        let row_number = self.abstract_generator.next_row_number()?;
        let row = self.generate_reason_row(row_number).expect("row gen");
        self.abstract_generator.finish_row();
        Some(row)
    }
}
