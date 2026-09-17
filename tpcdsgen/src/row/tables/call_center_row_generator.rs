use crate::config::Session;
use crate::distribution::{CallCenterDistributions, FirstNamesWeights, NamesDistributions};
use crate::error::Result;
use crate::generator::CallCenterGeneratorColumn;
use crate::random::RandomValueGenerator;
use crate::row::{AbstractRowGenerator, CallCenterRow, RowGenerator, RowGeneratorResult};
use crate::slowly_changing_dimension_utils::{
    compute_scd_key, get_value_for_slowly_changing_dimension, SlowlyChangingDimensionKey,
};
use crate::table::Table;
use crate::types::{Address, Date, Decimal};

/// Row generator for the CALL_CENTER table (CallCenterRowGenerator)
pub struct CallCenterRowGenerator {
    abstract_generator: AbstractRowGenerator,
    previous_row: Option<CallCenterRow>,
}

// Constants matching Java implementation
// We'll define these as functions since const Decimal isn't available
fn min_tax_percentage() -> Decimal {
    Decimal::new(0, 2).unwrap()
}

fn max_tax_percentage() -> Decimal {
    Decimal::new(12, 2).unwrap()
}
const WIDTH_CC_DIVISION_NAME: i32 = 50;
const WIDTH_CC_MARKET_CLASS: i32 = 50;
const WIDTH_CC_MARKET_DESC: i32 = 100;
const MAX_NUMBER_OF_EMPLOYEES_UNSCALED: i32 = 7;
const JULIAN_DATE_START: i64 = Date::JULIAN_DATA_START_DATE - 23; // 23 is the ordinal of CALL_CENTER table

impl Default for CallCenterRowGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CallCenterRowGenerator {
    /// Create a new CallCenterRowGenerator
    pub fn new() -> Self {
        Self {
            abstract_generator: AbstractRowGenerator::new(Table::CallCenter),
            previous_row: None,
        }
    }

    /// Generate a CallCenterRow with realistic data following Java implementation
    fn generate_call_center_row(
        &mut self,
        row_number: u64,
        session: &Session,
    ) -> Result<CallCenterRow> {
        let row_number_i64 = i64::try_from(row_number).expect("row number fits in i64");

        // Create null bit map (createNullBitMap call)
        let nulls_stream = self
            .abstract_generator
            .get_random_number_stream(&CallCenterGeneratorColumn::CcNulls);
        let _threshold = RandomValueGenerator::generate_uniform_random_int(0, 9999, nulls_stream);
        let _bit_map = RandomValueGenerator::generate_uniform_random_int(1, i32::MAX, nulls_stream);

        // The id combined with start and end dates represent the unique key for this row.
        // The id is what would be a primary key if there were only one version of each row
        // the start and end dates are the version information for the row.
        let scd_key: SlowlyChangingDimensionKey = compute_scd_key(Table::CallCenter, row_number);

        let scaling = session.get_scaling();
        let is_new_business_key = scd_key.is_new_business_key();

        // These fields only change when there is a new id. They remain constant across different version of a row.
        let (cc_open_date_id, cc_name, cc_address) = if is_new_business_key {
            let open_date_stream = self
                .abstract_generator
                .get_random_number_stream(&CallCenterGeneratorColumn::CcOpenDateId);
            let open_date_random =
                RandomValueGenerator::generate_uniform_random_int(-365, 0, open_date_stream);
            let open_date_id = JULIAN_DATE_START - open_date_random as i64;

            let number_of_call_centers =
                CallCenterDistributions::get_number_of_call_centers().unwrap_or(12);
            let suffix = (row_number_i64 / number_of_call_centers as i64) as i32;
            let cc_name = CallCenterDistributions::get_call_center_at_index(
                (row_number_i64 % number_of_call_centers as i64) as usize,
            )
            .unwrap_or("Unknown");

            let final_cc_name = if suffix > 0 {
                format!("{}_{}", cc_name, suffix)
            } else {
                cc_name.to_string()
            };

            // Generate address
            let address_stream = self
                .abstract_generator
                .get_random_number_stream(&CallCenterGeneratorColumn::CcAddress);
            let address =
                Address::make_address_for_column(Table::CallCenter, address_stream, scaling)?;

            (open_date_id, final_cc_name, address)
        } else {
            // Use values from previous row - DO NOT consume random streams!
            if let Some(ref prev_row) = self.previous_row {
                (
                    prev_row.get_cc_open_date_id(),
                    prev_row.get_cc_name().to_string(),
                    prev_row.get_cc_address().clone(),
                )
            } else {
                return Err(crate::TpcdsError::new(
                    "previousRow has not yet been initialized",
                ));
            }
        };

        // Select the random number that controls if a field changes from one record to the next.
        let scd_stream = self
            .abstract_generator
            .get_random_number_stream(&CallCenterGeneratorColumn::CcScd);
        let mut field_change_flag = scd_stream.next_random() as i32;

        // The rest of the fields can either be a new data value or not.
        // We use a random number to determine which fields to replace and which to retain.
        // A field changes if is_new_business_key is true or the lowest order bit of the random number is zero.
        // Then we divide the field_change_flag by 2 so the next field is determined by the next lower bit.

        // There is a bug in the C code for adjusting pointer types (which this is) with slowly changing dimensions,
        // so it always uses the new value
        let class_stream = self
            .abstract_generator
            .get_random_number_stream(&CallCenterGeneratorColumn::CcClass);
        let cc_class =
            CallCenterDistributions::pick_random_call_center_class(class_stream).unwrap_or("large");
        field_change_flag >>= 1;

        let employees_stream = self
            .abstract_generator
            .get_random_number_stream(&CallCenterGeneratorColumn::CcEmployees);
        let mut cc_employees = RandomValueGenerator::generate_uniform_random_int(
            1,
            MAX_NUMBER_OF_EMPLOYEES_UNSCALED
                * scaling.get_scale().ceil() as i32
                * scaling.get_scale().ceil() as i32,
            employees_stream,
        );
        if let Some(ref prev_row) = self.previous_row {
            cc_employees = get_value_for_slowly_changing_dimension(
                field_change_flag,
                is_new_business_key,
                prev_row.get_cc_employees(),
                cc_employees,
            );
        }
        field_change_flag >>= 1;

        let sq_ft_stream = self
            .abstract_generator
            .get_random_number_stream(&CallCenterGeneratorColumn::CcSqFt);
        let mut cc_sq_ft =
            RandomValueGenerator::generate_uniform_random_int(100, 700, sq_ft_stream)
                * cc_employees;
        if let Some(ref prev_row) = self.previous_row {
            cc_sq_ft = get_value_for_slowly_changing_dimension(
                field_change_flag,
                is_new_business_key,
                prev_row.get_cc_sq_ft(),
                cc_sq_ft,
            );
        }
        field_change_flag >>= 1;

        // Another casualty of the bug with pointer types in the C code. Will always use a new value.
        let hours_stream = self
            .abstract_generator
            .get_random_number_stream(&CallCenterGeneratorColumn::CcHours);
        let cc_hours = CallCenterDistributions::pick_random_call_center_hours(hours_stream)
            .unwrap_or("8AM-8PM");
        field_change_flag >>= 1;

        let manager_stream = self
            .abstract_generator
            .get_random_number_stream(&CallCenterGeneratorColumn::CcManager);
        let manager_first_name = NamesDistributions::pick_random_first_name(
            if session.is_sexist() {
                FirstNamesWeights::MaleFrequency
            } else {
                FirstNamesWeights::GeneralFrequency
            },
            manager_stream,
        )
        .unwrap_or("John");
        let manager_last_name =
            NamesDistributions::pick_random_last_name(manager_stream).unwrap_or("Smith");
        let mut cc_manager = format!("{} {}", manager_first_name, manager_last_name);
        if let Some(ref prev_row) = self.previous_row {
            cc_manager = get_value_for_slowly_changing_dimension(
                field_change_flag,
                is_new_business_key,
                prev_row.get_cc_manager().to_string(),
                cc_manager,
            );
        }
        field_change_flag >>= 1;

        let market_id_stream = self
            .abstract_generator
            .get_random_number_stream(&CallCenterGeneratorColumn::CcMarketId);
        let mut cc_market_id =
            RandomValueGenerator::generate_uniform_random_int(1, 6, market_id_stream);
        if let Some(ref prev_row) = self.previous_row {
            cc_market_id = get_value_for_slowly_changing_dimension(
                field_change_flag,
                is_new_business_key,
                prev_row.get_cc_market_id(),
                cc_market_id,
            );
        }
        field_change_flag >>= 1;

        let market_class_stream = self
            .abstract_generator
            .get_random_number_stream(&CallCenterGeneratorColumn::CcMarketClass);
        let mut cc_market_class = RandomValueGenerator::generate_random_text(
            20,
            WIDTH_CC_MARKET_CLASS,
            market_class_stream,
        );
        if let Some(ref prev_row) = self.previous_row {
            cc_market_class = get_value_for_slowly_changing_dimension(
                field_change_flag,
                is_new_business_key,
                prev_row.get_cc_market_class().to_string(),
                cc_market_class,
            );
        }
        field_change_flag >>= 1;

        let market_desc_stream = self
            .abstract_generator
            .get_random_number_stream(&CallCenterGeneratorColumn::CcMarketDesc);
        let mut cc_market_desc = RandomValueGenerator::generate_random_text(
            20,
            WIDTH_CC_MARKET_DESC,
            market_desc_stream,
        );
        if let Some(ref prev_row) = self.previous_row {
            cc_market_desc = get_value_for_slowly_changing_dimension(
                field_change_flag,
                is_new_business_key,
                prev_row.get_cc_market_desc().to_string(),
                cc_market_desc,
            );
        }
        field_change_flag >>= 1;

        let market_manager_stream = self
            .abstract_generator
            .get_random_number_stream(&CallCenterGeneratorColumn::CcMarketManager);
        let market_manager_first_name = NamesDistributions::pick_random_first_name(
            if session.is_sexist() {
                FirstNamesWeights::MaleFrequency
            } else {
                FirstNamesWeights::GeneralFrequency
            },
            market_manager_stream,
        )
        .unwrap_or("Jane");
        let market_manager_last_name =
            NamesDistributions::pick_random_last_name(market_manager_stream).unwrap_or("Doe");
        let mut cc_market_manager =
            format!("{} {}", market_manager_first_name, market_manager_last_name);
        if let Some(ref prev_row) = self.previous_row {
            cc_market_manager = get_value_for_slowly_changing_dimension(
                field_change_flag,
                is_new_business_key,
                prev_row.get_cc_market_manager().to_string(),
                cc_market_manager,
            );
        }
        field_change_flag >>= 1;

        let company_stream = self
            .abstract_generator
            .get_random_number_stream(&CallCenterGeneratorColumn::CcCompany);
        let mut cc_company =
            RandomValueGenerator::generate_uniform_random_int(1, 6, company_stream);
        if let Some(ref prev_row) = self.previous_row {
            cc_company = get_value_for_slowly_changing_dimension(
                field_change_flag,
                is_new_business_key,
                prev_row.get_cc_company(),
                cc_company,
            );
        }
        field_change_flag >>= 1;

        let division_id_stream = self
            .abstract_generator
            .get_random_number_stream(&CallCenterGeneratorColumn::CcCompany); // Note: uses same stream as company
        let mut cc_division_id =
            RandomValueGenerator::generate_uniform_random_int(1, 6, division_id_stream);
        if let Some(ref prev_row) = self.previous_row {
            cc_division_id = get_value_for_slowly_changing_dimension(
                field_change_flag,
                is_new_business_key,
                prev_row.get_cc_division_id(),
                cc_division_id,
            );
        }
        field_change_flag >>= 1;

        // Generate word doesn't use random numbers - it deterministically creates from seed
        // We still need to consume the stream to maintain RNG state
        let _division_name_stream = self
            .abstract_generator
            .get_random_number_stream(&CallCenterGeneratorColumn::CcDivisionName);
        let mut cc_division_name = RandomValueGenerator::generate_word(
            cc_division_id as i64,
            WIDTH_CC_DIVISION_NAME,
            crate::distribution::get_syllables_distribution(),
        );
        if let Some(ref prev_row) = self.previous_row {
            cc_division_name = get_value_for_slowly_changing_dimension(
                field_change_flag,
                is_new_business_key,
                prev_row.get_cc_division_name().to_string(),
                cc_division_name,
            );
        }
        field_change_flag >>= 1;

        let _company_name_stream = self
            .abstract_generator
            .get_random_number_stream(&CallCenterGeneratorColumn::CcCompanyName);
        let mut cc_company_name = RandomValueGenerator::generate_word(
            cc_company as i64,
            10,
            crate::distribution::get_syllables_distribution(),
        );
        if let Some(ref prev_row) = self.previous_row {
            cc_company_name = get_value_for_slowly_changing_dimension(
                field_change_flag,
                is_new_business_key,
                prev_row.get_cc_company_name().to_string(),
                cc_company_name,
            );
        }
        field_change_flag >>= 1;

        let tax_percentage_stream = self
            .abstract_generator
            .get_random_number_stream(&CallCenterGeneratorColumn::CcTaxPercentage);
        let mut cc_tax_percentage = RandomValueGenerator::generate_uniform_random_decimal(
            min_tax_percentage(),
            max_tax_percentage(),
            tax_percentage_stream,
        );
        if let Some(ref prev_row) = self.previous_row {
            cc_tax_percentage = get_value_for_slowly_changing_dimension(
                field_change_flag,
                is_new_business_key,
                *prev_row.get_cc_tax_percentage(),
                cc_tax_percentage,
            );
        }

        // Build the row in one go
        let new_row = CallCenterRow::builder()
            .set_null_bit_map(0)
            .set_cc_call_center_sk(row_number_i64)
            .set_cc_call_center_id(scd_key.get_business_key().to_string())
            .set_cc_rec_start_date_id(scd_key.get_start_date())
            .set_cc_rec_end_date_id(scd_key.get_end_date())
            .set_cc_closed_date_id(-1)
            .set_cc_open_date_id(cc_open_date_id)
            .set_cc_name(cc_name)
            .set_cc_class(cc_class.to_string())
            .set_cc_employees(cc_employees)
            .set_cc_sq_ft(cc_sq_ft)
            .set_cc_hours(cc_hours.to_string())
            .set_cc_manager(cc_manager)
            .set_cc_market_id(cc_market_id)
            .set_cc_market_class(cc_market_class)
            .set_cc_market_desc(cc_market_desc)
            .set_cc_market_manager(cc_market_manager)
            .set_cc_division_id(cc_division_id)
            .set_cc_division_name(cc_division_name)
            .set_cc_company(cc_company)
            .set_cc_company_name(cc_company_name)
            .set_cc_address(cc_address)
            .set_cc_tax_percentage(cc_tax_percentage)
            .build();

        self.previous_row = Some(new_row.clone());

        Ok(new_row)
    }
}

impl RowGenerator for CallCenterRowGenerator {
    fn generate_row_and_child_rows(
        &mut self,
        row_number: u64,
        session: &Session,
        _parent_row_generator: Option<&mut dyn RowGenerator>,
        _child_row_generator: Option<&mut dyn RowGenerator>,
    ) -> Result<RowGeneratorResult> {
        let row = self.generate_call_center_row(row_number, session)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Session;
    use crate::row::dat_values;

    #[test]
    fn test_call_center_row_generator_creation() {
        let generator = CallCenterRowGenerator::new();
        assert_eq!(generator.abstract_generator.get_table(), Table::CallCenter);
    }

    #[test]
    fn test_generate_call_center_row() {
        let mut generator = CallCenterRowGenerator::new();
        let session = Session::default();

        let result = generator
            .generate_row_and_child_rows(1, &session, None, None)
            .unwrap();
        let rows = result.get_rows();

        assert_eq!(rows.len(), 1);
        assert!(result.should_end_row());

        // Check that we can get values (CSV serialization works)
        let values = dat_values(&rows[0]);
        assert_eq!(values[0], "1"); // cc_call_center_sk should be row number
    }
}
