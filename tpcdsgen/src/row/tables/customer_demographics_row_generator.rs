use crate::config::Session;
use crate::distribution::DemographicsDistributions;
use crate::generator::CustomerDemographicsGeneratorColumn;
use crate::random::RandomValueGenerator;
use crate::row::{AbstractRowGenerator, CustomerDemographicsRow};
use crate::table::Table;

/// Row generator for the CUSTOMER_DEMOGRAPHICS table (CustomerDemographicsRowGenerator)
pub struct CustomerDemographicsRowGenerator {
    abstract_generator: AbstractRowGenerator,
    current_row: u64,
    row_count: u64,
}

impl CustomerDemographicsRowGenerator {
    // Constants matching Java implementation
    const MAX_CHILDREN: i64 = 7;
    const MAX_EMPLOYED: i64 = 7;
    const MAX_COLLEGE: i64 = 7;

    /// Generate source rows `1..=row_count`.
    pub fn new(_session: Session, row_count: u64) -> Self {
        Self {
            abstract_generator: AbstractRowGenerator::new(Table::CustomerDemographics),
            current_row: 1,
            row_count,
        }
    }

    /// Generate a CustomerDemographicsRow with realistic data following Java implementation
    fn generate_customer_demographics_row(&mut self, row_number: u64) -> CustomerDemographicsRow {
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

        CustomerDemographicsRow::new(
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
        )
    }

    /// Start generating at `starting_row_number` (1-based), fast forwarding
    /// the random number streams to that row.
    pub fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64) {
        self.abstract_generator
            .skip_rows_until_starting_row_number(starting_row_number);
        self.current_row = starting_row_number;
    }

    /// Restrict generation to source rows
    /// `starting_row_number..=ending_row_number` (1-based, inclusive).
    ///
    /// The ending row number is clamped to the table's row count.
    pub fn set_source_row_range(&mut self, starting_row_number: u64, ending_row_number: u64) {
        self.skip_rows_until_starting_row_number(starting_row_number);
        self.row_count = self.row_count.min(ending_row_number);
    }
}

impl Iterator for CustomerDemographicsRowGenerator {
    type Item = CustomerDemographicsRow;

    fn next(&mut self) -> Option<CustomerDemographicsRow> {
        if self.current_row > self.row_count {
            return None;
        }
        let row = self.generate_customer_demographics_row(self.current_row);
        self.abstract_generator.consume_remaining_seeds_for_row();
        self.current_row += 1;
        Some(row)
    }
}
