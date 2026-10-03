use crate::distribution::FileBasedStringValuesDistribution;
use std::sync::OnceLock;

/// Ship mode distributions (ShipModeDistributions)
pub struct ShipModeDistributions;

impl ShipModeDistributions {
    fn get_ship_mode_carrier_distribution() -> &'static FileBasedStringValuesDistribution {
        static DISTRIBUTION: OnceLock<FileBasedStringValuesDistribution> = OnceLock::new();
        DISTRIBUTION.get_or_init(|| {
            FileBasedStringValuesDistribution::build_string_values_distribution(
                "ship_mode_carrier.dst",
                1,
                1,
            )
            .expect("Failed to load ship_mode_carrier.dst")
        })
    }

    fn get_ship_mode_code_distribution() -> &'static FileBasedStringValuesDistribution {
        static DISTRIBUTION: OnceLock<FileBasedStringValuesDistribution> = OnceLock::new();
        DISTRIBUTION.get_or_init(|| {
            FileBasedStringValuesDistribution::build_string_values_distribution(
                "ship_mode_code.dst",
                1,
                1,
            )
            .expect("Failed to load ship_mode_code.dst")
        })
    }

    fn get_ship_mode_type_distribution() -> &'static FileBasedStringValuesDistribution {
        static DISTRIBUTION: OnceLock<FileBasedStringValuesDistribution> = OnceLock::new();
        DISTRIBUTION.get_or_init(|| {
            FileBasedStringValuesDistribution::build_string_values_distribution(
                "ship_mode_type.dst",
                1,
                1,
            )
            .expect("Failed to load ship_mode_type.dst")
        })
    }

    pub fn get_ship_mode_carrier_at_index(index: usize) -> &'static str {
        Self::get_ship_mode_carrier_distribution().get_value_at_index(0, index)
    }

    pub fn get_ship_mode_code_for_index_mod_size(index: i64) -> &'static str {
        Self::get_ship_mode_code_distribution().get_value_for_index_mod_size(index, 0)
    }

    pub fn get_ship_mode_type_for_index_mod_size(index: i64) -> &'static str {
        Self::get_ship_mode_type_distribution().get_value_for_index_mod_size(index, 0)
    }

    pub fn get_ship_mode_type_size() -> usize {
        Self::get_ship_mode_type_distribution().get_size()
    }
}
