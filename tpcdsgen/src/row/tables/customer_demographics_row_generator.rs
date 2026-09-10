use crate::config::Session;
use crate::distribution::DemographicsDistributions;
use crate::error::Result;
use crate::generator::CustomerDemographicsGeneratorColumn;
use crate::random::RandomValueGenerator;
use crate::row::{AbstractRowGenerator, CustomerDemographicsRow, RowGenerator, RowGeneratorResult};
use crate::table::Table;

/// Row generator for the CUSTOMER_DEMOGRAPHICS table (CustomerDemographicsRowGenerator)
pub struct CustomerDemographicsRowGenerator {
    abstract_generator: AbstractRowGenerator,
}

impl Default for CustomerDemographicsRowGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomerDemographicsRowGenerator {
    // Constants matching Java implementation
    const MAX_CHILDREN: i64 = 7;
    const MAX_EMPLOYED: i64 = 7;
    const MAX_COLLEGE: i64 = 7;

    /// Create a new CustomerDemographicsRowGenerator
    pub fn new() -> Self {
        Self {
            abstract_generator: AbstractRowGenerator::new(Table::CustomerDemographics),
        }
    }

    /// Generate a CustomerDemographicsRow with realistic data following Java implementation
    fn generate_customer_demographics_row(
        &mut self,
        row_number: u64,
        _session: &Session,
    ) -> Result<CustomerDemographicsRow> {
        let row_number_i64 = i64::try_from(row_number).expect("row number fits in i64");

        // Create null bit map (createNullBitMap call)
        let nulls_stream = self
            .abstract_generator
            .get_random_number_stream(&CustomerDemographicsGeneratorColumn::CdNulls);
        let threshold = RandomValueGenerator::generate_uniform_random_int(0, 9999, nulls_stream);
        let bit_map =
            RandomValueGenerator::generate_uniform_random_key(1, i32::MAX as i64, nulls_stream);

        // Calculate null_bit_map based on threshold and table's not-null bitmap (Nulls.createNullBitMap)
        let null_bit_map = if threshold < Table::CustomerDemographics.get_null_basis_points() {
            bit_map & !Table::CustomerDemographics.get_not_null_bit_map()
        } else {
            0
        };

        // Generate demographics using index-based cartesian product (algorithm)
        let cd_demo_sk = row_number_i64;
        let mut index = cd_demo_sk - 1;

        // Get gender and divide index
        let cd_gender = DemographicsDistributions::get_gender_for_index_mod_size(index);
        index /= DemographicsDistributions::get_gender_size() as i64;

        // Get marital status and divide index
        let cd_marital_status =
            DemographicsDistributions::get_marital_status_for_index_mod_size(index);
        index /= DemographicsDistributions::get_marital_status_size() as i64;

        // Get education and divide index
        let cd_education_status =
            DemographicsDistributions::get_education_for_index_mod_size(index);
        index /= DemographicsDistributions::get_education_size() as i64;

        // Get purchase band and divide index
        let cd_purchase_estimate =
            DemographicsDistributions::get_purchase_band_for_index_mod_size(index);
        index /= DemographicsDistributions::get_purchase_band_size() as i64;

        // Get credit rating and divide index
        let cd_credit_rating =
            DemographicsDistributions::get_credit_rating_for_index_mod_size(index);
        index /= DemographicsDistributions::get_credit_rating_size() as i64;

        // Get dependent counts using modulo (no division lookup needed)
        let cd_dep_count = (index % Self::MAX_CHILDREN) as i32;
        index /= Self::MAX_CHILDREN;

        let cd_dep_employed_count = (index % Self::MAX_EMPLOYED) as i32;
        index /= Self::MAX_EMPLOYED;

        let cd_dep_college_count = (index % Self::MAX_COLLEGE) as i32;

        Ok(CustomerDemographicsRow::new(
            null_bit_map,
            cd_demo_sk,
            cd_gender.to_string(),
            cd_marital_status.to_string(),
            cd_education_status.to_string(),
            cd_purchase_estimate,
            cd_credit_rating.to_string(),
            cd_dep_count,
            cd_dep_employed_count,
            cd_dep_college_count,
        ))
    }
}

impl RowGenerator for CustomerDemographicsRowGenerator {
    fn generate_row_and_child_rows(
        &mut self,
        row_number: u64,
        session: &Session,
        _parent_row_generator: Option<&mut dyn RowGenerator>,
        _child_row_generator: Option<&mut dyn RowGenerator>,
    ) -> Result<RowGeneratorResult> {
        let row = self.generate_customer_demographics_row(row_number, session)?;
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
