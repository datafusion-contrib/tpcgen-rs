use crate::business_key_generator::make_business_key;
use crate::config::Session;
use crate::error::Result;
use crate::generator::WarehouseGeneratorColumn;
use crate::random::RandomValueGenerator;
use crate::row::{AbstractRowGenerator, RowGenerator, RowGeneratorResult, WarehouseRow};
use crate::table::Table;
use crate::types::Address;

/// Row generator for the WAREHOUSE table (WarehouseRowGenerator)
pub struct WarehouseRowGenerator {
    abstract_generator: AbstractRowGenerator,
}

impl Default for WarehouseRowGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl WarehouseRowGenerator {
    /// Create a new WarehouseRowGenerator
    pub fn new() -> Self {
        Self {
            abstract_generator: AbstractRowGenerator::new(Table::Warehouse),
        }
    }

    /// Generate a WarehouseRow with realistic data following Java implementation
    fn generate_warehouse_row(
        &mut self,
        row_number: u64,
        session: &Session,
    ) -> Result<WarehouseRow> {
        let row_number_i64 = i64::try_from(row_number).expect("row number fits in i64");

        // Create null bit map (createNullBitMap call)
        let nulls_stream = self
            .abstract_generator
            .get_random_number_stream(&WarehouseGeneratorColumn::WNulls);
        let threshold = RandomValueGenerator::generate_uniform_random_int(0, 9999, nulls_stream);
        let bit_map = RandomValueGenerator::generate_uniform_random_int(1, i32::MAX, nulls_stream);

        // Calculate null_bit_map based on threshold and table's not-null bitmap (Nulls.createNullBitMap)
        let null_bit_map = if threshold < Table::Warehouse.get_null_basis_points() {
            (bit_map as i64) & !Table::Warehouse.get_not_null_bit_map()
        } else {
            0
        };

        let w_warehouse_sk = row_number_i64;
        let w_warehouse_id = make_business_key(row_number);

        let name_stream = self
            .abstract_generator
            .get_random_number_stream(&WarehouseGeneratorColumn::WWarehouseName);
        let w_warehouse_name = RandomValueGenerator::generate_random_text(10, 20, name_stream);

        let sq_ft_stream = self
            .abstract_generator
            .get_random_number_stream(&WarehouseGeneratorColumn::WWarehouseSqFt);
        let w_warehouse_sq_ft =
            RandomValueGenerator::generate_uniform_random_int(50000, 1000000, sq_ft_stream);

        let scaling = session.get_scaling();
        let address_stream = self
            .abstract_generator
            .get_random_number_stream(&WarehouseGeneratorColumn::WWarehouseAddress);
        let w_address =
            Address::make_address_for_column(Table::Warehouse, address_stream, scaling)?;

        Ok(WarehouseRow::new(
            null_bit_map,
            w_warehouse_sk,
            w_warehouse_id,
            w_warehouse_name,
            w_warehouse_sq_ft,
            w_address,
        ))
    }
}

impl RowGenerator for WarehouseRowGenerator {
    fn generate_row_and_child_rows(
        &mut self,
        row_number: u64,
        session: &Session,
        _parent_row_generator: Option<&mut dyn RowGenerator>,
        _child_row_generator: Option<&mut dyn RowGenerator>,
    ) -> Result<RowGeneratorResult> {
        let row = self.generate_warehouse_row(row_number, session)?;
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
