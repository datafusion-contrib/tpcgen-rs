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

//! [`SalesRowGenerator`]: a generator for the sales fact tables, which emit
//! several line items per source row (order) and a returns row for some of
//! them.
//!
//! See also [`SingleRowGenerator`](crate::row::SingleRowGenerator)

use crate::config::Session;
use crate::error::Result;

/// The rows generated for one line item of an order.
///
/// A generator constructed for one table (see [`SalesReturnsSelection`]) only
/// fills in that table's row.
///
/// [`SalesReturnsSelection`]: crate::row::SalesReturnsSelection
pub struct SalesRows<S, R> {
    pub sales: Option<S>,
    pub returns: Option<R>,
}

/// A generator that produces one [`SalesRows`] per call, several per source
/// row (order).
pub trait SalesRowGenerator: Send + Sync {
    /// The concrete sales row type this generator produces.
    type Sales;
    /// The concrete returns row type this generator produces.
    type Returns;

    /// Generate the next line item of `row_number` (1-based).
    fn generate_row(
        &mut self,
        row_number: u64,
        session: &Session,
    ) -> Result<SalesRows<Self::Sales, Self::Returns>>;

    /// Returns true if the line item just generated was the last one of its
    /// order, so the caller should advance to the next source row.
    fn is_last_row_in_order(&self) -> bool;

    /// Consume remaining seeds for the current row.
    fn consume_remaining_seeds_for_row(&mut self);

    /// Skip rows until reaching the starting row number.
    fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64);
}
