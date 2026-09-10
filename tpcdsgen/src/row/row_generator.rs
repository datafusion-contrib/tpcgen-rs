use crate::config::Session;
use crate::row::GeneratedRow;

/// Result of row generation (RowGeneratorResult)
///
/// Uses `GeneratedRow` enum instead of a boxed trait object to avoid
/// heap allocations and enable static dispatch. See ISSUE-004.
pub struct RowGeneratorResult {
    rows: Vec<GeneratedRow>,
    should_end_row: bool,
}

impl RowGeneratorResult {
    /// Create a result with a single row
    pub fn new<R: Into<GeneratedRow>>(row: R) -> Self {
        Self {
            rows: vec![row.into()],
            should_end_row: true,
        }
    }

    /// Create a result with multiple rows and end flag
    pub fn new_with_multiple(rows: Vec<GeneratedRow>, should_end_row: bool) -> Self {
        Self {
            rows,
            should_end_row,
        }
    }

    /// Get the generated rows
    pub fn get_rows(&self) -> &[GeneratedRow] {
        &self.rows
    }

    /// Check if row generation should end
    pub fn should_end_row(&self) -> bool {
        self.should_end_row
    }
}

/// RowGenerator trait matching the Java RowGenerator interface
pub trait RowGenerator: Send + Sync {
    /// Generate a row and its child rows (generateRowAndChildRows).
    ///
    /// `row_number` is 1-based.
    fn generate_row_and_child_rows(
        &mut self,
        row_number: u64,
        session: &Session,
        parent_row_generator: Option<&mut dyn RowGenerator>,
        child_row_generator: Option<&mut dyn RowGenerator>,
    ) -> crate::error::Result<RowGeneratorResult>;

    /// Consume remaining seeds for the current row
    fn consume_remaining_seeds_for_row(&mut self);

    /// Skip rows until reaching the starting row number
    fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::row::CallCenterRow;

    #[test]
    fn test_row_generator_result_single() {
        let row = CallCenterRow::builder().build();
        let result = RowGeneratorResult::new(row);

        assert_eq!(result.get_rows().len(), 1);
        assert!(result.should_end_row());
    }

    #[test]
    fn test_row_generator_result_multiple() {
        let rows = vec![
            GeneratedRow::from(CallCenterRow::builder().build()),
            GeneratedRow::from(CallCenterRow::builder().build()),
        ];
        let result = RowGeneratorResult::new_with_multiple(rows, false);

        assert_eq!(result.get_rows().len(), 2);
        assert!(!result.should_end_row());
    }
}
