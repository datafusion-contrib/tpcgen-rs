use crate::config::CompatMode;
use crate::distribution::FileBasedStringValuesDistribution;
use std::sync::OnceLock;

/// Distribution for return reasons (ReturnReasons)
pub struct ReturnReasonsDistribution;

impl ReturnReasonsDistribution {
    fn get_distribution() -> &'static FileBasedStringValuesDistribution {
        static DISTRIBUTION: OnceLock<FileBasedStringValuesDistribution> = OnceLock::new();
        DISTRIBUTION.get_or_init(|| {
            FileBasedStringValuesDistribution::build_string_values_distribution(
                "return_reasons_trino.dst",
                1,
                6,
            )
            .expect("Failed to load return_reasons_trino.dst")
        })
    }

    /// C dsdgen uses a corrected .dst where `reason 30` is not a duplicate of `reason 31`[1].
    ///
    /// [1]: https://github.com/trinodb/tpcds/blob/master/src/main/resources/io/trino/tpcds/distribution/return_reasons.dst#L38
    fn get_distribution_c() -> &'static FileBasedStringValuesDistribution {
        static DISTRIBUTION_C: OnceLock<FileBasedStringValuesDistribution> = OnceLock::new();
        DISTRIBUTION_C.get_or_init(|| {
            FileBasedStringValuesDistribution::build_string_values_distribution(
                "return_reasons_c.dst",
                1,
                6,
            )
            .expect("Failed to load return_reasons_c.dst")
        })
    }

    pub fn get_return_reason_at_index(index: usize, compat: CompatMode) -> &'static str {
        match compat {
            CompatMode::C => Self::get_distribution_c().get_value_at_index(0, index),
            CompatMode::Trino => Self::get_distribution().get_value_at_index(0, index),
        }
    }

    pub fn get_size() -> usize {
        Self::get_distribution().get_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_return_reasons_distribution() {
        // Test that we can load the distribution
        let size = ReturnReasonsDistribution::get_size();
        assert!(
            size > 0,
            "Return reasons distribution should have at least one entry"
        );

        // Test that we can get values at valid indices
        for i in 0..size.min(5) {
            let value = ReturnReasonsDistribution::get_return_reason_at_index(i, CompatMode::Trino);
            assert!(!value.is_empty(), "Value at index {i} should not be empty");
        }
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_return_reasons_out_of_bounds() {
        let size = ReturnReasonsDistribution::get_size();
        ReturnReasonsDistribution::get_return_reason_at_index(size + 100, CompatMode::Trino);
    }
}
