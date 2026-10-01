use crate::distribution::file_loader::DistributionFileLoader;
use crate::distribution::utils::{pick_random_value, WeightsBuilder};
use crate::error::Result;
use crate::random::RandomNumberStream;
use crate::TpcdsError;
use std::sync::OnceLock;

/// Weights for calendar distribution (CalendarDistribution.Weights)
#[derive(Debug, Clone, Copy)]
pub enum CalendarWeights {
    Uniform = 0,
    UniformLeapYear = 1,
    Sales = 2,
    SalesLeapYear = 3,
    Returns = 4,
    ReturnsLeapYear = 5,
    CombinedSkew = 6,
    Low = 7,
    Medium = 8,
    High = 9,
}

/// Calendar distribution for date_dim generation (CalendarDistribution)
pub struct CalendarDistribution {
    days_of_year: Vec<i32>,
    quarters: Vec<i32>,
    holiday_flags: Vec<i32>,
    weights_lists: Vec<Vec<i32>>,
}

impl CalendarDistribution {
    const NUM_WEIGHT_FIELDS: usize = 10;
    const VALUES_AND_WEIGHTS_FILENAME: &'static str = "calendar.dst";

    /// Days before each month (non-leap and leap year)
    pub const DAYS_BEFORE_MONTH: [[i32; 12]; 2] = [
        [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334], // Non-leap year
        [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335], // Leap year
    ];

    fn get_instance() -> &'static CalendarDistribution {
        static DISTRIBUTION: OnceLock<CalendarDistribution> = OnceLock::new();
        DISTRIBUTION.get_or_init(|| {
            Self::build_calendar_distribution().expect("Failed to load calendar distribution")
        })
    }

    fn build_calendar_distribution() -> Result<Self> {
        let mut days_of_year = Vec::new();
        let mut quarters = Vec::new();
        let mut holiday_flags = Vec::new();
        let mut weights_builders: Vec<WeightsBuilder> = (0..Self::NUM_WEIGHT_FIELDS)
            .map(|_| WeightsBuilder::new())
            .collect();

        let parsed_lines =
            DistributionFileLoader::load_distribution_file(Self::VALUES_AND_WEIGHTS_FILENAME)?;

        for (values, weights) in parsed_lines {
            if values.len() != 8 {
                return Err(TpcdsError::new(&format!(
                    "Expected line to contain 8 values, but it contained {}: {:?}",
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

            // Parse values (only use day_of_year, quarter, and holiday_flag)
            // Values are: [0]=day_of_year, [1]=month_name, [2]=day, [3]=season,
            //             [4]=month_number, [5]=quarter, [6]=first_of_month, [7]=holiday_flag
            days_of_year.push(values[0].parse().map_err(|e| {
                TpcdsError::new(&format!(
                    "Failed to parse day_of_year '{}': {}",
                    values[0], e
                ))
            })?);

            quarters.push(values[5].parse().map_err(|e| {
                TpcdsError::new(&format!("Failed to parse quarter '{}': {}", values[5], e))
            })?);

            holiday_flags.push(values[7].parse().map_err(|e| {
                TpcdsError::new(&format!(
                    "Failed to parse holiday_flag '{}': {}",
                    values[7], e
                ))
            })?);

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

        Ok(CalendarDistribution {
            days_of_year,
            quarters,
            holiday_flags,
            weights_lists,
        })
    }

    /// Get the quarter for a given day index (1-based index)
    pub fn get_quarter_at_index(index: i32) -> i32 {
        let dist = Self::get_instance();
        dist.quarters[(index - 1) as usize]
    }

    /// Get the holiday flag for a given day index (1-based index)
    pub fn get_is_holiday_flag_at_index(index: i32) -> i32 {
        let dist = Self::get_instance();
        dist.holiday_flags[(index - 1) as usize]
    }

    /// Pick a random day of year using weighted distribution (CalendarDistribution.pickRandomDayOfYear)
    ///
    /// This uses weighted random selection based on the specified weights type.
    /// Different weight types model different temporal patterns (sales, returns, uniform, etc.)
    ///
    /// # Arguments
    ///
    /// * `weights` - The weight distribution to use for selection
    /// * `stream` - Random number stream for generating random values
    ///
    /// # Returns
    ///
    /// A day of year value (1-366) based on the weighted distribution
    pub fn pick_random_day_of_year(
        weights: CalendarWeights,
        stream: &mut RandomNumberStream,
    ) -> Result<i32> {
        let dist = Self::get_instance();
        let weights_list = &dist.weights_lists[weights as usize];

        let value_ref = pick_random_value(&dist.days_of_year, weights_list, stream)?;
        Ok(*value_ref)
    }

    /// Get the 0-based index into the calendar distribution for a given date.
    ///
    /// Based on CalendarDistribution.getIndexForDate in Java.
    pub fn get_index_for_date(date: &crate::types::Date) -> i32 {
        let leap_year_index = if crate::types::Date::is_leap_year(date.year()) {
            1
        } else {
            0
        };
        Self::DAYS_BEFORE_MONTH[leap_year_index][(date.month() - 1) as usize] + date.day() - 1
    }

    /// Get the weight for a specific day number using the specified weights.
    ///
    /// Based on CalendarDistribution.getWeightForDayNumber in Java.
    /// This reverses the cumulative weights to get the raw (individual day) weight,
    /// matching Java's DistributionUtils.getWeightForIndex behavior.
    pub fn get_weight_for_day_number(day_number: i32, weights: CalendarWeights) -> i32 {
        let dist = Self::get_instance();
        let weights_list = &dist.weights_lists[weights as usize];
        // Reverse the accumulation to get raw weight (like Java's getWeightForIndex)
        if day_number == 0 {
            weights_list[0]
        } else {
            weights_list[day_number as usize] - weights_list[(day_number - 1) as usize]
        }
    }

    /// Get the maximum weight for the specified weights distribution.
    ///
    /// Based on CalendarDistribution.getMaxWeight in Java.
    /// The max weight is the last element in the cumulative weights list.
    pub fn get_max_weight(weights: CalendarWeights) -> i32 {
        let dist = Self::get_instance();
        let weights_list = &dist.weights_lists[weights as usize];
        *weights_list.last().unwrap_or(&0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calendar_distribution_loading() {
        let dist = CalendarDistribution::get_instance();
        assert!(!dist.days_of_year.is_empty());
        assert_eq!(dist.quarters.len(), dist.days_of_year.len());
        assert_eq!(dist.holiday_flags.len(), dist.days_of_year.len());
    }

    #[test]
    fn test_get_quarter_at_index() {
        // Day 1 (Jan 1) should be in Q1
        let quarter = CalendarDistribution::get_quarter_at_index(1);
        assert_eq!(quarter, 1);
    }

    #[test]
    fn test_days_before_month() {
        // Non-leap year
        assert_eq!(CalendarDistribution::DAYS_BEFORE_MONTH[0][0], 0); // January
        assert_eq!(CalendarDistribution::DAYS_BEFORE_MONTH[0][1], 31); // February

        // Leap year
        assert_eq!(CalendarDistribution::DAYS_BEFORE_MONTH[1][2], 60); // March in leap year
    }

    #[test]
    fn test_pick_random_day_of_year() {
        use crate::random::RandomNumberStream;

        let mut stream = RandomNumberStream::new(1).unwrap();
        let day =
            CalendarDistribution::pick_random_day_of_year(CalendarWeights::Uniform, &mut stream)
                .unwrap();

        // Day should be in valid range [1, 366]
        assert!(
            (1..=366).contains(&day),
            "Day {} should be in range [1, 366]",
            day
        );
    }

    #[test]
    fn test_pick_random_day_of_year_deterministic() {
        use crate::random::RandomNumberStream;

        // Same seed should produce same day
        let mut stream1 = RandomNumberStream::new(42).unwrap();
        let mut stream2 = RandomNumberStream::new(42).unwrap();

        let day1 =
            CalendarDistribution::pick_random_day_of_year(CalendarWeights::Sales, &mut stream1)
                .unwrap();
        let day2 =
            CalendarDistribution::pick_random_day_of_year(CalendarWeights::Sales, &mut stream2)
                .unwrap();

        assert_eq!(day1, day2, "Same seed should produce same day");
    }

    #[test]
    fn test_pick_random_day_of_year_different_weights() {
        use crate::random::RandomNumberStream;

        // Different weights should potentially produce different results
        let mut stream = RandomNumberStream::new(1).unwrap();

        let day_uniform =
            CalendarDistribution::pick_random_day_of_year(CalendarWeights::Uniform, &mut stream)
                .unwrap();
        let day_sales =
            CalendarDistribution::pick_random_day_of_year(CalendarWeights::Sales, &mut stream)
                .unwrap();

        // Both should be valid
        assert!((1..=366).contains(&day_uniform));
        assert!((1..=366).contains(&day_sales));
    }
}
