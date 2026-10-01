use crate::distribution::file_loader::DistributionFileLoader;
use crate::distribution::utils::{
    get_value_for_index_mod_size, get_weight_for_index, pick_random_index, pick_random_value,
    WeightsBuilder,
};
use crate::error::{Result, TpcdsError};
use crate::random::RandomNumberStream;

/// String values distribution that loads from .dst files
/// StringValuesDistribution functionality
#[derive(Debug, Clone)]
pub struct StringValuesDistribution {
    values_lists: Vec<Vec<String>>,
    weights_lists: Vec<Vec<i32>>,
}

impl StringValuesDistribution {
    /// Build a StringValuesDistribution from a distribution file
    ///
    /// # Arguments
    /// * `filename` - The .dst file to load
    /// * `num_value_fields` - Number of value fields per line
    /// * `num_weight_fields` - Number of weight fields per line
    pub fn build_string_values_distribution(
        filename: &str,
        num_value_fields: usize,
        num_weight_fields: usize,
    ) -> Result<Self> {
        let parsed_lines = DistributionFileLoader::load_distribution_file(filename)?;

        let mut values_builders: Vec<Vec<String>> = vec![Vec::new(); num_value_fields];
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

            // Add values to builders
            for (i, value) in values.into_iter().enumerate() {
                values_builders[i].push(value);
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

        Ok(StringValuesDistribution {
            values_lists,
            weights_lists,
        })
    }

    /// Pick a random value from the specified value list using the specified weight list
    pub fn pick_random_value(
        &self,
        value_list_index: usize,
        weight_list_index: usize,
        stream: &mut RandomNumberStream,
    ) -> Result<&str> {
        if value_list_index >= self.values_lists.len() {
            return Err(TpcdsError::new(&format!(
                "Value list index {} out of range, max is {}",
                value_list_index,
                self.values_lists.len() - 1
            )));
        }

        if weight_list_index >= self.weights_lists.len() {
            return Err(TpcdsError::new(&format!(
                "Weight list index {} out of range, max is {}",
                weight_list_index,
                self.weights_lists.len() - 1
            )));
        }

        let value = pick_random_value(
            &self.values_lists[value_list_index],
            &self.weights_lists[weight_list_index],
            stream,
        )?;

        Ok(value)
    }

    /// Get a value by index modulo the size of the list
    pub fn get_value_for_index_mod_size(
        &self,
        index: i64,
        value_list_index: usize,
    ) -> Result<&str> {
        if value_list_index >= self.values_lists.len() {
            return Err(TpcdsError::new(&format!(
                "Value list index {} out of range, max is {}",
                value_list_index,
                self.values_lists.len() - 1
            )));
        }

        let value = get_value_for_index_mod_size(index, &self.values_lists[value_list_index]);
        Ok(value)
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
                self.weights_lists.len() - 1
            )));
        }

        pick_random_index(&self.weights_lists[weight_list_index], stream)
    }

    /// Get the weight for a specific index
    pub fn get_weight_for_index(&self, index: usize, weight_list_index: usize) -> Result<i32> {
        if weight_list_index >= self.weights_lists.len() {
            return Err(TpcdsError::new(&format!(
                "Weight list index {} out of range, max is {}",
                weight_list_index,
                self.weights_lists.len() - 1
            )));
        }

        get_weight_for_index(index, &self.weights_lists[weight_list_index])
    }

    /// Get the size of the distribution (number of entries)
    pub fn get_size(&self) -> usize {
        if self.values_lists.is_empty() {
            0
        } else {
            self.values_lists[0].len()
        }
    }

    /// Get a specific value at a specific index
    pub fn get_value_at_index(&self, value_list_index: usize, value_index: usize) -> Result<&str> {
        if value_list_index >= self.values_lists.len() {
            return Err(TpcdsError::new(&format!(
                "Value list index {} out of range, max is {}",
                value_list_index,
                self.values_lists.len() - 1
            )));
        }

        if value_index >= self.values_lists[value_list_index].len() {
            return Err(TpcdsError::new(&format!(
                "Value index {} out of range, max is {}",
                value_index,
                self.values_lists[value_list_index].len() - 1
            )));
        }

        Ok(&self.values_lists[value_list_index][value_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::RandomNumberStream;

    #[test]
    fn test_build_call_centers_distribution() {
        let dist = StringValuesDistribution::build_string_values_distribution(
            "call_centers.dst",
            1, // 1 value field (name)
            2, // 2 weight fields (uniform, sales percentage)
        )
        .unwrap();

        assert!(dist.get_size() > 0);

        // Check we can get a value by index
        let first_center = dist.get_value_at_index(0, 0).unwrap();
        assert!(!first_center.is_empty());
    }

    #[test]
    fn test_pick_random_call_center() {
        let dist = StringValuesDistribution::build_string_values_distribution(
            "call_centers.dst",
            1, // 1 value field (name)
            2, // 2 weight fields (uniform, sales percentage)
        )
        .unwrap();

        let mut stream = RandomNumberStream::new(1).unwrap();

        // Pick using uniform weights (index 0)
        let center1 = dist.pick_random_value(0, 0, &mut stream).unwrap();
        assert!(!center1.is_empty());

        // Pick using sales percentage weights (index 1)
        let center2 = dist.pick_random_value(0, 1, &mut stream).unwrap();
        assert!(!center2.is_empty());
    }

    #[test]
    fn test_first_names_distribution() {
        let dist = StringValuesDistribution::build_string_values_distribution(
            "first_names.dst",
            1, // 1 value field (name)
            3, // 3 weight fields (male freq, female freq, general freq)
        )
        .unwrap();

        assert!(dist.get_size() > 100); // Should have many names

        let mut stream = RandomNumberStream::new(1).unwrap();

        // Pick using male frequency weights (index 0)
        let male_name = dist.pick_random_value(0, 0, &mut stream).unwrap();
        assert!(!male_name.is_empty());

        // Pick using female frequency weights (index 1)
        let female_name = dist.pick_random_value(0, 1, &mut stream).unwrap();
        assert!(!female_name.is_empty());

        // Pick using general frequency weights (index 2)
        let general_name = dist.pick_random_value(0, 2, &mut stream).unwrap();
        assert!(!general_name.is_empty());
    }

    #[test]
    fn test_deterministic_selection() {
        let dist =
            StringValuesDistribution::build_string_values_distribution("call_centers.dst", 1, 2)
                .unwrap();

        let mut stream1 = RandomNumberStream::new(1).unwrap();
        let mut stream2 = RandomNumberStream::new(1).unwrap();

        let result1 = dist.pick_random_value(0, 0, &mut stream1).unwrap();
        let result2 = dist.pick_random_value(0, 0, &mut stream2).unwrap();

        // Should be deterministic with same seed
        assert_eq!(result1, result2);
    }
}
