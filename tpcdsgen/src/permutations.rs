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

//! Permutation utilities for TPC-DS data generation.
//!
//! This module provides functionality to create random permutations, which are used
//! in sales table generation to ensure unique item selection within orders.

use crate::random::{RandomNumberStream, RandomValueGenerator};

/// Creates a random permutation of integers from 0 to size-1.
///
/// Uses the Fisher-Yates shuffle algorithm to generate a random permutation.
/// The permutation is an array where each position contains a unique value
/// from 0 to size-1.
///
/// # Arguments
///
/// * `size` - The number of elements in the permutation
/// * `stream` - The random number stream for generating random indices
///
/// # Returns
///
/// A vector containing a random permutation of integers [0, 1, 2, ..., size-1]
///
/// # Examples
///
/// ```
/// use tpcdsgen::random::RandomNumberStream;
/// use tpcdsgen::permutations::make_permutation;
///
/// let mut stream = RandomNumberStream::new(1).unwrap();
/// let perm = make_permutation(5, &mut stream);
/// // perm contains [0,1,2,3,4] in some random order
/// assert_eq!(perm.len(), 5);
/// ```
pub fn make_permutation(size: usize, stream: &mut RandomNumberStream) -> Vec<i32> {
    // Initialize array with sequential values [0, 1, 2, ..., size-1]
    let mut number_set: Vec<i32> = (0..size as i32).collect();

    // Fisher-Yates shuffle
    for i in 0..number_set.len() {
        let index = RandomValueGenerator::generate_uniform_random_int(0, (size - 1) as i32, stream)
            as usize;
        number_set.swap(i, index);
    }

    number_set
}

/// Gets an entry from a permutation using 1-based indexing.
///
/// **Important**: This function uses 1-based indexing (as per TPC-DS spec).
/// The returned value is also 1-based (permutation value + 1).
///
/// # Arguments
///
/// * `permutation` - The permutation array
/// * `index` - The 1-based index (must be >= 1)
///
/// # Returns
///
/// The value at the given index, incremented by 1 (to convert to 1-based)
///
/// # Panics
///
/// Panics if index < 1
///
/// # Examples
///
/// ```
/// use tpcdsgen::permutations::get_permutation_entry;
///
/// let perm = vec![2, 0, 1, 3];
/// let value = get_permutation_entry(&perm, 1);
/// // value is perm[0] + 1 = 2 + 1 = 3
/// assert_eq!(value, 3);
/// ```
pub fn get_permutation_entry(permutation: &[i32], index: i32) -> i32 {
    assert!(index >= 1, "index must be >= 1, got: {}", index);
    permutation[(index - 1) as usize] + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::RandomNumberStream;
    use std::collections::HashSet;

    #[test]
    fn test_make_permutation_correct_size() {
        let mut stream = RandomNumberStream::new(1).unwrap();
        let perm = make_permutation(10, &mut stream);
        assert_eq!(perm.len(), 10, "Permutation should have correct size");
    }

    #[test]
    fn test_make_permutation_contains_all_values() {
        let mut stream = RandomNumberStream::new(1).unwrap();
        let size = 20;
        let perm = make_permutation(size, &mut stream);

        // Convert to set to check for uniqueness and completeness
        let perm_set: HashSet<i32> = perm.iter().cloned().collect();

        assert_eq!(
            perm_set.len(),
            size,
            "Permutation should contain all unique values"
        );

        // Check that all values from 0 to size-1 are present
        for i in 0..size as i32 {
            assert!(
                perm_set.contains(&i),
                "Permutation should contain value {}",
                i
            );
        }
    }

    #[test]
    fn test_make_permutation_is_randomized() {
        let mut stream = RandomNumberStream::new(1).unwrap();
        let perm = make_permutation(10, &mut stream);

        // Check that it's not just the identity permutation [0,1,2,3,...]
        let is_identity = perm.iter().enumerate().all(|(i, &val)| val == i as i32);

        // Note: There's a small chance this could fail if random shuffle produces identity
        // But with 10 elements, probability is 1/10! which is negligible
        assert!(
            !is_identity,
            "Permutation should be shuffled (not identity)"
        );
    }

    #[test]
    fn test_make_permutation_deterministic() {
        // Same seed should produce same permutation
        let mut stream1 = RandomNumberStream::new(1).unwrap();
        let mut stream2 = RandomNumberStream::new(1).unwrap();

        let perm1 = make_permutation(10, &mut stream1);
        let perm2 = make_permutation(10, &mut stream2);

        assert_eq!(
            perm1, perm2,
            "Same random seed should produce identical permutation"
        );
    }

    #[test]
    fn test_get_permutation_entry_correct_value() {
        let perm = vec![2, 0, 1, 3];

        // Test 1-based indexing: index 1 gets perm[0] + 1
        assert_eq!(get_permutation_entry(&perm, 1), 3); // perm[0] = 2, +1 = 3
        assert_eq!(get_permutation_entry(&perm, 2), 1); // perm[1] = 0, +1 = 1
        assert_eq!(get_permutation_entry(&perm, 3), 2); // perm[2] = 1, +1 = 2
        assert_eq!(get_permutation_entry(&perm, 4), 4); // perm[3] = 3, +1 = 4
    }

    #[test]
    #[should_panic(expected = "index must be >= 1")]
    fn test_get_permutation_entry_rejects_zero_index() {
        let perm = vec![0, 1, 2];
        get_permutation_entry(&perm, 0);
    }

    #[test]
    #[should_panic(expected = "index must be >= 1")]
    fn test_get_permutation_entry_rejects_negative_index() {
        let perm = vec![0, 1, 2];
        get_permutation_entry(&perm, -1);
    }

    #[test]
    fn test_permutation_integration() {
        // Test the full workflow: create permutation, access entries
        let mut stream = RandomNumberStream::new(42).unwrap();
        let size = 5;
        let perm = make_permutation(size, &mut stream);

        // Access all entries using 1-based indexing
        let mut values = Vec::new();
        for i in 1..=size as i32 {
            values.push(get_permutation_entry(&perm, i));
        }

        // All values should be in range [1, size]
        for &val in &values {
            assert!(
                val >= 1 && val <= size as i32,
                "Value {} should be in range [1, {}]",
                val,
                size
            );
        }

        // All values should be unique
        let value_set: HashSet<i32> = values.iter().cloned().collect();
        assert_eq!(
            value_set.len(),
            size,
            "All accessed values should be unique"
        );
    }
}
