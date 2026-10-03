use crate::distribution::{Distribution, DistributionUtils, WeightsBuilder};
use crate::random::RandomNumberStream;
use crate::{error::Result, TpcdsError};

/// String-based weighted distribution (StringValuesDistribution)
#[derive(Debug, Clone)]
pub struct StringValuesDistribution {
    values_lists: Vec<Vec<String>>,
    weights_lists: Vec<Vec<i32>>,
}

impl StringValuesDistribution {
    /// Create new distribution with given values and weights lists
    pub fn new(values_lists: Vec<Vec<String>>, weights_lists: Vec<Vec<i32>>) -> Result<Self> {
        // Validate that values and weights lists have same structure
        if values_lists.len() != weights_lists.len() {
            return Err(TpcdsError::new(
                "Values and weights lists must have same number of lists",
            ));
        }

        for (i, (values, weights)) in values_lists.iter().zip(weights_lists.iter()).enumerate() {
            if values.len() != weights.len() {
                return Err(TpcdsError::new(&format!(
                    "Values list {} and weights list {} must have same length",
                    i, i
                )));
            }
        }

        Ok(StringValuesDistribution {
            values_lists,
            weights_lists,
        })
    }

    /// Create distribution from embedded data (for immediate use without files)
    pub fn from_embedded_data(data: &[(&str, i32)]) -> Result<Self> {
        let mut values = Vec::new();
        let mut weights_builder = WeightsBuilder::new();

        for (value, weight) in data {
            values.push(value.to_string());
            weights_builder.compute_and_add_next_weight(*weight)?;
        }

        Ok(StringValuesDistribution {
            values_lists: vec![values],
            weights_lists: vec![weights_builder.build()],
        })
    }

    /// Create distribution from DST-style data with multiple weight columns
    pub fn from_multi_weight_data(data: &[(&str, &[i32])]) -> Result<Self> {
        if data.is_empty() {
            return Ok(StringValuesDistribution {
                values_lists: vec![],
                weights_lists: vec![],
            });
        }

        let num_weight_columns = data[0].1.len();
        let mut values = Vec::new();
        let mut weights_builders: Vec<WeightsBuilder> = (0..num_weight_columns)
            .map(|_| WeightsBuilder::new())
            .collect();

        for (value, weights) in data {
            if weights.len() != num_weight_columns {
                return Err(TpcdsError::new(
                    "All data entries must have same number of weights",
                ));
            }

            values.push(value.to_string());
            for (i, &weight) in weights.iter().enumerate() {
                weights_builders[i].compute_and_add_next_weight(weight)?;
            }
        }

        let weights_lists: Vec<Vec<i32>> = weights_builders
            .into_iter()
            .map(|builder| builder.build())
            .collect();

        Ok(StringValuesDistribution {
            values_lists: vec![values],
            weights_lists,
        })
    }

    /// Get number of value lists
    pub fn get_value_lists_count(&self) -> usize {
        self.values_lists.len()
    }

    /// Get number of weight lists  
    pub fn get_weight_lists_count(&self) -> usize {
        self.weights_lists.len()
    }
}

impl Distribution<String> for StringValuesDistribution {
    /// Pick random value based on weights (core method matching Java)
    fn pick_random_value(
        &self,
        value_list: usize,
        weight_list: usize,
        stream: &mut RandomNumberStream,
    ) -> String {
        let values = &self.values_lists[value_list];
        let weights = &self.weights_lists[weight_list];
        let index = DistributionUtils::pick_random_index_from_weights(weights, stream);
        values[index].clone()
    }

    /// Get value at specific index
    fn get_value_at_index(&self, value_list: usize, index: usize) -> String {
        self.values_lists[value_list][index].clone()
    }

    /// Get number of values in a list
    fn get_value_count(&self, value_list: usize) -> usize {
        if value_list >= self.values_lists.len() {
            0
        } else {
            self.values_lists[value_list].len()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::RandomNumberStream;

    #[test]
    fn test_from_embedded_data() {
        let data = &[("apple", 30), ("banana", 20), ("cherry", 50)];

        let dist = StringValuesDistribution::from_embedded_data(data).unwrap();
        assert_eq!(dist.get_value_lists_count(), 1);
        assert_eq!(dist.get_weight_lists_count(), 1);
        assert_eq!(dist.get_value_count(0), 3);

        // Test value access
        assert_eq!(dist.get_value_at_index(0, 0), "apple");
        assert_eq!(dist.get_value_at_index(0, 1), "banana");
        assert_eq!(dist.get_value_at_index(0, 2), "cherry");
    }

    #[test]
    fn test_from_multi_weight_data() {
        let weights1 = [0, 317000];
        let weights2 = [4031, 4031];
        let weights3 = [2500, 2500];
        let data = &[
            ("", weights1.as_slice()), // Empty string - weight 0 for first, 317000 for second
            ("Church", weights2.as_slice()), // Church - weight 4031 for both
            ("Main", weights3.as_slice()), // Main - weight 2500 for both
        ];

        let dist = StringValuesDistribution::from_multi_weight_data(data).unwrap();
        assert_eq!(dist.get_value_lists_count(), 1);
        assert_eq!(dist.get_weight_lists_count(), 2); // Two weight columns
        assert_eq!(dist.get_value_count(0), 3);
    }

    #[test]
    fn test_pick_random_value() {
        let data = &[("rare", 1), ("common", 99)];

        let dist = StringValuesDistribution::from_embedded_data(data).unwrap();
        let mut stream = RandomNumberStream::new(1).unwrap();

        // Test multiple picks - should all be valid
        for _ in 0..10 {
            let value = dist.pick_random_value(0, 0, &mut stream);
            assert!(value == "rare" || value == "common");
        }
    }

    #[test]
    fn test_pick_from_multi_weight() {
        let weights1 = [0, 100];
        let weights2 = [50, 0];
        let data = &[
            ("empty", weights1.as_slice()), // Never picked from first weight, always from second
            ("value", weights2.as_slice()), // Sometimes picked from first, never from second
        ];

        let dist = StringValuesDistribution::from_multi_weight_data(data).unwrap();
        let mut stream = RandomNumberStream::new(1).unwrap();

        // Pick from second weight list (index 1) - should only get "empty"
        let value = dist.pick_random_value(0, 1, &mut stream);
        assert_eq!(value, "empty");

        // Note: First weight list would be probabilistic, but we can test it works
        let value = dist.pick_random_value(0, 0, &mut stream);
        assert!(value == "empty" || value == "value");
    }

    #[test]
    fn test_deterministic_behavior() {
        let data = &[("first", 25), ("second", 25), ("third", 25), ("fourth", 25)];

        let dist = StringValuesDistribution::from_embedded_data(data).unwrap();

        // Same seed should produce same results
        let mut stream1 = RandomNumberStream::new_with_column(42, 1).unwrap();
        let mut stream2 = RandomNumberStream::new_with_column(42, 1).unwrap();

        let value1 = dist.pick_random_value(0, 0, &mut stream1);
        let value2 = dist.pick_random_value(0, 0, &mut stream2);

        assert_eq!(value1, value2);
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_invalid_index_panics() {
        let data = &[("test", 100)];
        let dist = StringValuesDistribution::from_embedded_data(data).unwrap();
        dist.get_value_at_index(0, 1);
    }

    #[test]
    fn test_validation() {
        // Mismatched list counts
        assert!(StringValuesDistribution::new(
            vec![vec!["a".to_string()]],
            vec![] // Empty weights but non-empty values
        )
        .is_err());

        // Mismatched value/weight lengths
        assert!(StringValuesDistribution::new(
            vec![vec!["a".to_string(), "b".to_string()]], // 2 values
            vec![vec![100]]                               // 1 weight
        )
        .is_err());
    }
}
