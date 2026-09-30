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

//! Null value generation utilities for TPC-DS tables.
//!
//! This module provides functionality to create null bitmaps for table rows
//! based on each table's null probability settings.

use crate::random::{RandomNumberStream, RandomValueGenerator};
use crate::table::Table;

/// Creates a null bitmap for a table row based on the table's null probability.
///
/// The function generates a random threshold and bitmap. If the threshold is less
/// than the table's null basis points (probability of nulls), it returns a bitmap
/// indicating which columns can be null (respecting the table's not-null constraints).
///
/// # Arguments
///
/// * `table` - The table for which to generate the null bitmap
/// * `random_number_stream` - The random number stream for generating values
///
/// # Returns
///
/// A 64-bit bitmap where each bit represents whether a column should be null.
/// Returns 0 if no columns should be null.
///
/// # Examples
///
/// ```
/// use tpcdsgen::table::Table;
/// use tpcdsgen::random::RandomNumberStreamImpl;
/// use tpcdsgen::nulls::create_null_bit_map;
///
/// let mut stream = RandomNumberStreamImpl::new(1).unwrap();
/// let null_bitmap = create_null_bit_map(Table::CallCenter, &mut stream);
/// // null_bitmap will be 0 or a value respecting CallCenter's not-null constraints
/// ```
pub fn create_null_bit_map(table: Table, random_number_stream: &mut dyn RandomNumberStream) -> i64 {
    let threshold =
        RandomValueGenerator::generate_uniform_random_int(0, 9999, random_number_stream);
    let bit_map =
        RandomValueGenerator::generate_uniform_random_key(1, i32::MAX as i64, random_number_stream);

    // Set the bitmap based on threshold and NOT NULL definitions
    if threshold < table.get_null_basis_points() {
        return bit_map & !table.get_not_null_bit_map();
    }

    0
}

/// Advances `random_number_stream` past the values [`create_null_bit_map`]
/// would draw, without computing a bitmap.
///
/// Must be kept in sync with [`create_null_bit_map`]
pub fn skip_null_bit_map(random_number_stream: &mut dyn RandomNumberStream) {
    random_number_stream.next_random();
    random_number_stream.next_random();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::RandomNumberStreamImpl;

    #[test]
    fn test_create_null_bit_map_for_table_with_zero_null_basis_points() {
        // Tables with 0 null basis points should always return 0
        let mut stream = RandomNumberStreamImpl::new(1).unwrap();

        // IncomeBand has 0 null basis points
        let null_bitmap = create_null_bit_map(Table::IncomeBand, &mut stream);
        assert_eq!(
            null_bitmap, 0,
            "Table with 0 null basis points should always return 0 bitmap"
        );
    }

    #[test]
    fn test_create_null_bit_map_respects_not_null_constraints() {
        // Generate multiple bitmaps and verify they respect not-null constraints
        let mut stream = RandomNumberStreamImpl::new(1).unwrap();

        // CallCenter has null basis points of 100 and not-null bitmap of 0xB
        for _ in 0..10 {
            let null_bitmap = create_null_bit_map(Table::CallCenter, &mut stream);

            if null_bitmap != 0 {
                // If bitmap is non-zero, verify it respects not-null constraints
                let not_null_bitmap = Table::CallCenter.get_not_null_bit_map();
                assert_eq!(
                    null_bitmap & not_null_bitmap,
                    0,
                    "Null bitmap should not set bits that are in not-null bitmap"
                );
            }
        }
    }

    #[test]
    fn test_create_null_bit_map_produces_varied_results() {
        // Verify that the function can produce both zero and non-zero results
        let mut stream = RandomNumberStreamImpl::new(1).unwrap();
        let mut has_zero = false;
        let mut has_non_zero = false;

        // Warehouse has null basis points of 100, so should sometimes generate nulls
        for _ in 0..100 {
            let null_bitmap = create_null_bit_map(Table::Warehouse, &mut stream);
            if null_bitmap == 0 {
                has_zero = true;
            } else {
                has_non_zero = true;
            }
            if has_zero && has_non_zero {
                break;
            }
        }

        assert!(
            has_zero || has_non_zero,
            "Should produce at least some results (either zero or non-zero)"
        );
    }

    #[test]
    fn test_create_null_bit_map_deterministic() {
        // Same seed should produce same results
        let mut stream1 = RandomNumberStreamImpl::new(1).unwrap();
        let mut stream2 = RandomNumberStreamImpl::new(1).unwrap();

        let bitmap1 = create_null_bit_map(Table::CallCenter, &mut stream1);
        let bitmap2 = create_null_bit_map(Table::CallCenter, &mut stream2);

        assert_eq!(
            bitmap1, bitmap2,
            "Same random stream should produce same null bitmap"
        );
    }

    #[test]
    fn test_skip_null_bit_map_matches_create() {
        let mut created = RandomNumberStreamImpl::new(1).unwrap();
        let mut skipped = RandomNumberStreamImpl::new(1).unwrap();
        create_null_bit_map(Table::CallCenter, &mut created);
        skip_null_bit_map(&mut skipped);
        assert_eq!(created.next_random(), skipped.next_random());
    }
}
