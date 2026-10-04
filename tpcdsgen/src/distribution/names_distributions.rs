use crate::distribution::string_values_distribution::StringValuesDistribution;
use crate::error::Result;
use crate::random::RandomNumberStream;
use std::sync::OnceLock;

/// First names weight categories (FirstNamesWeights enum)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirstNamesWeights {
    MaleFrequency = 0,
    FemaleFrequency = 1,
    GeneralFrequency = 2,
}

/// Salutations weight categories (SalutationsWeights enum)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SalutationsWeights {
    GenderNeutral = 0,
    Male = 1,
    Female = 2,
}

/// Names distributions (NamesDistributions)
pub struct NamesDistributions;

static FIRST_NAMES_DISTRIBUTION: OnceLock<StringValuesDistribution> = OnceLock::new();
static LAST_NAMES_DISTRIBUTION: OnceLock<StringValuesDistribution> = OnceLock::new();
static SALUTATIONS_DISTRIBUTION: OnceLock<StringValuesDistribution> = OnceLock::new();

impl NamesDistributions {
    /// Initialize all distributions (lazy loading)
    fn ensure_initialized() -> Result<()> {
        // Initialize first names distribution
        if FIRST_NAMES_DISTRIBUTION.get().is_none() {
            let dist = StringValuesDistribution::build_string_values_distribution(
                "first_names.dst",
                1, // 1 value field: name
                3, // 3 weight fields: male freq, female freq, general freq
            )?;
            let _ = FIRST_NAMES_DISTRIBUTION.set(dist);
        }

        // Initialize last names distribution
        if LAST_NAMES_DISTRIBUTION.get().is_none() {
            let dist = StringValuesDistribution::build_string_values_distribution(
                "last_names.dst",
                1, // 1 value field: name
                1, // 1 weight field: frequency
            )?;
            let _ = LAST_NAMES_DISTRIBUTION.set(dist);
        }

        // Initialize salutations distribution
        if SALUTATIONS_DISTRIBUTION.get().is_none() {
            let dist = StringValuesDistribution::build_string_values_distribution(
                "salutations.dst",
                1, // 1 value field: salutation
                3, // 3 weight fields: gender neutral, male, female
            )?;
            let _ = SALUTATIONS_DISTRIBUTION.set(dist);
        }

        Ok(())
    }

    /// Pick a random first name using the specified weight category
    pub fn pick_random_first_name(
        weights: FirstNamesWeights,
        stream: &mut RandomNumberStream,
    ) -> Result<&'static str> {
        Self::ensure_initialized()?;
        let dist = FIRST_NAMES_DISTRIBUTION.get().unwrap();
        dist.pick_random_value(0, weights as usize, stream)
    }

    /// Pick a random index from first names using the specified weight category
    pub fn pick_random_index(
        weights: FirstNamesWeights,
        stream: &mut RandomNumberStream,
    ) -> Result<usize> {
        Self::ensure_initialized()?;
        let dist = FIRST_NAMES_DISTRIBUTION.get().unwrap();
        dist.pick_random_index(weights as usize, stream)
    }

    /// Get first name from specific index
    pub fn get_first_name_from_index(index: usize) -> Result<&'static str> {
        Self::ensure_initialized()?;
        let dist = FIRST_NAMES_DISTRIBUTION.get().unwrap();
        dist.get_value_at_index(0, index)
    }

    /// Get weight for specific index and weight category
    pub fn get_weight_for_index(index: usize, weights: FirstNamesWeights) -> Result<i32> {
        Self::ensure_initialized()?;
        let dist = FIRST_NAMES_DISTRIBUTION.get().unwrap();
        dist.get_weight_for_index(index, weights as usize)
    }

    /// Pick a random last name
    pub fn pick_random_last_name(stream: &mut RandomNumberStream) -> Result<&'static str> {
        Self::ensure_initialized()?;
        let dist = LAST_NAMES_DISTRIBUTION.get().unwrap();
        dist.pick_random_value(0, 0, stream)
    }

    /// Pick a random salutation using the specified weight category
    pub fn pick_random_salutation(
        weights: SalutationsWeights,
        stream: &mut RandomNumberStream,
    ) -> Result<&'static str> {
        Self::ensure_initialized()?;
        let dist = SALUTATIONS_DISTRIBUTION.get().unwrap();
        dist.pick_random_value(0, weights as usize, stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::RandomNumberStream;

    #[test]
    fn test_pick_random_first_name_male() {
        let mut stream = RandomNumberStream::new(1);
        let name = NamesDistributions::pick_random_first_name(
            FirstNamesWeights::MaleFrequency,
            &mut stream,
        )
        .unwrap();
        assert!(!name.is_empty());
    }

    #[test]
    fn test_pick_random_first_name_female() {
        let mut stream = RandomNumberStream::new(1);
        let name = NamesDistributions::pick_random_first_name(
            FirstNamesWeights::FemaleFrequency,
            &mut stream,
        )
        .unwrap();
        assert!(!name.is_empty());
    }

    #[test]
    fn test_pick_random_first_name_general() {
        let mut stream = RandomNumberStream::new(1);
        let name = NamesDistributions::pick_random_first_name(
            FirstNamesWeights::GeneralFrequency,
            &mut stream,
        )
        .unwrap();
        assert!(!name.is_empty());
    }

    #[test]
    fn test_pick_random_last_name() {
        let mut stream = RandomNumberStream::new(1);
        let name = NamesDistributions::pick_random_last_name(&mut stream).unwrap();
        assert!(!name.is_empty());
    }

    #[test]
    fn test_pick_random_salutation() {
        let mut stream = RandomNumberStream::new(1);

        let neutral = NamesDistributions::pick_random_salutation(
            SalutationsWeights::GenderNeutral,
            &mut stream,
        )
        .unwrap();
        assert!(!neutral.is_empty());

        let male =
            NamesDistributions::pick_random_salutation(SalutationsWeights::Male, &mut stream)
                .unwrap();
        assert!(!male.is_empty());

        let female =
            NamesDistributions::pick_random_salutation(SalutationsWeights::Female, &mut stream)
                .unwrap();
        assert!(!female.is_empty());
    }

    #[test]
    fn test_get_first_name_from_index() {
        let name = NamesDistributions::get_first_name_from_index(0).unwrap();
        assert!(!name.is_empty());
    }

    #[test]
    fn test_deterministic_behavior() {
        let mut stream1 = RandomNumberStream::new(42);
        let mut stream2 = RandomNumberStream::new(42);

        let name1 = NamesDistributions::pick_random_first_name(
            FirstNamesWeights::GeneralFrequency,
            &mut stream1,
        )
        .unwrap();

        let name2 = NamesDistributions::pick_random_first_name(
            FirstNamesWeights::GeneralFrequency,
            &mut stream2,
        )
        .unwrap();

        assert_eq!(name1, name2);
    }
}
