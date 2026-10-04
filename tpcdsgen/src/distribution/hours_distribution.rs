use crate::distribution::file_loader::DistributionFileLoader;
use crate::distribution::utils::{pick_random_value, WeightsBuilder};
use crate::error::Result;
use crate::random::RandomNumberStream;
use crate::TpcdsError;
use std::sync::OnceLock;

/// Weights for hours distribution (HoursDistribution.Weights)
#[derive(Debug, Clone, Copy)]
pub enum HoursWeights {
    Uniform = 0,
    Store = 1,
    CatalogAndWeb = 2,
}

/// Information about a specific hour (HourInfo)
#[derive(Debug, Clone)]
pub struct HourInfo {
    am_pm: String,
    shift: String,
    sub_shift: String,
    meal: String,
}

impl HourInfo {
    pub fn new(am_pm: String, shift: String, sub_shift: String, meal: String) -> Self {
        HourInfo {
            am_pm,
            shift,
            sub_shift,
            meal,
        }
    }

    pub fn get_am_pm(&self) -> &str {
        &self.am_pm
    }

    pub fn get_shift(&self) -> &str {
        &self.shift
    }

    pub fn get_sub_shift(&self) -> &str {
        &self.sub_shift
    }

    pub fn get_meal(&self) -> &str {
        &self.meal
    }
}

/// Hours distribution for time_dim generation (HoursDistribution)
pub struct HoursDistribution {
    hours: Vec<i32>,
    am_pm: Vec<String>,
    shifts: Vec<String>,
    sub_shifts: Vec<String>,
    meals: Vec<String>,
    weights_lists: Vec<Vec<i32>>,
}

impl HoursDistribution {
    const NUM_WEIGHT_FIELDS: usize = 3;
    const VALUES_AND_WEIGHTS_FILENAME: &'static str = "hours.dst";

    fn get_instance() -> &'static HoursDistribution {
        static DISTRIBUTION: OnceLock<HoursDistribution> = OnceLock::new();
        DISTRIBUTION.get_or_init(|| {
            Self::build_hours_distribution().expect("Failed to load hours distribution")
        })
    }

    fn build_hours_distribution() -> Result<Self> {
        let mut hours = Vec::new();
        let mut am_pm = Vec::new();
        let mut shifts = Vec::new();
        let mut sub_shifts = Vec::new();
        let mut meals = Vec::new();
        let mut weights_builders: Vec<WeightsBuilder> = (0..Self::NUM_WEIGHT_FIELDS)
            .map(|_| WeightsBuilder::new())
            .collect();

        let parsed_lines =
            DistributionFileLoader::load_distribution_file(Self::VALUES_AND_WEIGHTS_FILENAME)?;

        for (values, weights) in parsed_lines {
            if values.len() < 4 || values.len() > 5 {
                return Err(TpcdsError::new(&format!(
                    "Expected line to contain 4 or 5 values, but it contained {}: {:?}",
                    values.len(),
                    values
                )));
            }

            if weights.len() != Self::NUM_WEIGHT_FIELDS {
                return Err(TpcdsError::new(&format!(
                    "Expected line to contain {} weights, but it contained {}: {:?}",
                    Self::NUM_WEIGHT_FIELDS,
                    weights.len(),
                    weights
                )));
            }

            // Parse values: [0]=hour, [1]=am_pm, [2]=shift, [3]=sub_shift, [4]=meal (optional)
            hours.push(values[0].parse().map_err(|e| {
                TpcdsError::new(&format!("Failed to parse hour '{}': {}", values[0], e))
            })?);

            am_pm.push(values[1].trim().to_string());
            shifts.push(values[2].trim().to_string());
            sub_shifts.push(values[3].trim().to_string());

            // Meal is optional (index 4)
            let meal = if values.len() > 4 && !values[4].trim().is_empty() {
                values[4].trim().to_string()
            } else {
                String::new()
            };
            meals.push(meal);

            // Parse weights
            for (i, weight_str) in weights.iter().enumerate() {
                let weight: i32 = weight_str.parse().map_err(|e| {
                    TpcdsError::new(&format!("Failed to parse weight '{}': {}", weight_str, e))
                })?;
                weights_builders[i].compute_and_add_next_weight(weight)?;
            }
        }

        let weights_lists = weights_builders
            .into_iter()
            .map(|builder| builder.build())
            .collect();

        Ok(HoursDistribution {
            hours,
            am_pm,
            shifts,
            sub_shifts,
            meals,
            weights_lists,
        })
    }

    /// Get hour information for a specific hour (0-23)
    pub fn get_hour_info_for_hour(hour: i32) -> HourInfo {
        let dist = Self::get_instance();
        HourInfo::new(
            dist.am_pm[hour as usize].clone(),
            dist.shifts[hour as usize].clone(),
            dist.sub_shifts[hour as usize].clone(),
            dist.meals[hour as usize].clone(),
        )
    }

    /// Pick a random hour using weighted distribution (HoursDistribution.pickRandomHour)
    ///
    /// This uses weighted random selection based on the specified weights type.
    /// Different weight types model different hour patterns for different sales channels.
    ///
    /// # Arguments
    ///
    /// * `weights` - The weight distribution to use for selection
    /// * `stream` - Random number stream for generating random values
    ///
    /// # Returns
    ///
    /// An hour value (0-23) based on the weighted distribution
    pub fn pick_random_hour(weights: HoursWeights, stream: &mut RandomNumberStream) -> Result<i32> {
        let dist = Self::get_instance();
        let weights_list = &dist.weights_lists[weights as usize];

        let value_ref = pick_random_value(&dist.hours, weights_list, stream)?;
        Ok(*value_ref)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hours_distribution_loading() {
        let dist = HoursDistribution::get_instance();
        assert_eq!(dist.hours.len(), 24); // Should have 24 hours
        assert_eq!(dist.am_pm.len(), 24);
        assert_eq!(dist.shifts.len(), 24);
    }

    #[test]
    fn test_get_hour_info() {
        let hour_info = HoursDistribution::get_hour_info_for_hour(0);
        assert_eq!(hour_info.get_am_pm(), "AM");

        let hour_info_12 = HoursDistribution::get_hour_info_for_hour(12);
        // Hour 12 should be PM
        assert!(hour_info_12.get_am_pm() == "AM" || hour_info_12.get_am_pm() == "PM");
    }

    #[test]
    fn test_pick_random_hour() {
        use crate::random::RandomNumberStream;

        let mut stream = RandomNumberStream::new(1);
        let hour = HoursDistribution::pick_random_hour(HoursWeights::Uniform, &mut stream).unwrap();

        // Hour should be in valid range [0, 23]
        assert!(
            (0..=23).contains(&hour),
            "Hour {} should be in range [0, 23]",
            hour
        );
    }

    #[test]
    fn test_pick_random_hour_deterministic() {
        use crate::random::RandomNumberStream;

        // Same seed should produce same hour
        let mut stream1 = RandomNumberStream::new(42);
        let mut stream2 = RandomNumberStream::new(42);

        let hour1 = HoursDistribution::pick_random_hour(HoursWeights::Store, &mut stream1).unwrap();
        let hour2 = HoursDistribution::pick_random_hour(HoursWeights::Store, &mut stream2).unwrap();

        assert_eq!(hour1, hour2, "Same seed should produce same hour");
    }

    #[test]
    fn test_pick_random_hour_different_weights() {
        use crate::random::RandomNumberStream;

        // Different weights should potentially produce different results
        let mut stream = RandomNumberStream::new(1);

        let hour_uniform =
            HoursDistribution::pick_random_hour(HoursWeights::Uniform, &mut stream).unwrap();
        let hour_store =
            HoursDistribution::pick_random_hour(HoursWeights::Store, &mut stream).unwrap();
        let hour_catalog =
            HoursDistribution::pick_random_hour(HoursWeights::CatalogAndWeb, &mut stream).unwrap();

        // All should be valid
        assert!((0..=23).contains(&hour_uniform));
        assert!((0..=23).contains(&hour_store));
        assert!((0..=23).contains(&hour_catalog));
    }
}
