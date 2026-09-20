//! Implementation of the core random number generators.

use crate::{distribution::Distribution, text::TextPool};
use std::fmt::Display;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RowRandomInt {
    seed: i64,
    usage: i32,
    seeds_per_row: i32,
}

impl RowRandomInt {
    /// The default multiplier value is a TPC-H constant.
    const MULTIPLIER: i64 = 16807;
    /// Modulus value is a TPC-H constant 2^31 - 1.
    const MODULUS: i64 = 2147483647;
    /// Default seed value as specified in CMU's benchbase.
    const DEFAULT_SEED: i64 = 19620718;

    /// Creates a new random number generator with the given seed and potential
    /// number of random values per row.
    pub fn new(seed: i64, seeds_per_row: i32) -> Self {
        Self {
            seed,
            usage: 0,
            seeds_per_row,
        }
    }

    /// Creates a new random number generator with a specified column number and number of
    /// random values per row. Uses the default seed value `19620718`.
    pub fn new_with_default_seed_and_column_number(column_number: i32, seeds_per_row: i32) -> Self {
        Self::new_with_column_number(column_number, Self::DEFAULT_SEED, seeds_per_row)
    }

    /// Creates a new random number generator with a specified column number and number of
    /// random values per row. The seed can be specified.
    pub fn new_with_column_number(column_number: i32, seed: i64, seeds_per_row: i32) -> Self {
        Self {
            seed: seed + column_number as i64 * (Self::MODULUS / 799),
            seeds_per_row,
            usage: 0,
        }
    }

    /// Returns a random value between lower and upper bounds (both inclusive).
    pub fn next_int(&mut self, lower_bound: i32, upper_bound: i32) -> i32 {
        let _ = self.next_rand();

        // This is buggy because it can overflow e.g in `RandomAlphaNumeric` when
        // `upper_bound` is `i32::MAX` and `lower_bound` is 0 so to replicate
        // the overflow behaviour we need to wrap around to `i32::MIN`.
        let range = (upper_bound - lower_bound).wrapping_add(1) as f64;
        let value = ((1.0 * (self.seed as f64) / Self::MODULUS as f64) * range) as i32;

        lower_bound + value
    }

    /// Instantiates a new seed for the next random value.
    pub fn next_rand(&mut self) -> i64 {
        self.seed = (self.seed * Self::MULTIPLIER) % Self::MODULUS;
        self.usage += 1;
        self.seed
    }

    /// Signals to the random number generator that we consumed the seeds for the current row
    /// and we need to start generating new ones.
    pub fn row_finished(&mut self) {
        self.advance_seed((self.seeds_per_row - self.usage) as i64);
        self.usage = 0;
    }

    /// Advance the specified number of rows which is required for partitioned datasets.
    pub fn advance_rows(&mut self, row_count: i64) {
        // Signals the we consumed all the seeds for the current row.
        if self.usage != 0 {
            self.row_finished();
        }

        self.advance_seed(self.seeds_per_row as i64 * row_count);
    }

    /// Advances the seed value by the number of uses.
    fn advance_seed(&mut self, count: i64) {
        let mut multiplier = Self::MULTIPLIER;
        let mut count = count;

        while count > 0 {
            if count % 2 != 0 {
                self.seed = (multiplier * self.seed) % Self::MODULUS;
            }
            count /= 2;
            multiplier = (multiplier * multiplier) % Self::MODULUS;
        }
    }
}

/// Random 64-bit integer generator for large scale factors
#[derive(Default, Debug, Clone, Copy)]
pub struct RowRandomLong {
    seed: i64,
    seeds_per_row: i32,
    usage: i32,
}

impl RowRandomLong {
    /// The default multiplier value is a TPC-H constant.
    const MULTIPLIER: i64 = 6364136223846793005;
    /// The default increment used.
    const INCREMENT: i64 = 1;

    /// The default multiplier for 32-bit seeds (a TPC-H oddity).
    const MULTIPLIER_32: i64 = 16807;
    /// The default modulus for 32-bit seeds (a TPC-H oddity).
    const MODULUS_32: i64 = 2147483647;

    /// Instantiates a new random number generator with the specified seed and number of random
    /// values per row.
    pub fn new(seed: i64, seeds_per_row: i32) -> Self {
        Self {
            seed,
            seeds_per_row,
            usage: 0,
        }
    }

    /// Returnsa random value between `lower_bound` and `upper_bound` (both inclusive).
    pub fn next_long(&mut self, lower_bound: i64, upper_bound: i64) -> i64 {
        self.next_rand();

        let value_in_range = (self.seed.abs()) % (upper_bound - lower_bound + 1);

        lower_bound + value_in_range
    }

    /// Instantiates a new seed for the next random value.
    fn next_rand(&mut self) -> i64 {
        self.seed = (self.seed.wrapping_mul(Self::MULTIPLIER)) + Self::INCREMENT;
        self.usage += 1;
        self.seed
    }

    /// Signals that all seeds for the current row have been consumed and we need to start
    /// generating new ones.
    pub fn row_finished(&mut self) {
        // For the 64-bit case, TPC-H actually uses the 32-bit advance method
        self.advance_seed_32((self.seeds_per_row - self.usage) as i64);
        self.usage = 0;
    }

    /// Advances the seed by the specified number of rows
    pub fn advance_rows(&mut self, row_count: i64) {
        // Finish current row if needed
        if self.usage != 0 {
            self.row_finished();
        }

        // Advance the seed
        self.advance_seed_32((self.seeds_per_row as i64) * row_count);
    }

    // TPC-H uses this 32-bit method even for 64-bit numbers
    fn advance_seed_32(&mut self, mut count: i64) {
        let mut multiplier: i64 = Self::MULTIPLIER_32;

        while count > 0 {
            if count % 2 != 0 {
                self.seed = multiplier.wrapping_mul(self.seed) % Self::MODULUS_32;
            }

            // Integer division, truncates
            count /= 2;
            multiplier = (multiplier * multiplier) % Self::MODULUS_32;
        }
    }
}

/// Random number generator for bounded values.
#[derive(Default, Debug, Clone, Copy)]
pub struct RandomBoundedInt {
    lower_bound: i32,
    upper_bound: i32,
    random_int: RowRandomInt,
}

impl RandomBoundedInt {
    /// Creates a new random number generator with the given seed and lower and upper bounds.
    pub fn new(seed: i64, lower_bound: i32, upper_bound: i32) -> Self {
        Self {
            lower_bound,
            upper_bound,
            random_int: RowRandomInt::new(seed, 1),
        }
    }

    /// Creates a new random number generator with the given seed and lower and upper bounds
    /// and the number of random values per row.
    pub fn new_with_seeds_per_row(
        seed: i64,
        lower_bound: i32,
        upper_bound: i32,
        seeds_per_row: i32,
    ) -> Self {
        Self {
            lower_bound,
            upper_bound,
            random_int: RowRandomInt::new(seed, seeds_per_row),
        }
    }

    /// Returns a random value between the lower and upper bounds (both inclusive).
    pub fn next_value(&mut self) -> i32 {
        self.random_int.next_int(self.lower_bound, self.upper_bound)
    }

    /// Advance the inner random number generator by the specified number of rows.
    pub fn advance_rows(&mut self, row_count: i64) {
        self.random_int.advance_rows(row_count);
    }

    pub fn row_finished(&mut self) {
        self.random_int.row_finished();
    }
}

/// Random number generator for bounded 64-bit values.
#[derive(Default, Debug, Clone, Copy)]
pub struct RandomBoundedLong {
    use_64bits: bool,
    lower_bound: i64,
    upper_bound: i64,
    // 64-bit values.
    random_long: RowRandomLong,
    // 32-bit values.
    random_int: RowRandomInt,
}

impl RandomBoundedLong {
    /// Creates a new random number generator with the given seed and lower and upper bounds.
    pub fn new(seed: i64, use_64bits: bool, lower_bound: i64, upper_bound: i64) -> Self {
        Self {
            lower_bound,
            use_64bits,
            upper_bound,
            random_long: RowRandomLong::new(seed, 1),
            random_int: RowRandomInt::new(seed, 1),
        }
    }

    /// Creates a new random number generator with the given seed and lower and upper bounds
    /// and the number of random values per row.
    pub fn new_with_seeds_per_row(
        seed: i64,
        use_64bits: bool,
        lower_bound: i64,
        upper_bound: i64,
        seeds_per_row: i32,
    ) -> Self {
        Self {
            use_64bits,
            lower_bound,
            upper_bound,
            random_long: RowRandomLong::new(seed, seeds_per_row),
            random_int: RowRandomInt::new(seed, seeds_per_row),
        }
    }

    /// Returns a random value between the lower and upper bounds (both inclusive).
    pub fn next_value(&mut self) -> i64 {
        if self.use_64bits {
            self.random_long
                .next_long(self.lower_bound, self.upper_bound)
        } else {
            self.random_int
                .next_int(self.lower_bound as i32, self.upper_bound as i32) as i64
        }
    }

    /// Advance the inner random number generator by the specified number of rows.
    pub fn advance_rows(&mut self, row_count: i64) {
        if self.use_64bits {
            self.random_long.advance_rows(row_count);
        } else {
            self.random_int.advance_rows(row_count);
        }
    }

    pub fn row_finished(&mut self) {
        if self.use_64bits {
            self.random_long.row_finished();
        } else {
            self.random_int.row_finished();
        }
    }
}

/// Generates random alphanumeric strings
#[derive(Debug)]
pub struct RandomAlphaNumeric {
    inner: RowRandomInt,
    min_length: i32,
    max_length: i32,
}

impl RandomAlphaNumeric {
    // Characters allowed in alphanumeric strings
    const ALPHA_NUMERIC: &'static [u8] =
        b"0123456789abcdefghijklmnopqrstuvwxyz ABCDEFGHIJKLMNOPQRSTUVWXYZ,";

    // Length multipliers from TPC-H spec
    const LOW_LENGTH_MULTIPLIER: f64 = 0.4;
    const HIGH_LENGTH_MULTIPLIER: f64 = 1.6;

    // Usage count per row
    const USAGE_PER_ROW: i32 = 9;

    pub fn new(seed: i64, average_length: i32) -> Self {
        Self::new_with_expected_row_count(seed, average_length, 1)
    }

    pub fn new_with_expected_row_count(seed: i64, average_length: i32, seeds_per_row: i32) -> Self {
        let min_length = (average_length as f64 * Self::LOW_LENGTH_MULTIPLIER) as i32;
        let max_length = (average_length as f64 * Self::HIGH_LENGTH_MULTIPLIER) as i32;

        Self {
            inner: RowRandomInt::new(seed, Self::USAGE_PER_ROW * seeds_per_row),
            min_length,
            max_length,
        }
    }

    /// Returns the next string as a [`RandomAlphaNumericInstance`], which can
    /// generate the string on demand.
    pub fn next_value(&mut self) -> RandomAlphaNumericInstance {
        let length = self.inner.next_int(self.min_length, self.max_length) as usize;

        RandomAlphaNumericInstance {
            length,
            snapshot: self.inner,
        }
    }

    /// Advance the inner random number generator by the specified number of rows.
    pub fn advance_rows(&mut self, row_count: i64) {
        self.inner.advance_rows(row_count);
    }

    pub fn row_finished(&mut self) {
        self.inner.row_finished();
    }
}

/// A random alphanumeric string. To avoid allocations
/// the string is created on demand with the Display implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RandomAlphaNumericInstance {
    length: usize,
    /// snapshot of the random number generator
    snapshot: RowRandomInt,
}

impl Display for RandomAlphaNumericInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Use up to  64 bytes of a stack buffer for small strings to avoid
        // allocation, and heap allocation for larger ones.
        let mut stack_buffer = [0u8; 64];
        let mut heap_buffer = Vec::new();

        let buffer = if self.length <= stack_buffer.len() {
            &mut stack_buffer[0..self.length]
        } else {
            heap_buffer.resize(self.length, 0);
            &mut heap_buffer
        };

        let mut generator = self.snapshot; // copy to for mutation

        let mut char_index = 0;
        // todo remove
        #[allow(clippy::needless_range_loop)]
        for i in 0..self.length {
            if i % 5 == 0 {
                char_index = generator.next_int(0, i32::MAX) as i64;
            }

            let char_pos = (char_index & 0x3f) as usize;
            buffer[i] = RandomAlphaNumeric::ALPHA_NUMERIC[char_pos];
            char_index >>= 6;
        }
        // Safety: only pushed ascii characters into the buffer
        let s = unsafe { std::str::from_utf8_unchecked(buffer) };
        f.write_str(s)?;
        Ok(())
    }
}

/// Generates phone numbers according to TPC-H spec
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RandomPhoneNumber {
    inner: RowRandomInt,
}

impl RandomPhoneNumber {
    // Maximum number of nations in TPC-H
    const NATIONS_MAX: i32 = 90;

    pub fn new(seed: i64) -> Self {
        Self::new_with_expected_row_count(seed, 1)
    }

    pub fn new_with_expected_row_count(seed: i64, seeds_per_row: i32) -> Self {
        Self {
            inner: RowRandomInt::new(seed, 3 * seeds_per_row),
        }
    }

    pub fn next_value(&mut self, nation_key: i64) -> PhoneNumberInstance {
        PhoneNumberInstance {
            country_code: 10 + (nation_key % Self::NATIONS_MAX as i64) as i32,
            local1: self.inner.next_int(100, 999),
            local2: self.inner.next_int(100, 999),
            local3: self.inner.next_int(1000, 9999),
        }
    }

    /// Advance the inner random number generator by the specified number of rows.
    pub fn advance_rows(&mut self, row_count: i64) {
        self.inner.advance_rows(row_count);
    }

    pub fn row_finished(&mut self) {
        self.inner.row_finished();
    }
}

/// A displayable phone number
///
/// Example display:
/// ```text
/// 27-918-335-1736
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhoneNumberInstance {
    country_code: i32,
    local1: i32,
    local2: i32,
    local3: i32,
}

impl Display for PhoneNumberInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02}-{:03}-{:03}-{:04}",
            self.country_code, self.local1, self.local2, self.local3
        )
    }
}

/// Fetches random strings from a distribution.
#[derive(Debug, Clone)]
pub struct RandomString<'a> {
    inner: RowRandomInt,
    distribution: &'a Distribution,
}

impl<'a> RandomString<'a> {
    pub fn new(seed: i64, distribution: &'a Distribution) -> Self {
        Self::new_with_expected_row_count(seed, distribution, 1)
    }

    pub fn new_with_expected_row_count(
        seed: i64,
        distribution: &'a Distribution,
        seeds_per_row: i32,
    ) -> Self {
        Self {
            inner: RowRandomInt::new(seed, seeds_per_row),
            distribution,
        }
    }

    pub fn next_value(&mut self) -> &'a str {
        self.distribution.random_value(&mut self.inner)
    }

    /// Advance the inner random number generator by the given number of rows.
    pub fn advance_rows(&mut self, row_count: i64) {
        self.inner.advance_rows(row_count);
    }

    pub fn row_finished(&mut self) {
        self.inner.row_finished();
    }
}

/// Generates sequences of random sequence of strings from a distribution
#[derive(Debug)]
pub struct RandomStringSequence<'a> {
    inner: RowRandomInt,
    count: i32,
    distribution: &'a Distribution,
}

impl<'a> RandomStringSequence<'a> {
    pub fn new(seed: i64, count: i32, distribution: &'a Distribution) -> Self {
        Self::new_with_expected_row_count(seed, count, distribution, 1)
    }

    pub fn new_with_expected_row_count(
        seed: i64,
        count: i32,
        distribution: &'a Distribution,
        seeds_per_row: i32,
    ) -> Self {
        Self {
            inner: RowRandomInt::new(seed, distribution.size() as i32 * seeds_per_row),
            count,
            distribution,
        }
    }

    pub fn next_value(&mut self) -> StringSequenceInstance<'a> {
        // Get all values from the distribution
        let mut values: Vec<&str> = self.distribution.get_values().to_vec();

        // Randomize first 'count' elements
        for current_position in 0..self.count {
            // Pick a random position to swap with
            let swap_position =
                self.inner
                    .next_int(current_position, values.len() as i32 - 1) as usize;

            // Swap the elements
            values.swap(current_position as usize, swap_position);
        }

        // Keep only the first 'count' values, and join them with spaces
        values.truncate(self.count as usize);
        StringSequenceInstance { values }
    }

    /// Advance the inner random number generator by the given number of rows.
    pub fn advance_rows(&mut self, row_count: i64) {
        self.inner.advance_rows(row_count);
    }

    pub fn row_finished(&mut self) {
        self.inner.row_finished();
    }
}

/// Displayable string sequence instance
///
/// Prints the sequence of strings as a single string with spaces between them.
///
/// Example display:
/// ```text
/// "value1 value2 value3"
/// ```
#[derive(Default, Debug, Clone, PartialEq)]
pub struct StringSequenceInstance<'a> {
    values: Vec<&'a str>,
}

impl Display for StringSequenceInstance<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.values.iter();
        if let Some(first) = iter.next() {
            write!(f, "{}", first)?;
        }
        for value in iter {
            write!(f, " {}", value)?;
        }
        Ok(())
    }
}

/// Generates random text according to TPC-H spec
#[derive(Debug, Clone)]
pub struct RandomText<'a> {
    inner: RowRandomInt,
    text_pool: &'a TextPool,
    min_length: i32,
    max_length: i32,
}

impl<'a> RandomText<'a> {
    const LOW_LENGTH_MULTIPLIER: f64 = 0.4;
    const HIGH_LENGTH_MULTIPLIER: f64 = 1.6;

    pub fn new(seed: i64, text_pool: &'a TextPool, average_text_length: f64) -> Self {
        Self::new_with_expected_row_count(seed, text_pool, average_text_length, 1)
    }

    pub fn new_with_expected_row_count(
        seed: i64,
        text_pool: &'a TextPool,
        average_text_length: f64,
        expected_row_count: i32,
    ) -> Self {
        Self {
            inner: RowRandomInt::new(seed, expected_row_count * 2),
            text_pool,
            min_length: (average_text_length * Self::LOW_LENGTH_MULTIPLIER) as i32,
            max_length: (average_text_length * Self::HIGH_LENGTH_MULTIPLIER) as i32,
        }
    }

    pub fn next_value(&mut self) -> &'a str {
        let offset = self
            .inner
            .next_int(0, self.text_pool.size() - self.max_length);
        let length = self.inner.next_int(self.min_length, self.max_length);

        self.text_pool.text(offset, offset + length)
    }

    pub fn advance_rows(&mut self, row_count: i64) {
        self.inner.advance_rows(row_count);
    }

    pub fn row_finished(&mut self) {
        self.inner.row_finished();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;
    #[test]
    fn test_small_random_alpha_numeric() {
        RandomAlphaNumericTest {
            average_length: 10,
            num_rows: 100,
            expected_average_length: 10,
        }
        .assert()
    }

    #[test]
    fn test_large_random_alpha_numeric() {
        RandomAlphaNumericTest {
            average_length: 100,
            num_rows: 100,
            expected_average_length: 102,
        }
        .assert()
    }

    struct RandomAlphaNumericTest {
        average_length: i32,
        num_rows: usize,
        expected_average_length: usize,
    }
    impl RandomAlphaNumericTest {
        fn assert(self) {
            let Self {
                average_length,
                num_rows,
                expected_average_length,
            } = self;

            let mut generator = RandomAlphaNumeric::new(1, average_length);
            let mut values = HashSet::new();
            // check that the values are within the expected length and not repeated
            let mut total_len = 0;
            for _ in 0..num_rows {
                let value = generator.next_value().to_string();
                total_len += value.len();
                assert!(values.insert(value)); // no dupes
            }
            assert_eq!(total_len / num_rows, expected_average_length);
        }
    }
}
