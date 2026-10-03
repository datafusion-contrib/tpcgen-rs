use crate::distribution::file_loader::DistributionFileLoader;
use crate::distribution::utils::{pick_random_index, WeightsBuilder};
use crate::error::{Result, TpcdsError};
use crate::random::RandomNumberStream;
use std::sync::OnceLock;

static FIPS_COUNTY_DISTRIBUTION: OnceLock<FipsCountyDistribution> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct FipsCountyDistribution {
    counties: Vec<String>,
    state_abbreviations: Vec<String>,
    zip_prefixes: Vec<i32>,
    gmt_offsets: Vec<i32>,
    weights_lists: Vec<Vec<i32>>,
}

#[derive(Debug, Clone, Copy)]
pub enum FipsWeights {
    Uniform = 0,
    Population = 1,
    Timezone = 2,
    InZone1 = 3,
    InZone2 = 4,
    InZone3 = 5,
}

impl FipsCountyDistribution {
    const NUM_WEIGHT_FIELDS: usize = 6;

    fn build_fips_county_distribution() -> Result<Self> {
        let parsed_lines = DistributionFileLoader::load_distribution_file("fips.dst")?;

        let mut counties = Vec::new();
        let mut state_abbreviations = Vec::new();
        let mut zip_prefixes = Vec::new();
        let mut gmt_offsets = Vec::new();
        let mut weights_builders = vec![WeightsBuilder::new(); Self::NUM_WEIGHT_FIELDS];

        for (values, weights) in parsed_lines {
            if values.len() != 6 {
                return Err(TpcdsError::new(&format!(
                    "Expected line to contain 6 values, but it contained {}: {:?}",
                    values.len(),
                    values
                )));
            }

            if weights.len() != Self::NUM_WEIGHT_FIELDS {
                return Err(TpcdsError::new(&format!(
                    "Expected line to contain {} weights, but it contained {}: {:?}",
                    Self::NUM_WEIGHT_FIELDS,
                    weights.len(),
                    weights
                )));
            }

            // fips codes (values[0]) and state names (values[3]) are never used, so we leave them out
            counties.push(values[1].clone()); // County name
            state_abbreviations.push(values[2].clone()); // State abbreviation
            zip_prefixes.push(values[4].parse::<i32>().map_err(|e| {
                TpcdsError::new(&format!(
                    "Failed to parse zip prefix '{}': {}",
                    values[4], e
                ))
            })?);
            gmt_offsets.push(values[5].parse::<i32>().map_err(|e| {
                TpcdsError::new(&format!(
                    "Failed to parse GMT offset '{}': {}",
                    values[5], e
                ))
            })?);

            // Add weights to builders
            for (i, weight_str) in weights.iter().enumerate() {
                let weight = weight_str.parse::<i32>().map_err(|e| {
                    TpcdsError::new(&format!("Failed to parse weight '{}': {}", weight_str, e))
                })?;
                weights_builders[i].compute_and_add_next_weight(weight)?;
            }
        }

        let weights_lists = weights_builders
            .into_iter()
            .map(|builder| builder.build())
            .collect();

        Ok(FipsCountyDistribution {
            counties,
            state_abbreviations,
            zip_prefixes,
            gmt_offsets,
            weights_lists,
        })
    }

    fn get_instance() -> &'static FipsCountyDistribution {
        FIPS_COUNTY_DISTRIBUTION.get_or_init(|| {
            Self::build_fips_county_distribution()
                .expect("Failed to build FIPS county distribution")
        })
    }

    pub fn pick_random_index(weights: FipsWeights, stream: &mut RandomNumberStream) -> usize {
        let instance = Self::get_instance();
        pick_random_index(&instance.weights_lists[weights as usize], stream)
    }

    pub fn get_county_at_index(index: usize) -> &'static str {
        &Self::get_instance().counties[index]
    }

    pub fn get_state_abbreviation_at_index(index: usize) -> &'static str {
        &Self::get_instance().state_abbreviations[index]
    }

    pub fn get_zip_prefix_at_index(index: usize) -> i32 {
        Self::get_instance().zip_prefixes[index]
    }

    pub fn get_gmt_offset_at_index(index: usize) -> i32 {
        Self::get_instance().gmt_offsets[index]
    }
}
