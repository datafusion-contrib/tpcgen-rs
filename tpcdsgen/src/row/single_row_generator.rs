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

//! [`SingleRowGenerator`]: a generator  for tables that emit exactly one row
//! per source row.
//!
//! See also [`RowGenerator`](crate::row::RowGenerator)

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
