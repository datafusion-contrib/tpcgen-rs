use crate::distribution::string_values_distribution::StringValuesDistribution;
use crate::random::stream::RandomNumberStream;
use std::sync::OnceLock;

static STREET_NAMES_DISTRIBUTION: OnceLock<StringValuesDistribution> = OnceLock::new();
static STREET_TYPES_DISTRIBUTION: OnceLock<StringValuesDistribution> = OnceLock::new();
static CITIES_DISTRIBUTION: OnceLock<StringValuesDistribution> = OnceLock::new();
static COUNTRIES_DISTRIBUTION: OnceLock<StringValuesDistribution> = OnceLock::new();

#[derive(Debug, Clone, Copy)]
pub enum StreetNamesWeights {
    Default = 0,
    HalfEmpty = 1,
}

#[derive(Debug, Clone, Copy)]
pub enum CitiesWeights {
    UsgsSkewed = 0,
    Uniform = 1,
    Large = 2,
    Medium = 3,
    Small = 4,
    UnifiedStepFunction = 5,
}

pub fn pick_random_street_name(
    weights: StreetNamesWeights,
    stream: &mut RandomNumberStream,
) -> &'static str {
    let dist = STREET_NAMES_DISTRIBUTION.get_or_init(|| {
        StringValuesDistribution::build_string_values_distribution("street_names.dst", 1, 2)
            .expect("Failed to load street names distribution")
    });

    dist.pick_random_value(0, weights as usize, stream)
}

pub fn pick_random_street_type(stream: &mut RandomNumberStream) -> &'static str {
    let dist = STREET_TYPES_DISTRIBUTION.get_or_init(|| {
        StringValuesDistribution::build_string_values_distribution("street_types.dst", 1, 1)
            .expect("Failed to load street types distribution")
    });

    dist.pick_random_value(0, 0, stream)
}

pub fn pick_random_city(weights: CitiesWeights, stream: &mut RandomNumberStream) -> &'static str {
    let dist = CITIES_DISTRIBUTION.get_or_init(|| {
        StringValuesDistribution::build_string_values_distribution("cities.dst", 1, 6)
            .expect("Failed to load cities distribution")
    });

    dist.pick_random_value(0, weights as usize, stream)
}

pub fn pick_random_country(stream: &mut RandomNumberStream) -> &'static str {
    let dist = COUNTRIES_DISTRIBUTION.get_or_init(|| {
        StringValuesDistribution::build_string_values_distribution("countries.dst", 1, 1)
            .expect("Failed to load countries distribution")
    });

    dist.pick_random_value(0, 0, stream)
}

pub fn get_city_at_index(index: usize) -> &'static str {
    let dist = CITIES_DISTRIBUTION.get_or_init(|| {
        StringValuesDistribution::build_string_values_distribution("cities.dst", 1, 6)
            .expect("Failed to load cities distribution")
    });

    dist.get_value_at_index(0, index)
}
