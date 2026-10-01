use crate::distribution::{
    Distribution, DistributionFileLoader, DistributionUtils, WeightsBuilder,
};
use crate::random::RandomNumberStream;
use crate::{error::Result, TpcdsError};

/// Integer-based weighted distribution (IntValuesDistribution)
#[derive(Debug, Clone)]
pub struct IntValuesDistribution {
    values_lists: Vec<Vec<i32>>,
    weights_lists: Vec<Vec<i32>>,
}

impl IntValuesDistribution {
    /// Create new distribution with given values and weights lists
    pub fn new(values_lists: Vec<Vec<i32>>, weights_lists: Vec<Vec<i32>>) -> Result<Self> {
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

        Ok(IntValuesDistribution {
            values_lists,
            weights_lists,
        })
    }

    /// Create distribution from embedded data (for immediate use without files)
    pub fn from_embedded_data(data: &[(i32, i32)]) -> Result<Self> {
        let mut values = Vec::new();
        let mut weights_builder = WeightsBuilder::new();

        for (value, weight) in data {
            values.push(*value);
            weights_builder.compute_and_add_next_weight(*weight)?;
        }

        Ok(IntValuesDistribution {
            values_lists: vec![values],
            weights_lists: vec![weights_builder.build()],
        })
    }

    /// Create distribution from DST-style data with multiple weight columns
    pub fn from_multi_weight_data(data: &[(i32, &[i32])]) -> Result<Self> {
        if data.is_empty() {
            return Ok(IntValuesDistribution {
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

            values.push(*value);
            for (i, &weight) in weights.iter().enumerate() {
                weights_builders[i].compute_and_add_next_weight(weight)?;
            }
        }

        let weights_lists: Vec<Vec<i32>> = weights_builders
            .into_iter()
            .map(|builder| builder.build())
            .collect();

        Ok(IntValuesDistribution {
            values_lists: vec![values],
            weights_lists,
        })
    }

    /// Create uniform distribution (all values have equal weight)
    pub fn uniform(values: &[i32]) -> Result<Self> {
        let data: Vec<(i32, i32)> = values.iter().map(|&v| (v, 1)).collect();
        Self::from_embedded_data(&data)
    }

    /// Build an IntValuesDistribution from a distribution file
    ///
    /// # Arguments
    /// * `filename` - The .dst file to load
    /// * `num_value_fields` - Number of value fields per line (integer values)
    /// * `num_weight_fields` - Number of weight fields per line
    pub fn build_int_values_distribution(
        filename: &str,
        num_value_fields: usize,
        num_weight_fields: usize,
    ) -> Result<Self> {
        let parsed_lines = DistributionFileLoader::load_distribution_file(filename)?;

        let mut values_builders: Vec<Vec<i32>> = vec![Vec::new(); num_value_fields];
        let mut weights_builders: Vec<WeightsBuilder> =
            vec![WeightsBuilder::new(); num_weight_fields];

        for (values, weights) in parsed_lines {
            if values.len() != num_value_fields {
                return Err(TpcdsError::new(&format!(
                    "Expected line to contain {} values, but it contained {}: {:?}",
                    num_value_fields,
                    values.len(),
                    values
                )));
            }

            if weights.len() != num_weight_fields {
                return Err(TpcdsError::new(&format!(
                    "Expected line to contain {} weights, but it contained {}: {:?}",
                    num_weight_fields,
                    weights.len(),
                    weights
                )));
            }

            // Add values to builders - parse as integers
            for (i, value) in values.into_iter().enumerate() {
                let int_value: i32 = value.parse().map_err(|e| {
                    TpcdsError::new(&format!(
                        "Failed to parse value '{}' as integer: {}",
                        value, e
                    ))
                })?;
                values_builders[i].push(int_value);
            }

            // Add weights to builders
            for (i, weight_str) in weights.into_iter().enumerate() {
                let weight: i32 = weight_str.parse().map_err(|e| {
                    TpcdsError::new(&format!("Failed to parse weight '{}': {}", weight_str, e))
                })?;
                weights_builders[i].compute_and_add_next_weight(weight)?;
            }
        }

        let values_lists = values_builders;
        let weights_lists = weights_builders
            .into_iter()
            .map(|builder| builder.build())
            .collect();

        Ok(IntValuesDistribution {
            values_lists,
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

    /// Get value count for a specific value list
    pub fn get_value_count(&self, value_list: usize) -> usize {
        if value_list >= self.values_lists.len() {
            return 0;
        }
        self.values_lists[value_list].len()
    }

    /// Get the size of the distribution (number of entries in first value list)
    pub fn get_size(&self) -> usize {
        if self.values_lists.is_empty() {
            0
        } else {
            self.values_lists[0].len()
        }
    }

    /// Get a value by index modulo the size of the list (getValueForIndexModSize)
    pub fn get_value_for_index_mod_size(&self, index: i64, value_list_index: usize) -> i32 {
        if value_list_index >= self.values_lists.len() {
            panic!("Value list index {} out of range", value_list_index);
        }

        let values = &self.values_lists[value_list_index];
        if values.is_empty() {
            panic!("Cannot get value from empty distribution");
        }

        let actual_index = (index as usize) % values.len();
        values[actual_index]
    }

    /// Pick a random index from the specified weight list
    pub fn pick_random_index(
        &self,
        weight_list_index: usize,
        stream: &mut RandomNumberStream,
    ) -> Result<usize> {
        if weight_list_index >= self.weights_lists.len() {
            return Err(TpcdsError::new(&format!(
                "Weight list index {} out of range, max is {}",
                weight_list_index,
                self.weights_lists.len().saturating_sub(1)
            )));
        }

        DistributionUtils::pick_random_index_from_weights(
            &self.weights_lists[weight_list_index],
            stream,
        )
    }
}

impl Distribution<i32> for IntValuesDistribution {
    /// Pick random value based on weights (core method matching Java)
    fn pick_random_value(
        &self,
        value_list: usize,
        weight_list: usize,
        stream: &mut RandomNumberStream,
    ) -> Result<i32> {
        if value_list >= self.values_lists.len() {
            return Err(TpcdsError::new(&format!(
                "Value list index {} out of bounds",
                value_list
            )));
        }
        if weight_list >= self.weights_lists.len() {
            return Err(TpcdsError::new(&format!(
                "Weight list index {} out of bounds",
                weight_list
            )));
        }

        let values = &self.values_lists[value_list];
        let weights = &self.weights_lists[weight_list];

        if values.len() != weights.len() {
            return Err(TpcdsError::new(
                "Values and weights lists have different lengths",
            ));
        }

        if values.is_empty() {
            return Err(TpcdsError::new("Cannot pick from empty distribution"));
        }

        let index = DistributionUtils::pick_random_index_from_weights(weights, stream)?;
        if index >= values.len() {
            return Err(TpcdsError::new(&format!(
                "Selected index {} out of bounds for values",
                index
            )));
        }

        Ok(values[index])
    }

    /// Get value at specific index
    fn get_value_at_index(&self, value_list: usize, index: usize) -> Result<i32> {
        if value_list >= self.values_lists.len() {
            return Err(TpcdsError::new(&format!(
                "Value list index {} out of bounds",
                value_list
            )));
        }

        let values = &self.values_lists[value_list];
        if index >= values.len() {
            return Err(TpcdsError::new(&format!(
                "Index {} out of bounds for values",
                index
            )));
        }

        Ok(values[index])
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
        let data = &[(1, 30), (2, 20), (3, 50)];

        let dist = IntValuesDistribution::from_embedded_data(data).unwrap();
        assert_eq!(dist.get_value_lists_count(), 1);
        assert_eq!(dist.get_weight_lists_count(), 1);
        assert_eq!(dist.get_value_count(0), 3);

        // Test value access
        assert_eq!(dist.get_value_at_index(0, 0).unwrap(), 1);
        assert_eq!(dist.get_value_at_index(0, 1).unwrap(), 2);
        assert_eq!(dist.get_value_at_index(0, 2).unwrap(), 3);
    }

    #[test]
    fn test_uniform_distribution() {
        let values = &[10, 20, 30, 40, 50];
        let dist = IntValuesDistribution::uniform(values).unwrap();

        assert_eq!(dist.get_value_lists_count(), 1);
        assert_eq!(dist.get_value_count(0), 5);

        let mut stream = RandomNumberStream::new(1).unwrap();

        // Test multiple picks - all should be valid values
        for _ in 0..10 {
            let value = dist.pick_random_value(0, 0, &mut stream).unwrap();
            assert!(values.contains(&value));
        }
    }

    #[test]
    fn test_from_multi_weight_data() {
        let weights1 = [0, 1000];
        let weights2 = [500, 500];
        let weights3 = [1000, 0];
        let data = &[
            (0, weights1.as_slice()), // 0 - never picked from first, always from second
            (1, weights2.as_slice()), // 1 - balanced in both
            (2, weights3.as_slice()), // 2 - always picked from first, never from second
        ];

        let dist = IntValuesDistribution::from_multi_weight_data(data).unwrap();
        assert_eq!(dist.get_value_lists_count(), 1);
        assert_eq!(dist.get_weight_lists_count(), 2); // Two weight columns
        assert_eq!(dist.get_value_count(0), 3);
    }

    #[test]
    fn test_pick_random_value() {
        let data = &[
            (100, 1),  // Rare value
            (200, 99), // Common value
        ];

        let dist = IntValuesDistribution::from_embedded_data(data).unwrap();
        let mut stream = RandomNumberStream::new(1).unwrap();

        // Test multiple picks - should all be valid
        for _ in 0..10 {
            let value = dist.pick_random_value(0, 0, &mut stream).unwrap();
            assert!(value == 100 || value == 200);
        }
    }

    #[test]
    fn test_deterministic_behavior() {
        let data = &[(1, 25), (2, 25), (3, 25), (4, 25)];

        let dist = IntValuesDistribution::from_embedded_data(data).unwrap();

        // Same seed should produce same results
        let mut stream1 = RandomNumberStream::new_with_column(42, 1).unwrap();
        let mut stream2 = RandomNumberStream::new_with_column(42, 1).unwrap();

        let value1 = dist.pick_random_value(0, 0, &mut stream1).unwrap();
        let value2 = dist.pick_random_value(0, 0, &mut stream2).unwrap();

        assert_eq!(value1, value2);
    }

    #[test]
    fn test_weighted_distribution_bias() {
        // Create heavily biased distribution
        let data = &[
            (1, 1),    // Very rare
            (2, 1000), // Very common
        ];

        let dist = IntValuesDistribution::from_embedded_data(data).unwrap();
        let mut stream = RandomNumberStream::new(1).unwrap();

        // Test many picks - should heavily favor value 2
        let mut count_1 = 0;
        let mut count_2 = 0;

        for _ in 0..100 {
            match dist.pick_random_value(0, 0, &mut stream).unwrap() {
                1 => count_1 += 1,
                2 => count_2 += 1,
                _ => panic!("Unexpected value"),
            }
        }

        // Should heavily favor 2 (this is probabilistic but very likely)
        assert!(count_2 > count_1);
    }

    #[test]
    fn test_error_conditions() {
        let data = &[(42, 100)];
        let dist = IntValuesDistribution::from_embedded_data(data).unwrap();
        let mut stream = RandomNumberStream::new(1).unwrap();

        // Invalid list indices
        assert!(dist.pick_random_value(1, 0, &mut stream).is_err()); // Invalid value list
        assert!(dist.pick_random_value(0, 1, &mut stream).is_err()); // Invalid weight list
        assert!(dist.get_value_at_index(1, 0).is_err()); // Invalid value list
        assert!(dist.get_value_at_index(0, 1).is_err()); // Invalid index

        // Empty distribution
        let empty_dist = IntValuesDistribution::new(vec![], vec![]).unwrap();
        assert!(empty_dist.pick_random_value(0, 0, &mut stream).is_err());
    }

    #[test]
    fn test_validation() {
        // Mismatched list counts
        assert!(IntValuesDistribution::new(
            vec![vec![1]],
            vec![] // Empty weights but non-empty values
        )
        .is_err());

        // Mismatched value/weight lengths
        assert!(IntValuesDistribution::new(
            vec![vec![1, 2]], // 2 values
            vec![vec![100]]   // 1 weight
        )
        .is_err());
    }
}
