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

//! Catalog page distributions for TPC-DS catalog_page table generation.
//!
//! This module provides distribution of catalog types (monthly, bi-annual, quarterly)
//! with weighted random selection based on distribution frequency and sales volume.

use crate::distribution::file_loader::DistributionFileLoader;
use crate::distribution::utils::{pick_random_value, WeightsBuilder};
use crate::error::Result;
use crate::random::RandomNumberStream;
use crate::TpcdsError;
use std::sync::OnceLock;

/// Catalog page type distribution (CatalogPageDistributions)
///
/// Loads catalog_page_types.dst which contains:
/// - 1 value field: catalog type name (monthly, bi-annual, quarterly)
/// - 2 weight fields: distribution frequency, sales volume
///
/// Only the second weight field (sales volume) is used for random picking.
pub struct CatalogPageTypesDistribution {
    values: Vec<String>,      // Catalog type names
    _weights_list1: Vec<i32>, // Distribution frequency weights (not used)
    weights_list2: Vec<i32>,  // Sales volume weights (used for picking)
}

impl CatalogPageTypesDistribution {
    const NUM_VALUE_FIELDS: usize = 1;
    const NUM_WEIGHT_FIELDS: usize = 2;
    const VALUES_AND_WEIGHTS_FILENAME: &'static str = "catalog_page_types.dst";

    fn get_instance() -> &'static CatalogPageTypesDistribution {
        static DISTRIBUTION: OnceLock<CatalogPageTypesDistribution> = OnceLock::new();
        DISTRIBUTION.get_or_init(|| {
            Self::build_catalog_page_types_distribution()
                .expect("Failed to load catalog page types distribution")
        })
    }

    fn build_catalog_page_types_distribution() -> Result<Self> {
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

            // Parse value (catalog type name)
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

        Ok(CatalogPageTypesDistribution {
            values,
            _weights_list1: weights_builder1.build(),
            weights_list2: weights_builder2.build(),
        })
    }

    /// Pick a random catalog page type using sales volume weights.
    ///
    /// This corresponds to CatalogPageDistributions.pickRandomCatalogPageType()
    /// which uses only the second set of weights (sales volume).
    ///
    /// # Arguments
    ///
    /// * `stream` - Random number stream for generating random values
    ///
    /// # Returns
    ///
    /// A catalog type string ("monthly", "bi-annual", or "quarterly")
    pub fn pick_random_catalog_page_type(stream: &mut RandomNumberStream) -> String {
        let dist = Self::get_instance();

        // Use the second weight list (sales volume, index 1)
        pick_random_value(&dist.values, &dist.weights_list2, stream).clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::RandomNumberStream;

    #[test]
    fn test_catalog_page_types_distribution_loading() {
        let dist = CatalogPageTypesDistribution::get_instance();

        // Should have 3 catalog types: monthly, bi-annual, quarterly
        assert_eq!(dist.values.len(), 3, "Should have 3 catalog types");
        assert_eq!(dist._weights_list1.len(), 3);
        assert_eq!(dist.weights_list2.len(), 3);
    }

    #[test]
    fn test_pick_random_catalog_page_type() {
        let mut stream = RandomNumberStream::new(1).unwrap();
        let catalog_type = CatalogPageTypesDistribution::pick_random_catalog_page_type(&mut stream);

        // Should be one of the valid types
        assert!(
            catalog_type == "monthly" || catalog_type == "bi-annual" || catalog_type == "quarterly",
            "Catalog type '{}' should be one of: monthly, bi-annual, quarterly",
            catalog_type
        );
    }

    #[test]
    fn test_pick_random_catalog_page_type_deterministic() {
        // Same seed should produce same result
        let mut stream1 = RandomNumberStream::new(42).unwrap();
        let mut stream2 = RandomNumberStream::new(42).unwrap();

        let type1 = CatalogPageTypesDistribution::pick_random_catalog_page_type(&mut stream1);
        let type2 = CatalogPageTypesDistribution::pick_random_catalog_page_type(&mut stream2);

        assert_eq!(type1, type2, "Same seed should produce same catalog type");
    }

    #[test]
    fn test_catalog_type_values() {
        let dist = CatalogPageTypesDistribution::get_instance();

        // Verify expected catalog types are present
        let types_set: std::collections::HashSet<&String> = dist.values.iter().collect();
        assert!(types_set.contains(&"monthly".to_string()));
        assert!(
            types_set.contains(&"bi-annual".to_string())
                || types_set.contains(&"quarterly".to_string())
        );
    }
}
