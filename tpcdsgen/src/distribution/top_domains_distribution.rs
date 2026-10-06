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

//! Top domains distribution for web_site table generation.
//!
//! This module provides distribution of top-level domain suffixes (com, org, edu)
//! with uniform weighted random selection.

use crate::distribution::file_loader::DistributionFileLoader;
use crate::distribution::utils::{pick_random_value, WeightsBuilder};
use crate::error::Result;
use crate::random::RandomNumberStream;
use crate::TpcdsError;
use std::sync::OnceLock;

/// Top domains distribution (TopDomainsDistribution)
///
/// Loads top_domains.dst which contains:
/// - 1 value field: domain suffix (com, org, edu, etc.)
/// - 1 weight field: uniform weights
pub struct TopDomainsDistribution {
    values: Vec<String>,    // Domain suffixes
    weights_list: Vec<i32>, // Uniform weights
}

impl TopDomainsDistribution {
    const NUM_VALUE_FIELDS: usize = 1;
    const NUM_WEIGHT_FIELDS: usize = 1;
    const VALUES_AND_WEIGHTS_FILENAME: &'static str = "top_domains.dst";

    fn get_instance() -> &'static TopDomainsDistribution {
        static DISTRIBUTION: OnceLock<TopDomainsDistribution> = OnceLock::new();
        DISTRIBUTION.get_or_init(|| {
            Self::build_top_domains_distribution().expect("Failed to load top domains distribution")
        })
    }

    fn build_top_domains_distribution() -> Result<Self> {
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

            // Parse value (domain suffix)
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

        Ok(TopDomainsDistribution {
            values,
            weights_list: weights_builder.build(),
        })
    }

    /// Pick a random top-level domain suffix.
    ///
    /// This corresponds to TopDomainsDistribution.pickRandomTopDomain()
    ///
    /// # Arguments
    ///
    /// * `stream` - Random number stream for generating random values
    ///
    /// # Returns
    ///
    /// A domain suffix string (e.g., "com", "org", "edu")
    pub fn pick_random_top_domain(stream: &mut RandomNumberStream) -> String {
        let dist = Self::get_instance();
        pick_random_value(&dist.values, &dist.weights_list, stream).clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::RandomNumberStream;

    #[test]
    fn test_top_domains_distribution_loading() {
        let dist = TopDomainsDistribution::get_instance();

        // Should have at least 3 domain types: com, org, edu
        assert!(dist.values.len() >= 3, "Should have at least 3 top domains");
        assert_eq!(dist.weights_list.len(), dist.values.len());
    }

    #[test]
    fn test_pick_random_top_domain() {
        let mut stream = RandomNumberStream::new(1);
        let domain = TopDomainsDistribution::pick_random_top_domain(&mut stream);

        // Should be a non-empty string
        assert!(!domain.is_empty(), "Domain should not be empty");

        // Typical domains are short
        assert!(
            domain.len() <= 10,
            "Domain suffix '{}' should be reasonably short",
            domain
        );
    }

    #[test]
    fn test_pick_random_top_domain_deterministic() {
        // Same seed should produce same result
        let mut stream1 = RandomNumberStream::new(42);
        let mut stream2 = RandomNumberStream::new(42);

        let domain1 = TopDomainsDistribution::pick_random_top_domain(&mut stream1);
        let domain2 = TopDomainsDistribution::pick_random_top_domain(&mut stream2);

        assert_eq!(domain1, domain2, "Same seed should produce same domain");
    }

    #[test]
    fn test_domain_values() {
        let dist = TopDomainsDistribution::get_instance();

        // Verify expected domains are present
        let domains_set: std::collections::HashSet<&String> = dist.values.iter().collect();
        assert!(
            domains_set.contains(&"com".to_string()),
            "Should contain 'com' domain"
        );
    }

    #[test]
    fn test_multiple_picks_are_valid() {
        let mut stream = RandomNumberStream::new(1);

        // Pick multiple domains and verify all are valid
        for _ in 0..10 {
            let domain = TopDomainsDistribution::pick_random_top_domain(&mut stream);
            assert!(!domain.is_empty(), "All picked domains should be non-empty");
        }
    }
}
