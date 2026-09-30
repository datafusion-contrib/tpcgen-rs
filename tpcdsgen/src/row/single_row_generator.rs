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

//! [`SingleRowGenerator`]: a typed generator trait for tables that emit
//! exactly one concrete row per source row.
//!
//! Compared to [`RowGenerator`](crate::row::RowGenerator), this trait exposes
//! the generator's concrete `Row` type directly, so callers never wrap or
//! match on the [`GeneratedRow`](crate::row::GeneratedRow) enum and never
//! allocate a per-call [`RowGeneratorResult`](crate::row::RowGeneratorResult).
//!
//! This trait is independent of [`RowGenerator`](crate::row::RowGenerator)
//! and intentionally does not support paired fact-table generation (a single
//! source row producing rows for more than one table); use `RowGenerator`
//! for those generators.

use crate::config::Session;
use crate::error::Result;

/// A generator that produces exactly one `Self::Row` per source row.
pub trait SingleRowGenerator: Send + Sync {
    /// The concrete row type this generator produces.
    type Row;

    /// Generate the row for `row_number` (1-based).
    fn generate_row(&mut self, row_number: u64, session: &Session) -> Result<Self::Row>;

    /// Consume remaining seeds for the current row.
    fn consume_remaining_seeds_for_row(&mut self);

    /// Skip rows until reaching the starting row number.
    fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64);
}
