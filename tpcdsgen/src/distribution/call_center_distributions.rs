use crate::distribution::string_values_distribution::StringValuesDistribution;
use crate::random::RandomNumberStream;
use std::sync::OnceLock;

/// Call center distributions (CallCenterDistributions)
pub struct CallCenterDistributions;

static CALL_CENTERS_DISTRIBUTION: OnceLock<StringValuesDistribution> = OnceLock::new();
static CALL_CENTER_CLASSES_DISTRIBUTION: OnceLock<StringValuesDistribution> = OnceLock::new();
static CALL_CENTER_HOURS_DISTRIBUTION: OnceLock<StringValuesDistribution> = OnceLock::new();

impl CallCenterDistributions {
    fn call_centers() -> &'static StringValuesDistribution {
        CALL_CENTERS_DISTRIBUTION.get_or_init(|| {
            // 1 value field: name; 2 weight fields: uniform, sales percentage
            StringValuesDistribution::build_string_values_distribution("call_centers.dst", 1, 2)
                .expect("Failed to load call_centers.dst")
        })
    }

    fn call_center_classes() -> &'static StringValuesDistribution {
        CALL_CENTER_CLASSES_DISTRIBUTION.get_or_init(|| {
            StringValuesDistribution::build_string_values_distribution(
                "call_center_classes.dst",
                1,
                1,
            )
            .expect("Failed to load call_center_classes.dst")
        })
    }

    fn call_center_hours() -> &'static StringValuesDistribution {
        CALL_CENTER_HOURS_DISTRIBUTION.get_or_init(|| {
            StringValuesDistribution::build_string_values_distribution(
                "call_center_hours.dst",
                1,
                1,
            )
            .expect("Failed to load call_center_hours.dst")
        })
    }

    /// Get call center name at specific index
    pub fn get_call_center_at_index(index: usize) -> &'static str {
        Self::call_centers().get_value_at_index(0, index)
    }

    /// Get total number of call centers
    pub fn get_number_of_call_centers() -> usize {
        Self::call_centers().get_size()
    }

    /// Pick a random call center class
    pub fn pick_random_call_center_class(stream: &mut RandomNumberStream) -> &'static str {
        Self::call_center_classes().pick_random_value(0, 0, stream)
    }

    /// Pick random call center hours
    pub fn pick_random_call_center_hours(stream: &mut RandomNumberStream) -> &'static str {
        Self::call_center_hours().pick_random_value(0, 0, stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::RandomNumberStream;

    #[test]
    fn test_call_center_at_index() {
        let center = CallCenterDistributions::get_call_center_at_index(0);
        assert!(!center.is_empty());
    }

    #[test]
    fn test_number_of_call_centers() {
        let count = CallCenterDistributions::get_number_of_call_centers();
        assert!(count > 0);
    }

    #[test]
    fn test_pick_random_call_center_class() {
        let mut stream = RandomNumberStream::new(1);
        let class = CallCenterDistributions::pick_random_call_center_class(&mut stream);
        assert!(!class.is_empty());
    }

    #[test]
    fn test_pick_random_call_center_hours() {
        let mut stream = RandomNumberStream::new(1);
        let hours = CallCenterDistributions::pick_random_call_center_hours(&mut stream);
        assert!(!hours.is_empty());
    }

    #[test]
    fn test_deterministic_selection() {
        let mut stream1 = RandomNumberStream::new(42);
        let mut stream2 = RandomNumberStream::new(42);

        let class1 = CallCenterDistributions::pick_random_call_center_class(&mut stream1);
        let class2 = CallCenterDistributions::pick_random_call_center_class(&mut stream2);

        assert_eq!(class1, class2);
    }
}
