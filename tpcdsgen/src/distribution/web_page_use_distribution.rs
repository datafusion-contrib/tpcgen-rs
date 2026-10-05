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

//! Web page use distribution for web_page table generation.
//!
//! This module provides distribution of web page types (general, order, welcome, etc.)
//! with uniform weighted random selection.

use crate::distribution::file_loader::DistributionFileLoader;
use crate::distribution::utils::{pick_random_value, WeightsBuilder};
use crate::error::Result;
use crate::random::RandomNumberStream;
use crate::TpcdsError;
use std::sync::OnceLock;

/// Web page use distribution (WebPageUseDistribution)
///
/// Loads web_page_use.dst which contains:
/// - 1 value field: page use type (general, order, welcome, ad, feedback, protected, dynamic)
/// - 1 weight field: uniform weights
pub struct WebPageUseDistribution {
    values: Vec<String>,    // Page use types
    weights_list: Vec<i32>, // Uniform weights
}

impl WebPageUseDistribution {
    const NUM_VALUE_FIELDS: usize = 1;
    const NUM_WEIGHT_FIELDS: usize = 1;
    const VALUES_AND_WEIGHTS_FILENAME: &'static str = "web_page_use.dst";

    fn get_instance() -> &'static WebPageUseDistribution {
        static DISTRIBUTION: OnceLock<WebPageUseDistribution> = OnceLock::new();
        DISTRIBUTION.get_or_init(|| {
            Self::build_web_page_use_distribution()
                .expect("Failed to load web page use distribution")
        })
    }

    fn build_web_page_use_distribution() -> Result<Self> {
        let mut values = Vec::new();
        let mut weights_builder = WeightsBuilder::new();

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
                    "Expected line to contain {} weight field, but it contained {}: {:?}",
                    Self::NUM_WEIGHT_FIELDS,
                    weight_fields.len(),
                    weight_fields
                )));
            }

            // Parse value (page use type)
            values.push(value_fields[0].trim().to_string());

            // Parse weight
            let weight: i32 = weight_fields[0].parse().map_err(|e| {
                TpcdsError::new(&format!(
                    "Failed to parse weight '{}': {}",
                    weight_fields[0], e
                ))
            })?;
            weights_builder.compute_and_add_next_weight(weight)?;
        }

        Ok(WebPageUseDistribution {
            values,
            weights_list: weights_builder.build(),
        })
    }

    /// Pick a random web page use type.
    ///
    /// This corresponds to WebPageUseDistribution.pickRandomWebPageUseType()
    ///
    /// # Arguments
    ///
    /// * `stream` - Random number stream for generating random values
    ///
    /// # Returns
    ///
    /// A web page use type string (e.g., "general", "order", "welcome", "ad", "feedback", "protected", "dynamic")
    pub fn pick_random_web_page_use_type(stream: &mut RandomNumberStream) -> String {
        let dist = Self::get_instance();
        pick_random_value(&dist.values, &dist.weights_list, stream).clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::RandomNumberStream;

    #[test]
    fn test_web_page_use_distribution_loading() {
        let dist = WebPageUseDistribution::get_instance();

        // Should have 7 page use types: general, order, welcome, ad, feedback, protected, dynamic
        assert_eq!(dist.values.len(), 7, "Should have 7 web page use types");
        assert_eq!(dist.weights_list.len(), 7);
    }

    #[test]
    fn test_pick_random_web_page_use_type() {
        let mut stream = RandomNumberStream::new(1).unwrap();
        let page_use = WebPageUseDistribution::pick_random_web_page_use_type(&mut stream);

        // Should be a non-empty string
        assert!(!page_use.is_empty(), "Page use type should not be empty");

        // Should be one of the expected types
        let valid_types = [
            "general",
            "order",
            "welcome",
            "ad",
            "feedback",
            "protected",
            "dynamic",
        ];
        assert!(
            valid_types.contains(&page_use.as_str()),
            "Page use '{}' should be one of: {:?}",
            page_use,
            valid_types
        );
    }

    #[test]
    fn test_pick_random_web_page_use_type_deterministic() {
        // Same seed should produce same result
        let mut stream1 = RandomNumberStream::new(42).unwrap();
        let mut stream2 = RandomNumberStream::new(42).unwrap();

        let page_use1 = WebPageUseDistribution::pick_random_web_page_use_type(&mut stream1);
        let page_use2 = WebPageUseDistribution::pick_random_web_page_use_type(&mut stream2);

        assert_eq!(
            page_use1, page_use2,
            "Same seed should produce same page use type"
        );
    }

    #[test]
    fn test_page_use_values() {
        let dist = WebPageUseDistribution::get_instance();

        // Verify expected page use types are present
        let types_set: std::collections::HashSet<&String> = dist.values.iter().collect();
        assert!(
            types_set.contains(&"general".to_string()),
            "Should contain 'general' type"
        );
        assert!(
            types_set.contains(&"order".to_string()),
            "Should contain 'order' type"
        );
    }

    #[test]
    fn test_multiple_picks_are_valid() {
        let mut stream = RandomNumberStream::new(1).unwrap();
        let valid_types = [
            "general",
            "order",
            "welcome",
            "ad",
            "feedback",
            "protected",
            "dynamic",
        ];

        // Pick multiple page use types and verify all are valid
        for _ in 0..20 {
            let page_use = WebPageUseDistribution::pick_random_web_page_use_type(&mut stream);
            assert!(
                valid_types.contains(&page_use.as_str()),
                "Page use '{}' should be valid",
                page_use
            );
        }
    }
}
