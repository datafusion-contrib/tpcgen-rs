use crate::random::RandomNumberStream;
use crate::{check_argument, error::Result, TpcdsError};

/// Core trait for weighted distributions
pub trait Distribution<T> {
    /// Pick a random value based on weights
    fn pick_random_value(
        &self,
        value_list: usize,
        weight_list: usize,
        stream: &mut dyn RandomNumberStream,
    ) -> Result<T>;

    /// Get value at specific index
    fn get_value_at_index(&self, value_list: usize, index: usize) -> Result<T>;

    /// Get number of values in a list
    fn get_value_count(&self, value_list: usize) -> usize;
}

/// Builds cumulative weights for weighted selection (WeightsBuilder)
#[derive(Debug, Clone)]
pub struct WeightsBuilder {
    weights: Vec<i32>,
    previous_weight: i32,
}

impl WeightsBuilder {
    pub fn new() -> Self {
        WeightsBuilder {
            weights: Vec::new(),
            previous_weight: 0,
        }
    }

    /// Compute and add next cumulative weight
    pub fn compute_and_add_next_weight(&mut self, weight: i32) -> Result<&mut Self> {
        check_argument!(weight >= 0, "Weight cannot be negative.");
        let new_weight = self.previous_weight + weight;
        self.weights.push(new_weight);
        self.previous_weight = new_weight;
        Ok(self)
    }

    /// Build final immutable weights list
    pub fn build(self) -> Vec<i32> {
        self.weights
    }

    /// Get current total weight
    pub fn get_total_weight(&self) -> i32 {
        self.previous_weight
    }
}

/// Pick a random value from values list based on weights (DistributionUtils.pickRandomValue)
///
/// Weights must be nondecreasing.
pub fn pick_random_value<'a, T>(
    values: &'a [T],
    weights: &[i32],
    stream: &mut dyn RandomNumberStream,
) -> Result<&'a T> {
    use crate::random::RandomValueGenerator;

    if values.len() != weights.len() {
        return Err(TpcdsError::new(
            "Values and weights lists must be the same size",
        ));
    }

    if weights.is_empty() {
        return Err(TpcdsError::new("Cannot pick from empty distribution"));
    }

    let max_weight = weights[weights.len() - 1];
    let random_weight = RandomValueGenerator::generate_uniform_random_int(1, max_weight, stream);

    get_value_for_weight(random_weight, values, weights)
}

/// Get value for specific weight (DistributionUtils.getValueForWeight)
fn get_value_for_weight<'a, T>(weight: i32, values: &'a [T], weights: &[i32]) -> Result<&'a T> {
    if values.len() != weights.len() {
        return Err(TpcdsError::new(
            "Values and weights lists must be the same size",
        ));
    }

    let index = get_index_for_weight(weight, weights)?;
    Ok(&values[index])
}

/// Get value for index modulo size (DistributionUtils.getValueForIndexModSize)
pub fn get_value_for_index_mod_size<T>(index: i64, values: &[T]) -> &T {
    let size = values.len() as i64;
    let index_mod_size = (index % size) as usize;
    &values[index_mod_size]
}

/// Pick random index from weights (DistributionUtils.pickRandomIndex)
///
/// Weights must be nondecreasing.
pub fn pick_random_index(weights: &[i32], stream: &mut dyn RandomNumberStream) -> Result<usize> {
    use crate::random::RandomValueGenerator;

    if weights.is_empty() {
        return Err(TpcdsError::new("Cannot pick from empty weights"));
    }

    let max_weight = weights[weights.len() - 1];
    let random_weight = RandomValueGenerator::generate_uniform_random_int(1, max_weight, stream);

    get_index_for_weight(random_weight, weights)
}

/// Get index for specific weight (DistributionUtils.getIndexForWeight)
fn get_index_for_weight(weight: i32, weights: &[i32]) -> Result<usize> {
    let index = weights.partition_point(|&w| w < weight);
    if index < weights.len() {
        Ok(index)
    } else {
        Err(TpcdsError::new("Random weight was greater than max weight"))
    }
}

/// Get weight for specific index (DistributionUtils.getWeightForIndex)
pub fn get_weight_for_index(index: usize, weights: &[i32]) -> Result<i32> {
    if index >= weights.len() {
        return Err(TpcdsError::new(&format!(
            "Index {} larger than distribution size {}",
            index,
            weights.len()
        )));
    }

    // Reverse the accumulation of weights
    if index == 0 {
        Ok(weights[index])
    } else {
        Ok(weights[index] - weights[index - 1])
    }
}

impl Default for WeightsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Core distribution utilities (DistributionUtils)
pub struct DistributionUtils;

impl DistributionUtils {
    /// Pick random index based on cumulative weights (core algorithm from Java)
    ///
    /// Weights must be nondecreasing.
    pub fn pick_random_index_from_weights(
        weights: &[i32],
        stream: &mut dyn RandomNumberStream,
    ) -> Result<usize> {
        if weights.is_empty() {
            return Err(TpcdsError::new("Cannot pick from empty weights"));
        }

        let max_weight = *weights.last().unwrap();
        if max_weight <= 0 {
            return Err(TpcdsError::new("Total weight must be positive"));
        }

        // Generate random number in range [1, max_weight] (inclusive)
        // This matches the Java implementation exactly
        let random_weight =
            crate::random::RandomValueGenerator::generate_uniform_random_int(1, max_weight, stream);

        get_index_for_weight(random_weight, weights)
    }

    /// Pick random index with uniform distribution (for non-weighted selection)
    pub fn pick_random_index_uniform(
        count: usize,
        stream: &mut dyn RandomNumberStream,
    ) -> Result<usize> {
        if count == 0 {
            return Err(TpcdsError::new("Cannot pick from empty collection"));
        }

        let index = crate::random::RandomValueGenerator::generate_uniform_random_int(
            0,
            count as i32 - 1,
            stream,
        );
        Ok(index as usize)
    }

    /// Parse comma-separated list of values (utility for file parsing)
    pub fn parse_comma_separated_values(line: &str) -> Vec<String> {
        line.split(',').map(|s| s.trim().to_string()).collect()
    }

    /// Parse colon-separated value and weight(s) from .dst format
    pub fn parse_dst_line(line: &str) -> Result<(String, Vec<i32>)> {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with("--") || line.starts_with("//") {
            return Err(TpcdsError::new("Skip comment or empty line"));
        }

        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() != 2 {
            return Err(TpcdsError::new(&format!(
                "Invalid .dst line format: {}",
                line
            )));
        }

        let value = parts[0].trim().to_string();
        let weights_str = parts[1].trim();

        // Parse weights (can be single or comma-separated)
        let weights: Result<Vec<i32>> = weights_str
            .split(',')
            .map(|w| {
                w.trim()
                    .parse::<i32>()
                    .map_err(|_| TpcdsError::new(&format!("Invalid weight: {}", w)))
            })
            .collect();

        Ok((value, weights?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::RandomNumberStreamImpl;

    #[test]
    fn test_weights_builder() {
        let mut builder = WeightsBuilder::new();
        builder.compute_and_add_next_weight(10).unwrap();
        builder.compute_and_add_next_weight(20).unwrap();
        builder.compute_and_add_next_weight(30).unwrap();

        let weights = builder.build();
        assert_eq!(weights, vec![10, 30, 60]); // Cumulative weights
    }

    #[test]
    fn test_weights_builder_negative_weight() {
        let mut builder = WeightsBuilder::new();
        assert!(builder.compute_and_add_next_weight(-5).is_err());
    }

    #[test]
    fn test_pick_random_index_from_weights() {
        let mut stream = RandomNumberStreamImpl::new(1).unwrap();
        let weights = vec![10, 30, 60, 100]; // Cumulative weights

        // Test multiple selections to ensure they're in valid range
        for _ in 0..10 {
            let index =
                DistributionUtils::pick_random_index_from_weights(&weights, &mut stream).unwrap();
            assert!(index < weights.len());
        }
    }

    #[test]
    fn test_weight_lookup_boundaries() {
        let weights = [0, 10, 10, 20, 40];
        let values = ["zero", "first ten", "second ten", "twenty", "forty"];

        assert_eq!(get_index_for_weight(0, &weights).unwrap(), 0);
        assert_eq!(get_index_for_weight(1, &weights).unwrap(), 1);
        assert_eq!(get_index_for_weight(10, &weights).unwrap(), 1);
        assert_eq!(get_index_for_weight(11, &weights).unwrap(), 3);
        assert_eq!(get_index_for_weight(40, &weights).unwrap(), 4);
        assert!(get_index_for_weight(41, &weights).is_err());
        assert!(get_index_for_weight(1, &[]).is_err());

        assert_eq!(
            get_value_for_weight(10, &values, &weights).unwrap(),
            &"first ten"
        );
        assert_eq!(
            get_value_for_weight(11, &values, &weights).unwrap(),
            &"twenty"
        );
        assert!(get_value_for_weight(1, &[] as &[i32], &[]).is_err());
        assert!(get_value_for_weight(1, &values[..4], &weights).is_err());
    }

    #[test]
    fn test_pick_random_index_uniform() {
        let mut stream = RandomNumberStreamImpl::new(1).unwrap();

        for _ in 0..10 {
            let index = DistributionUtils::pick_random_index_uniform(5, &mut stream).unwrap();
            assert!(index < 5);
        }
    }

    #[test]
    fn test_pick_random_index_empty() {
        let mut stream = RandomNumberStreamImpl::new(1).unwrap();
        assert!(DistributionUtils::pick_random_index_from_weights(&[], &mut stream).is_err());
        assert!(DistributionUtils::pick_random_index_uniform(0, &mut stream).is_err());
    }

    #[test]
    fn test_parse_comma_separated_values() {
        let result = DistributionUtils::parse_comma_separated_values("a, b , c,d");
        assert_eq!(result, vec!["a", "b", "c", "d"]);
    }

    #[test]
    fn test_parse_dst_line() {
        // Simple value:weight
        let (value, weights) = DistributionUtils::parse_dst_line("then:619").unwrap();
        assert_eq!(value, "then");
        assert_eq!(weights, vec![619]);

        // Multi-weight
        let (value, weights) = DistributionUtils::parse_dst_line(": 0, 317000").unwrap();
        assert_eq!(value, "");
        assert_eq!(weights, vec![0, 317000]);

        // Value with spaces
        let (value, weights) = DistributionUtils::parse_dst_line("Church: 4031, 4031").unwrap();
        assert_eq!(value, "Church");
        assert_eq!(weights, vec![4031, 4031]);

        // Comments should error
        assert!(DistributionUtils::parse_dst_line("-- comment").is_err());
        assert!(DistributionUtils::parse_dst_line("").is_err());

        // Invalid format
        assert!(DistributionUtils::parse_dst_line("invalid_format").is_err());
    }

    #[test]
    fn test_deterministic_selection() {
        // Test that same seed produces same results
        let weights = vec![25, 50, 75, 100];

        let mut stream1 = RandomNumberStreamImpl::new_with_column(1, 1).unwrap();
        let mut stream2 = RandomNumberStreamImpl::new_with_column(1, 1).unwrap();

        let index1 =
            DistributionUtils::pick_random_index_from_weights(&weights, &mut stream1).unwrap();
        let index2 =
            DistributionUtils::pick_random_index_from_weights(&weights, &mut stream2).unwrap();

        assert_eq!(index1, index2); // Should be deterministic
    }
}
