/*
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! Location types distribution for customer_address table generation.
//!
//! This module provides distribution of location types (single family, condo, apartment)
//! with weighted random selection.

use crate::distribution::file_loader::DistributionFileLoader;
use crate::distribution::utils::{pick_random_value, WeightsBuilder};
use crate::error::Result;
use crate::random::RandomNumberStream;
use crate::TpcdsError;
use std::sync::OnceLock;

/// Weights for location types distribution (LocationTypesDistribution.LocationTypeWeights)
#[derive(Debug, Clone, Copy)]
pub enum LocationTypeWeights {
    Uniform = 0,
    DistributionFrequency = 1,
}

/// Location types distribution (LocationTypesDistribution)
///
/// Loads location_types.dst which contains:
/// - 1 value field: location type name (single family, condo, apartment)
/// - 2 weight fields: uniform, distribution frequency
pub struct LocationTypesDistribution {
    values: Vec<String>,     // Location type names
    weights_list1: Vec<i32>, // Uniform weights
    weights_list2: Vec<i32>, // Distribution frequency weights
}

impl LocationTypesDistribution {
    const NUM_VALUE_FIELDS: usize = 1;
    const NUM_WEIGHT_FIELDS: usize = 2;
    const VALUES_AND_WEIGHTS_FILENAME: &'static str = "location_types.dst";

    fn get_instance() -> &'static LocationTypesDistribution {
        static DISTRIBUTION: OnceLock<LocationTypesDistribution> = OnceLock::new();
        DISTRIBUTION.get_or_init(|| {
            Self::build_location_types_distribution()
                .expect("Failed to load location types distribution")
        })
    }

    fn build_location_types_distribution() -> Result<Self> {
        let mut values = Vec::new();
        let mut weights_builder1 = WeightsBuilder::new();
        let mut weights_builder2 = WeightsBuilder::new();

        let parsed_lines =
            DistributionFileLoader::load_distribution_file(Self::VALUES_AND_WEIGHTS_FILENAME)?;

        for (value_fields, weight_fields) in parsed_lines {
            if value_fields.len() != Self::NUM_VALUE_FIELDS {
                return Err(TpcdsError::new(&format!(
                    "Expected line to contain {} value field, but it contained {}: {:?}",
                    Self::NUM_VALUE_FIELDS,
                    value_fields.len(),
                    value_fields
                )));
            }

            if weight_fields.len() != Self::NUM_WEIGHT_FIELDS {
                return Err(TpcdsError::new(&format!(
                    "Expected line to contain {} weight fields, but it contained {}: {:?}",
                    Self::NUM_WEIGHT_FIELDS,
                    weight_fields.len(),
                    weight_fields
                )));
            }

            // Parse value (location type name)
            values.push(value_fields[0].trim().to_string());

            // Parse weights
            let weight1: i32 = weight_fields[0].parse().map_err(|e| {
                TpcdsError::new(&format!(
                    "Failed to parse weight1 '{}': {}",
                    weight_fields[0], e
                ))
            })?;
            weights_builder1.compute_and_add_next_weight(weight1)?;

            let weight2: i32 = weight_fields[1].parse().map_err(|e| {
                TpcdsError::new(&format!(
                    "Failed to parse weight2 '{}': {}",
                    weight_fields[1], e
                ))
            })?;
            weights_builder2.compute_and_add_next_weight(weight2)?;
        }

        Ok(LocationTypesDistribution {
            values,
            weights_list1: weights_builder1.build(),
            weights_list2: weights_builder2.build(),
        })
    }

    /// Pick a random location type using specified weights.
    ///
    /// This corresponds to LocationTypesDistribution.pickRandomLocationType()
    ///
    /// # Arguments
    ///
    /// * `weights` - Which weight distribution to use (Uniform or DistributionFrequency)
    /// * `stream` - Random number stream for generating random values
    ///
    /// # Returns
    ///
    /// A location type string ("single family", "condo", or "apartment")
    pub fn pick_random_location_type(
        weights: LocationTypeWeights,
        stream: &mut RandomNumberStream,
    ) -> Result<String> {
        let dist = Self::get_instance();

        let weights_list = match weights {
            LocationTypeWeights::Uniform => &dist.weights_list1,
            LocationTypeWeights::DistributionFrequency => &dist.weights_list2,
        };

        let value_ref = pick_random_value(&dist.values, weights_list, stream)?;
        Ok(value_ref.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::RandomNumberStream;

    #[test]
    fn test_location_types_distribution_loading() {
        let dist = LocationTypesDistribution::get_instance();

        // Should have 3 location types: single family, condo, apartment
        assert_eq!(dist.values.len(), 3, "Should have 3 location types");
        assert_eq!(dist.weights_list1.len(), 3);
        assert_eq!(dist.weights_list2.len(), 3);
    }

    #[test]
    fn test_pick_random_location_type() {
        let mut stream = RandomNumberStream::new(1);
        let location_type = LocationTypesDistribution::pick_random_location_type(
            LocationTypeWeights::DistributionFrequency,
            &mut stream,
        )
        .unwrap();

        // Should be one of the valid types
        assert!(
            location_type == "single family"
                || location_type == "condo"
                || location_type == "apartment",
            "Location type '{}' should be one of: single family, condo, apartment",
            location_type
        );
    }

    #[test]
    fn test_pick_random_location_type_deterministic() {
        // Same seed should produce same result
        let mut stream1 = RandomNumberStream::new(42);
        let mut stream2 = RandomNumberStream::new(42);

        let type1 = LocationTypesDistribution::pick_random_location_type(
            LocationTypeWeights::Uniform,
            &mut stream1,
        )
        .unwrap();
        let type2 = LocationTypesDistribution::pick_random_location_type(
            LocationTypeWeights::Uniform,
            &mut stream2,
        )
        .unwrap();

        assert_eq!(type1, type2, "Same seed should produce same location type");
    }

    #[test]
    fn test_location_type_values() {
        let dist = LocationTypesDistribution::get_instance();

        // Verify expected location types are present
        let types_set: std::collections::HashSet<&String> = dist.values.iter().collect();
        assert!(types_set.contains(&"single family".to_string()));
        assert!(
            types_set.contains(&"condo".to_string())
                || types_set.contains(&"apartment".to_string())
        );
    }

    #[test]
    fn test_both_weight_types() {
        let mut stream = RandomNumberStream::new(1);

        // Both weight types should work
        let type_uniform = LocationTypesDistribution::pick_random_location_type(
            LocationTypeWeights::Uniform,
            &mut stream,
        )
        .unwrap();
        let type_dist = LocationTypesDistribution::pick_random_location_type(
            LocationTypeWeights::DistributionFrequency,
            &mut stream,
        )
        .unwrap();

        // Both should be valid
        assert!(
            type_uniform == "single family"
                || type_uniform == "condo"
                || type_uniform == "apartment"
        );
        assert!(type_dist == "single family" || type_dist == "condo" || type_dist == "apartment");
    }
}
