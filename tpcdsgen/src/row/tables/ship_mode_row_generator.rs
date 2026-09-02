use crate::business_key_generator::make_business_key;
use crate::config::Session;
use crate::distribution::ShipModeDistributions;
use crate::error::Result;
use crate::generator::ShipModeGeneratorColumn;
use crate::random::RandomValueGenerator;
use crate::row::{AbstractRowGenerator, RowGenerator, RowGeneratorResult, ShipModeRow};
use crate::table::Table;

/// Row generator for the SHIP_MODE table (ShipModeRowGenerator)
pub struct ShipModeRowGenerator {
    abstract_generator: AbstractRowGenerator,
}

impl Default for ShipModeRowGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ShipModeRowGenerator {
    /// Create a new ShipModeRowGenerator
    pub fn new() -> Self {
        Self {
            abstract_generator: AbstractRowGenerator::new(Table::ShipMode),
        }
    }

    /// Generate a ShipModeRow with realistic data following Java implementation
    fn generate_ship_mode_row(
        &mut self,
        row_number: u64,
        _session: &Session,
    ) -> Result<ShipModeRow> {
        let row_number_i64 = i64::try_from(row_number).expect("row number fits in i64");

        // Create null bit map (createNullBitMap call)
        let nulls_stream = self
            .abstract_generator
            .get_random_number_stream(&ShipModeGeneratorColumn::SmNulls);
        let threshold = RandomValueGenerator::generate_uniform_random_int(0, 9999, nulls_stream);
        let bit_map = RandomValueGenerator::generate_uniform_random_int(1, i32::MAX, nulls_stream);

        // Calculate null_bit_map based on threshold and table's not-null bitmap (Nulls.createNullBitMap)
        let null_bit_map = if threshold < Table::ShipMode.get_null_basis_points() {
            (bit_map as i64) & !Table::ShipMode.get_not_null_bit_map()
        } else {
            0
        };

        let sm_ship_mode_sk = row_number_i64;
        let sm_ship_mode_id = make_business_key(row_number);

        let sm_type = ShipModeDistributions::get_ship_mode_type_for_index_mod_size(row_number_i64)?;

        // Calculate index for code (divide by type distribution size)
        let type_distribution_size = ShipModeDistributions::get_ship_mode_type_size() as i64;
        let index = row_number_i64 / type_distribution_size;

        let sm_code = ShipModeDistributions::get_ship_mode_code_for_index_mod_size(index)?;

        let sm_carrier =
            ShipModeDistributions::get_ship_mode_carrier_at_index((row_number - 1) as usize)?;

        let contract_stream = self
            .abstract_generator
            .get_random_number_stream(&ShipModeGeneratorColumn::SmContract);
        let sm_contract = RandomValueGenerator::generate_random_charset(
            RandomValueGenerator::ALPHA_NUMERIC,
            1,
            20,
            contract_stream,
        );

        Ok(ShipModeRow::new(
            null_bit_map,
            sm_ship_mode_sk,
            sm_ship_mode_id.to_string(),
            sm_type.to_string(),
            sm_code.to_string(),
            sm_carrier.to_string(),
            sm_contract,
        ))
    }
}

impl RowGenerator for ShipModeRowGenerator {
    fn generate_row_and_child_rows(
        &mut self,
        row_number: u64,
        session: &Session,
        _parent_row_generator: Option<&mut dyn RowGenerator>,
        _child_row_generator: Option<&mut dyn RowGenerator>,
    ) -> Result<RowGeneratorResult> {
        let row = self.generate_ship_mode_row(row_number, session)?;
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
