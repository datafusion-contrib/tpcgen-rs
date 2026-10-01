use crate::distribution::string_values_distribution::StringValuesDistribution;
use crate::error::Result;
use crate::random::RandomNumberStream;
use std::sync::OnceLock;

/// Call center distributions (CallCenterDistributions)
pub struct CallCenterDistributions;

static CALL_CENTERS_DISTRIBUTION: OnceLock<StringValuesDistribution> = OnceLock::new();
static CALL_CENTER_CLASSES_DISTRIBUTION: OnceLock<StringValuesDistribution> = OnceLock::new();
static CALL_CENTER_HOURS_DISTRIBUTION: OnceLock<StringValuesDistribution> = OnceLock::new();

impl CallCenterDistributions {
    /// Initialize all distributions (lazy loading)
    fn ensure_initialized() -> Result<()> {
        // Initialize call centers distribution
        if CALL_CENTERS_DISTRIBUTION.get().is_none() {
            let dist = StringValuesDistribution::build_string_values_distribution(
                "call_centers.dst",
                1, // 1 value field: name
                2, // 2 weight fields: uniform, sales percentage
            )?;
            let _ = CALL_CENTERS_DISTRIBUTION.set(dist);
        }

        // Initialize call center classes distribution
        if CALL_CENTER_CLASSES_DISTRIBUTION.get().is_none() {
            let dist = StringValuesDistribution::build_string_values_distribution(
                "call_center_classes.dst",
                1, // 1 value field: class
                1, // 1 weight field: frequency
            )?;
            let _ = CALL_CENTER_CLASSES_DISTRIBUTION.set(dist);
        }

        // Initialize call center hours distribution
        if CALL_CENTER_HOURS_DISTRIBUTION.get().is_none() {
            let dist = StringValuesDistribution::build_string_values_distribution(
                "call_center_hours.dst",
                1, // 1 value field: hours
                1, // 1 weight field: frequency
            )?;
            let _ = CALL_CENTER_HOURS_DISTRIBUTION.set(dist);
        }

        Ok(())
    }

    /// Get call center name at specific index
    pub fn get_call_center_at_index(index: usize) -> Result<&'static str> {
        Self::ensure_initialized()?;
        let dist = CALL_CENTERS_DISTRIBUTION.get().unwrap();
        dist.get_value_at_index(0, index)
    }

    /// Get total number of call centers
    pub fn get_number_of_call_centers() -> Result<usize> {
        Self::ensure_initialized()?;
        let dist = CALL_CENTERS_DISTRIBUTION.get().unwrap();
        Ok(dist.get_size())
    }

    /// Pick a random call center class
    pub fn pick_random_call_center_class(stream: &mut RandomNumberStream) -> Result<&'static str> {
        Self::ensure_initialized()?;
        let dist = CALL_CENTER_CLASSES_DISTRIBUTION.get().unwrap();
        dist.pick_random_value(0, 0, stream)
    }

    /// Pick random call center hours
    pub fn pick_random_call_center_hours(stream: &mut RandomNumberStream) -> Result<&'static str> {
        Self::ensure_initialized()?;
        let dist = CALL_CENTER_HOURS_DISTRIBUTION.get().unwrap();
        dist.pick_random_value(0, 0, stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::RandomNumberStream;

    #[test]
    fn test_call_center_at_index() {
        let center = CallCenterDistributions::get_call_center_at_index(0).unwrap();
        assert!(!center.is_empty());
    }

    #[test]
    fn test_number_of_call_centers() {
        let count = CallCenterDistributions::get_number_of_call_centers().unwrap();
        assert!(count > 0);
    }

    #[test]
    fn test_pick_random_call_center_class() {
        let mut stream = RandomNumberStream::new(1).unwrap();
        let class = CallCenterDistributions::pick_random_call_center_class(&mut stream).unwrap();
        assert!(!class.is_empty());
    }

    #[test]
    fn test_pick_random_call_center_hours() {
        let mut stream = RandomNumberStream::new(1).unwrap();
        let hours = CallCenterDistributions::pick_random_call_center_hours(&mut stream).unwrap();
        assert!(!hours.is_empty());
    }

    #[test]
    fn test_deterministic_selection() {
        let mut stream1 = RandomNumberStream::new(42).unwrap();
        let mut stream2 = RandomNumberStream::new(42).unwrap();

        let class1 = CallCenterDistributions::pick_random_call_center_class(&mut stream1).unwrap();
        let class2 = CallCenterDistributions::pick_random_call_center_class(&mut stream2).unwrap();

        assert_eq!(class1, class2);
    }
}
