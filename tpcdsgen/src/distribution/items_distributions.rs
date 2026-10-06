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

//! Item-related distributions for TPC-DS data generation

use crate::distribution::file_loader::DistributionFileLoader;
use crate::distribution::int_values::IntValuesDistribution;
use crate::distribution::string_values_distribution::StringValuesDistribution;
use crate::distribution::utils::{Distribution, WeightsBuilder};
use crate::random::RandomNumberStream;
use crate::types::Decimal;
use std::sync::OnceLock;

// ============================================================================
// ItemsDistributions - sizes, colors, units, brand syllables, manager/manufact IDs
// ============================================================================

/// Item manager ID distribution
static ITEM_MANAGER_ID_DISTRIBUTION: OnceLock<IntValuesDistribution> = OnceLock::new();

/// Item manufact ID distribution
static ITEM_MANUFACT_ID_DISTRIBUTION: OnceLock<IntValuesDistribution> = OnceLock::new();

/// Sizes distribution
static SIZES_DISTRIBUTION: OnceLock<StringValuesDistribution> = OnceLock::new();

/// Colors distribution
static COLORS_DISTRIBUTION: OnceLock<StringValuesDistribution> = OnceLock::new();

/// Units distribution
static UNITS_DISTRIBUTION: OnceLock<StringValuesDistribution> = OnceLock::new();

/// Brand syllables distribution
static BRAND_SYLLABLES_DISTRIBUTION: OnceLock<StringValuesDistribution> = OnceLock::new();

/// Weight enums for ID ranges (manager and manufact)
#[derive(Debug, Clone, Copy)]
pub enum IdWeights {
    Unified = 0,
    Low = 1,
    Medium = 2,
    High = 3,
}

/// Weight enums for sizes
#[derive(Debug, Clone, Copy)]
pub enum SizeWeights {
    Unified = 0,
    NoSize = 1,
    Sized = 2,
}

/// Weight enums for colors
#[derive(Debug, Clone, Copy)]
pub enum ColorsWeights {
    Uniform = 0,
    Skewed = 1,
    LowLikelihood = 2,
    MediumLikelihood = 3,
    HighLikelihood = 4,
}

fn get_item_manager_id_distribution() -> &'static IntValuesDistribution {
    ITEM_MANAGER_ID_DISTRIBUTION.get_or_init(|| {
        IntValuesDistribution::build_int_values_distribution("item_manager_id.dst", 3, 4)
            .expect("Failed to load item_manager_id.dst")
    })
}

fn get_item_manufact_id_distribution() -> &'static IntValuesDistribution {
    ITEM_MANUFACT_ID_DISTRIBUTION.get_or_init(|| {
        IntValuesDistribution::build_int_values_distribution("item_manufact_id.dst", 3, 4)
            .expect("Failed to load item_manufact_id.dst")
    })
}

fn get_sizes_distribution() -> &'static StringValuesDistribution {
    SIZES_DISTRIBUTION.get_or_init(|| {
        StringValuesDistribution::build_string_values_distribution("sizes.dst", 1, 3)
            .expect("Failed to load sizes.dst")
    })
}

fn get_colors_distribution() -> &'static StringValuesDistribution {
    COLORS_DISTRIBUTION.get_or_init(|| {
        StringValuesDistribution::build_string_values_distribution("colors.dst", 1, 5)
            .expect("Failed to load colors.dst")
    })
}

fn get_units_distribution() -> &'static StringValuesDistribution {
    UNITS_DISTRIBUTION.get_or_init(|| {
        StringValuesDistribution::build_string_values_distribution("units.dst", 1, 1)
            .expect("Failed to load units.dst")
    })
}

/// Get brand syllables distribution for word generation
pub fn get_brand_syllables_distribution() -> &'static StringValuesDistribution {
    BRAND_SYLLABLES_DISTRIBUTION.get_or_init(|| {
        StringValuesDistribution::build_string_values_distribution("brand_syllables.dst", 1, 1)
            .expect("Failed to load brand_syllables.dst")
    })
}

/// Pick random manager ID range
pub fn pick_random_manager_id_range(
    id_weights: IdWeights,
    stream: &mut RandomNumberStream,
) -> (i32, i32) {
    let dist = get_item_manager_id_distribution();
    let index = dist.pick_random_index(id_weights as usize, stream);
    (
        dist.get_value_at_index(1, index),
        dist.get_value_at_index(2, index),
    )
}

/// Pick random manufact ID range
pub fn pick_random_manufact_id_range(
    id_weights: IdWeights,
    stream: &mut RandomNumberStream,
) -> (i32, i32) {
    let dist = get_item_manufact_id_distribution();
    let index = dist.pick_random_index(id_weights as usize, stream);
    (
        dist.get_value_at_index(1, index),
        dist.get_value_at_index(2, index),
    )
}

/// Pick random size
pub fn pick_random_size(size_weights: SizeWeights, stream: &mut RandomNumberStream) -> String {
    get_sizes_distribution()
        .pick_random_value(0, size_weights as usize, stream)
        .to_string()
}

/// Pick random color
pub fn pick_random_color(colors_weights: ColorsWeights, stream: &mut RandomNumberStream) -> String {
    get_colors_distribution()
        .pick_random_value(0, colors_weights as usize, stream)
        .to_string()
}

/// Pick random unit
pub fn pick_random_unit(stream: &mut RandomNumberStream) -> String {
    get_units_distribution()
        .pick_random_value(0, 0, stream)
        .to_string()
}

// ============================================================================
// CategoriesDistribution - categories with hasSize flag
// ============================================================================

struct CategoriesDistributionData {
    names: Vec<String>,
    has_sizes: Vec<i32>,
    weights: Vec<i32>,
}

static CATEGORIES_DISTRIBUTION: OnceLock<CategoriesDistributionData> = OnceLock::new();

fn get_categories_distribution() -> &'static CategoriesDistributionData {
    CATEGORIES_DISTRIBUTION.get_or_init(|| {
        let parsed_lines = DistributionFileLoader::load_distribution_file("categories.dst")
            .expect("Failed to load categories.dst");

        let mut names = Vec::new();
        let mut has_sizes = Vec::new();
        let mut weights_builder = WeightsBuilder::new();

        for (values, weights) in parsed_lines {
            if values.len() != 3 {
                panic!(
                    "Expected line to contain 3 values, but it contained {}: {:?}",
                    values.len(),
                    values
                );
            }
            if weights.len() != 1 {
                panic!(
                    "Expected line to contain 1 weight, but it contained {}: {:?}",
                    weights.len(),
                    weights
                );
            }

            names.push(values[0].clone());
            // values[1] is the class distribution name (unused)
            has_sizes.push(values[2].parse().expect("Failed to parse hasSize"));

            let weight: i32 = weights[0].parse().expect("Failed to parse weight");
            weights_builder
                .compute_and_add_next_weight(weight)
                .expect("Failed to compute weight");
        }

        CategoriesDistributionData {
            names,
            has_sizes,
            weights: weights_builder.build(),
        }
    })
}

/// Pick random category index
pub fn pick_random_category_index(stream: &mut RandomNumberStream) -> usize {
    let dist = get_categories_distribution();
    crate::distribution::utils::pick_random_index(&dist.weights, stream)
}

/// Get category name at index
pub fn get_category_at_index(index: usize) -> &'static str {
    let dist = get_categories_distribution();
    &dist.names[index]
}

/// Get hasSize flag at index
pub fn get_has_size_at_index(index: usize) -> i32 {
    let dist = get_categories_distribution();
    dist.has_sizes[index]
}

// ============================================================================
// CategoryClassDistributions - class distributions per category
// ============================================================================

#[derive(Debug, Clone)]
pub struct CategoryClass {
    id: i64,
    name: String,
    brand_count: i32,
}

impl CategoryClass {
    pub fn get_id(&self) -> i64 {
        self.id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_brand_count(&self) -> i32 {
        self.brand_count
    }
}

struct CategoryClassDistribution {
    names: Vec<String>,
    brand_counts: Vec<i32>,
    weights: Vec<i32>,
}

impl CategoryClassDistribution {
    fn build(filename: &str) -> Self {
        let parsed_lines = DistributionFileLoader::load_distribution_file(filename)
            .unwrap_or_else(|e| panic!("Failed to load {}: {}", filename, e));

        let mut names = Vec::new();
        let mut brand_counts = Vec::new();
        let mut weights_builder = WeightsBuilder::new();

        for (values, weights) in parsed_lines {
            if values.len() != 2 {
                panic!(
                    "Expected line to contain 2 values, but it contained {}: {:?}",
                    values.len(),
                    values
                );
            }
            if weights.len() != 1 {
                panic!(
                    "Expected line to contain 1 weight, but it contained {}: {:?}",
                    weights.len(),
                    weights
                );
            }

            names.push(values[0].clone());
            brand_counts.push(values[1].parse().expect("Failed to parse brand_count"));

            let weight: i32 = weights[0].parse().expect("Failed to parse weight");
            weights_builder
                .compute_and_add_next_weight(weight)
                .expect("Failed to compute weight");
        }

        CategoryClassDistribution {
            names,
            brand_counts,
            weights: weights_builder.build(),
        }
    }

    fn pick_random_category_class(&self, stream: &mut RandomNumberStream) -> CategoryClass {
        let index = crate::distribution::utils::pick_random_index(&self.weights, stream);
        CategoryClass {
            id: (index + 1) as i64,
            name: self.names[index].clone(),
            brand_count: self.brand_counts[index],
        }
    }
}

static CATEGORY_CLASS_DISTRIBUTIONS: OnceLock<Vec<CategoryClassDistribution>> = OnceLock::new();

fn get_category_class_distributions() -> &'static Vec<CategoryClassDistribution> {
    CATEGORY_CLASS_DISTRIBUTIONS.get_or_init(|| {
        vec![
            CategoryClassDistribution::build("women_class.dst"),
            CategoryClassDistribution::build("men_class.dst"),
            CategoryClassDistribution::build("children_class.dst"),
            CategoryClassDistribution::build("shoe_class.dst"),
            CategoryClassDistribution::build("music_class.dst"),
            CategoryClassDistribution::build("jewelry_class.dst"),
            CategoryClassDistribution::build("home_class.dst"),
            CategoryClassDistribution::build("sport_class.dst"),
            CategoryClassDistribution::build("book_class.dst"),
            CategoryClassDistribution::build("electronic_class.dst"),
        ]
    })
}

/// Pick random category class for a given category ID
pub fn pick_random_category_class(
    category_id: usize,
    stream: &mut RandomNumberStream,
) -> CategoryClass {
    get_category_class_distributions()[category_id].pick_random_category_class(stream)
}

// ============================================================================
// ItemCurrentPriceDistribution - price ranges
// ============================================================================

struct ItemCurrentPriceDistributionData {
    mins: Vec<Decimal>,
    maxes: Vec<Decimal>,
    weights_lists: Vec<Vec<i32>>,
}

static ITEM_CURRENT_PRICE_DISTRIBUTION: OnceLock<ItemCurrentPriceDistributionData> =
    OnceLock::new();

fn get_item_current_price_distribution() -> &'static ItemCurrentPriceDistributionData {
    ITEM_CURRENT_PRICE_DISTRIBUTION.get_or_init(|| {
        let parsed_lines = DistributionFileLoader::load_distribution_file("item_current_price.dst")
            .expect("Failed to load item_current_price.dst");

        let num_weight_fields = 4;
        let mut mins = Vec::new();
        let mut maxes = Vec::new();
        let mut weights_builders: Vec<WeightsBuilder> = (0..num_weight_fields)
            .map(|_| WeightsBuilder::new())
            .collect();

        for (values, weights) in parsed_lines {
            if values.len() != 3 {
                panic!(
                    "Expected line to contain 3 values, but it contained {}: {:?}",
                    values.len(),
                    values
                );
            }
            if weights.len() != num_weight_fields {
                panic!(
                    "Expected line to contain {} weights, but it contained {}: {:?}",
                    num_weight_fields,
                    weights.len(),
                    weights
                );
            }

            // values[0] is index (unused)
            mins.push(Decimal::parse_decimal(&values[1]).expect("Failed to parse min decimal"));
            maxes.push(Decimal::parse_decimal(&values[2]).expect("Failed to parse max decimal"));

            for (i, weight_str) in weights.iter().enumerate() {
                let weight: i32 = weight_str.parse().expect("Failed to parse weight");
                weights_builders[i]
                    .compute_and_add_next_weight(weight)
                    .expect("Failed to compute weight");
            }
        }

        ItemCurrentPriceDistributionData {
            mins,
            maxes,
            weights_lists: weights_builders.into_iter().map(|b| b.build()).collect(),
        }
    })
}

/// Pick random current price range (returns min and max)
pub fn pick_random_current_price_range(stream: &mut RandomNumberStream) -> (Decimal, Decimal) {
    let dist = get_item_current_price_distribution();
    let index = crate::distribution::utils::pick_random_index(&dist.weights_lists[0], stream);
    (dist.mins[index], dist.maxes[index])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::RandomNumberStream;

    #[test]
    fn test_pick_random_size() {
        let mut stream = RandomNumberStream::new(1);
        let size = pick_random_size(SizeWeights::Sized, &mut stream);
        assert!(!size.is_empty());
    }

    #[test]
    fn test_pick_random_color() {
        let mut stream = RandomNumberStream::new(1);
        let color = pick_random_color(ColorsWeights::Skewed, &mut stream);
        assert!(!color.is_empty());
    }

    #[test]
    fn test_pick_random_unit() {
        let mut stream = RandomNumberStream::new(1);
        let unit = pick_random_unit(&mut stream);
        assert!(!unit.is_empty());
    }

    #[test]
    fn test_pick_random_category_index() {
        let mut stream = RandomNumberStream::new(1);
        let index = pick_random_category_index(&mut stream);
        assert!(index < 10); // Should be less than number of categories
    }

    #[test]
    fn test_get_category_at_index() {
        let category = get_category_at_index(0);
        assert!(!category.is_empty());
    }

    #[test]
    fn test_pick_random_category_class() {
        let mut stream = RandomNumberStream::new(1);
        let class = pick_random_category_class(0, &mut stream);
        assert!(!class.get_name().is_empty());
        assert!(class.get_brand_count() > 0);
    }

    #[test]
    fn test_pick_random_current_price_range() {
        let mut stream = RandomNumberStream::new(1);
        let (min, max) = pick_random_current_price_range(&mut stream);
        assert!(min.get_number() <= max.get_number());
    }

    #[test]
    fn test_pick_random_manager_id_range() {
        let mut stream = RandomNumberStream::new(1);
        let (min, max) = pick_random_manager_id_range(IdWeights::Unified, &mut stream);
        assert!(min <= max);
    }

    #[test]
    fn test_pick_random_manufact_id_range() {
        let mut stream = RandomNumberStream::new(1);
        let (min, max) = pick_random_manufact_id_range(IdWeights::Unified, &mut stream);
        assert!(min <= max);
    }

    #[test]
    fn test_brand_syllables_distribution() {
        let dist = get_brand_syllables_distribution();
        assert!(dist.get_size() > 0);
    }
}
