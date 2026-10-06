use crate::distribution::string_values_distribution::StringValuesDistribution as FileBasedStringValuesDistribution;
use crate::distribution::{Distribution, IntValuesDistribution};
use std::sync::OnceLock;

/// Distribution for demographics data including income bands (Demographics)
pub struct DemographicsDistributions;

impl DemographicsDistributions {
    /// Lazy-loaded distribution instance for genders.dst (GENDER_DISTRIBUTION)
    fn get_gender_distribution() -> &'static FileBasedStringValuesDistribution {
        static DISTRIBUTION: OnceLock<FileBasedStringValuesDistribution> = OnceLock::new();
        DISTRIBUTION.get_or_init(|| {
            FileBasedStringValuesDistribution::build_string_values_distribution("genders.dst", 1, 1)
                .expect("Failed to load genders.dst")
        })
    }

    /// Lazy-loaded distribution instance for marital_statuses.dst (MARITAL_STATUS_DISTRIBUTION)
    fn get_marital_status_distribution() -> &'static FileBasedStringValuesDistribution {
        static DISTRIBUTION: OnceLock<FileBasedStringValuesDistribution> = OnceLock::new();
        DISTRIBUTION.get_or_init(|| {
            FileBasedStringValuesDistribution::build_string_values_distribution(
                "marital_statuses.dst",
                1,
                1,
            )
            .expect("Failed to load marital_statuses.dst")
        })
    }

    /// Lazy-loaded distribution instance for education.dst (EDUCATION_DISTRIBUTION)
    fn get_education_distribution() -> &'static FileBasedStringValuesDistribution {
        static DISTRIBUTION: OnceLock<FileBasedStringValuesDistribution> = OnceLock::new();
        DISTRIBUTION.get_or_init(|| {
            FileBasedStringValuesDistribution::build_string_values_distribution(
                "education.dst",
                1,
                4,
            )
            .expect("Failed to load education.dst")
        })
    }

    /// Lazy-loaded distribution instance for purchase_band.dst (PURCHASE_BAND_DISTRIBUTION)
    fn get_purchase_band_distribution() -> &'static IntValuesDistribution {
        static DISTRIBUTION: OnceLock<IntValuesDistribution> = OnceLock::new();
        DISTRIBUTION.get_or_init(|| {
            IntValuesDistribution::build_int_values_distribution("purchase_band.dst", 1, 1)
                .expect("Failed to load purchase_band.dst")
        })
    }

    /// Lazy-loaded distribution instance for credit_ratings.dst (CREDIT_RATING_DISTRIBUTION)
    fn get_credit_rating_distribution() -> &'static FileBasedStringValuesDistribution {
        static DISTRIBUTION: OnceLock<FileBasedStringValuesDistribution> = OnceLock::new();
        DISTRIBUTION.get_or_init(|| {
            FileBasedStringValuesDistribution::build_string_values_distribution(
                "credit_ratings.dst",
                1,
                1,
            )
            .expect("Failed to load credit_ratings.dst")
        })
    }

    /// Lazy-loaded distribution instance for income_band.dst
    /// Contains 2 value fields (lower_bound, upper_bound) and 1 weight field
    fn get_income_band_distribution() -> &'static IntValuesDistribution {
        static DISTRIBUTION: OnceLock<IntValuesDistribution> = OnceLock::new();
        DISTRIBUTION.get_or_init(|| {
            IntValuesDistribution::build_int_values_distribution("income_band.dst", 2, 1)
                .expect("Failed to load income_band.dst")
        })
    }

    /// Lazy-loaded distribution instance for buy_potential.dst (BUY_POTENTIAL_DISTRIBUTION)
    fn get_buy_potential_distribution() -> &'static FileBasedStringValuesDistribution {
        static DISTRIBUTION: OnceLock<FileBasedStringValuesDistribution> = OnceLock::new();
        DISTRIBUTION.get_or_init(|| {
            FileBasedStringValuesDistribution::build_string_values_distribution(
                "buy_potential.dst",
                1,
                1,
            )
            .expect("Failed to load buy_potential.dst")
        })
    }

    /// Lazy-loaded distribution instance for dep_count.dst (DEP_COUNT_DISTRIBUTION)
    fn get_dep_count_distribution() -> &'static IntValuesDistribution {
        static DISTRIBUTION: OnceLock<IntValuesDistribution> = OnceLock::new();
        DISTRIBUTION.get_or_init(|| {
            IntValuesDistribution::build_int_values_distribution("dep_count.dst", 1, 1)
                .expect("Failed to load dep_count.dst")
        })
    }

    /// Lazy-loaded distribution instance for vehicle_count.dst (VEHICLE_COUNT_DISTRIBUTION)
    fn get_vehicle_count_distribution() -> &'static IntValuesDistribution {
        static DISTRIBUTION: OnceLock<IntValuesDistribution> = OnceLock::new();
        DISTRIBUTION.get_or_init(|| {
            IntValuesDistribution::build_int_values_distribution("vehicle_count.dst", 1, 1)
                .expect("Failed to load vehicle_count.dst")
        })
    }

    /// Get gender for index mod size (getGenderForIndexModSize)
    pub fn get_gender_for_index_mod_size(index: i64) -> &'static str {
        Self::get_gender_distribution().get_value_for_index_mod_size(index, 0)
    }

    /// Get marital status for index mod size (getMaritalStatusForIndexModSize)
    pub fn get_marital_status_for_index_mod_size(index: i64) -> &'static str {
        Self::get_marital_status_distribution().get_value_for_index_mod_size(index, 0)
    }

    /// Get education for index mod size (getEducationForIndexModSize)
    pub fn get_education_for_index_mod_size(index: i64) -> &'static str {
        Self::get_education_distribution().get_value_for_index_mod_size(index, 0)
    }

    /// Get purchase band for index mod size (getPurchaseBandForIndexModSize)
    pub fn get_purchase_band_for_index_mod_size(index: i64) -> i32 {
        Self::get_purchase_band_distribution().get_value_for_index_mod_size(index, 0)
    }

    /// Get credit rating for index mod size (getCreditRatingForIndexModSize)
    pub fn get_credit_rating_for_index_mod_size(index: i64) -> &'static str {
        Self::get_credit_rating_distribution().get_value_for_index_mod_size(index, 0)
    }

    /// Get gender distribution size
    pub fn get_gender_size() -> usize {
        Self::get_gender_distribution().get_size()
    }

    /// Get marital status distribution size
    pub fn get_marital_status_size() -> usize {
        Self::get_marital_status_distribution().get_size()
    }

    /// Get education distribution size
    pub fn get_education_size() -> usize {
        Self::get_education_distribution().get_size()
    }

    /// Get purchase band distribution size
    pub fn get_purchase_band_size() -> usize {
        Self::get_purchase_band_distribution().get_size()
    }

    /// Get credit rating distribution size
    pub fn get_credit_rating_size() -> usize {
        Self::get_credit_rating_distribution().get_size()
    }

    /// Get income band lower bound at the specified index (getValueAtIndex)
    pub fn get_income_band_lower_bound_at_index(index: usize) -> i32 {
        Self::get_income_band_distribution().get_value_at_index(0, index)
    }

    /// Get income band upper bound at the specified index (getValueAtIndex)
    pub fn get_income_band_upper_bound_at_index(index: usize) -> i32 {
        Self::get_income_band_distribution().get_value_at_index(1, index)
    }

    /// Get the size of the income band distribution
    pub fn get_income_band_size() -> usize {
        Self::get_income_band_distribution().get_value_count(0)
    }

    /// Get buy potential for index mod size (getBuyPotentialForIndexModSize)
    pub fn get_buy_potential_for_index_mod_size(index: i64) -> &'static str {
        Self::get_buy_potential_distribution().get_value_for_index_mod_size(index, 0)
    }

    /// Get dep count for index mod size (getDepCountForIndexModSize)
    pub fn get_dep_count_for_index_mod_size(index: i64) -> i32 {
        Self::get_dep_count_distribution().get_value_for_index_mod_size(index, 0)
    }

    /// Get vehicle count for index mod size (getVehicleCountForIndexModSize)
    pub fn get_vehicle_count_for_index_mod_size(index: i64) -> i32 {
        Self::get_vehicle_count_distribution().get_value_for_index_mod_size(index, 0)
    }

    /// Get buy potential distribution size
    pub fn get_buy_potential_size() -> usize {
        Self::get_buy_potential_distribution().get_size()
    }

    /// Get dep count distribution size
    pub fn get_dep_count_size() -> usize {
        Self::get_dep_count_distribution().get_size()
    }

    /// Get vehicle count distribution size
    pub fn get_vehicle_count_size() -> usize {
        Self::get_vehicle_count_distribution().get_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_income_band_distribution() {
        // Test that we can load the distribution
        let size = DemographicsDistributions::get_income_band_size();
        assert!(
            size > 0,
            "Income band distribution should have at least one entry"
        );

        // Test that we can get values at valid indices
        for i in 0..size.min(5) {
            let lower = DemographicsDistributions::get_income_band_lower_bound_at_index(i);
            let upper = DemographicsDistributions::get_income_band_upper_bound_at_index(i);
            assert!(
                lower <= upper,
                "Lower bound {lower} should be <= upper bound {upper} at index {i}"
            );
        }
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_income_band_out_of_bounds() {
        let size = DemographicsDistributions::get_income_band_size();
        DemographicsDistributions::get_income_band_lower_bound_at_index(size + 100);
    }
}
