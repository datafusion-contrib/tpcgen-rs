use crate::{check_argument, error::Result, TpcdsError};

pub trait RandomNumberStream: Send + Sync {
    fn next_random(&mut self) -> i64;
    fn next_random_double(&mut self) -> f64;
    fn skip_rows(&mut self, number_of_rows: u64);
    fn reset_seed(&mut self);
    fn get_seeds_used(&self) -> i32;
    fn reset_seeds_used(&mut self);
    fn get_seeds_per_row(&self) -> i32;
}

#[derive(Debug, Clone)]
pub struct RandomNumberStreamImpl {
    // Constants matching Java implementation exactly
    seed: i64,
    initial_seed: i64,
    seeds_used: i32,
    seeds_per_row: i32,
}

impl RandomNumberStreamImpl {
    const DEFAULT_SEED_BASE: i32 = 19620718;
    const MULTIPLIER: i64 = 16807;
    const QUOTIENT: i64 = 127773; // the quotient MAX_INT / MULTIPLIER
    const REMAINDER: i64 = 2836; // the remainder MAX_INT % MULTIPLIER

    pub fn new(seeds_per_row: i32) -> Result<Self> {
        check_argument!(seeds_per_row >= 0, "seedsPerRow must be >=0");
        Ok(RandomNumberStreamImpl {
            initial_seed: 3,
            seed: 3,
            seeds_used: 0,
            seeds_per_row,
        })
    }

    pub fn new_with_column(global_column_number: i32, seeds_per_row: i32) -> Result<Self> {
        Self::new_with_base(global_column_number, Self::DEFAULT_SEED_BASE, seeds_per_row)
    }

    pub fn new_with_base(
        global_column_number: i32,
        seed_base: i32,
        seeds_per_row: i32,
    ) -> Result<Self> {
        check_argument!(seeds_per_row >= 0, "seedsPerRow must be >=0");
        let initial_seed = seed_base as i64 + global_column_number as i64 * (i32::MAX as i64 / 799);
        Ok(RandomNumberStreamImpl {
            initial_seed,
            seed: initial_seed,
            seeds_used: 0,
            seeds_per_row,
        })
    }
}

impl RandomNumberStream for RandomNumberStreamImpl {
    // https://en.wikipedia.org/wiki/Lehmer_random_number_generator
    fn next_random(&mut self) -> i64 {
        let mut next_seed = self.seed;
        let division_result = next_seed / Self::QUOTIENT;
        let mod_result = next_seed % Self::QUOTIENT;
        next_seed = Self::MULTIPLIER * mod_result - division_result * Self::REMAINDER;
        if next_seed < 0 {
            next_seed += i32::MAX as i64;
        }

        self.seed = next_seed;
        self.seeds_used += 1;
        self.seed
    }

    fn next_random_double(&mut self) -> f64 {
        self.next_random() as f64 / i32::MAX as f64
    }

    fn skip_rows(&mut self, number_of_rows: u64) {
        let mut number_of_values_to_skip = number_of_rows * self.seeds_per_row as u64;
        let mut next_seed = self.initial_seed;
        let mut multiplier = Self::MULTIPLIER;

        while number_of_values_to_skip > 0 {
            if !number_of_values_to_skip.is_multiple_of(2) {
                // n is odd
                next_seed = (multiplier * next_seed) % i32::MAX as i64;
            }
            number_of_values_to_skip /= 2;
            multiplier = (multiplier * multiplier) % i32::MAX as i64;
        }

        self.seed = next_seed;
        self.seeds_used = 0;
    }

    fn reset_seed(&mut self) {
        self.seed = self.initial_seed;
        self.seeds_used = 0;
    }

    fn get_seeds_used(&self) -> i32 {
        self.seeds_used
    }

    fn reset_seeds_used(&mut self) {
        self.seeds_used = 0;
    }

    fn get_seeds_per_row(&self) -> i32 {
        self.seeds_per_row
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_stream_creation() {
        let stream = RandomNumberStreamImpl::new(1).unwrap();
        assert_eq!(stream.get_seeds_per_row(), 1);
        assert_eq!(stream.get_seeds_used(), 0);
    }

    #[test]
    fn test_random_stream_with_column() {
        let stream = RandomNumberStreamImpl::new_with_column(1, 1).unwrap();
        assert_eq!(stream.get_seeds_per_row(), 1);

        // Initial seed should be computed based on column number
        assert_ne!(stream.initial_seed, 3); // Should be different from default
    }

    #[test]
    fn test_next_random() {
        let mut stream = RandomNumberStreamImpl::new(1).unwrap();
        let first = stream.next_random();
        let second = stream.next_random();

        // Should generate different values
        assert_ne!(first, second);
        assert_eq!(stream.get_seeds_used(), 2);
    }

    #[test]
    fn test_random_double() {
        let mut stream = RandomNumberStreamImpl::new(1).unwrap();
        let random_double = stream.next_random_double();

        // Should be between 0 and 1
        assert!((0.0..=1.0).contains(&random_double));
    }

    #[test]
    fn test_reset_seed() {
        let mut stream = RandomNumberStreamImpl::new(1).unwrap();
        let initial = stream.next_random();
        stream.next_random(); // Generate another

        stream.reset_seed();
        let after_reset = stream.next_random();

        assert_eq!(initial, after_reset);
        assert_eq!(stream.get_seeds_used(), 1);
    }

    #[test]
    fn test_skip_rows() {
        let mut stream1 = RandomNumberStreamImpl::new(2).unwrap();
        let mut stream2 = RandomNumberStreamImpl::new(2).unwrap();

        // Generate 2 rows manually on stream1
        stream1.next_random();
        stream1.next_random();
        stream1.next_random();
        stream1.next_random();

        // Skip 2 rows on stream2
        stream2.skip_rows(2);

        // Both should now generate the same next value
        let manual = stream1.next_random();
        let skipped = stream2.next_random();
        assert_eq!(manual, skipped);
    }
}
