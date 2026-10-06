use crate::column::Table;
use crate::generator::GeneratorColumn;

/// Call Center generator columns (CallCenterGeneratorColumn enum)
/// These are used internally by the generator and may include non-visible columns
/// or omit visible columns that get derived from other columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CallCenterGeneratorColumn {
    CcCallCenterSk,
    CcCallCenterId,
    CcRecStartDateId,
    CcRecEndDateId,
    CcClosedDateId,
    CcOpenDateId,
    CcName,
    CcClass,
    CcEmployees,
    CcSqFt,
    CcHours,
    CcManager,
    CcMarketId,
    CcMarketClass,
    CcMarketDesc,
    CcMarketManager,
    CcDivision,
    CcDivisionName,
    CcCompany,
    CcCompanyName,
    CcStreetNumber,
    CcStreetName,
    CcStreetType,
    CcSuiteNumber,
    CcCity,
    CcCounty,
    CcState,
    CcZip,
    CcCountry,
    CcGmtOffset,
    CcAddress,
    CcTaxPercentage,
    CcScd,
    CcNulls,
}

impl CallCenterGeneratorColumn {
    /// Get all generator columns in order
    pub fn values() -> &'static [CallCenterGeneratorColumn] {
        use CallCenterGeneratorColumn::*;
        static VALUES: &[CallCenterGeneratorColumn] = &[
            CcCallCenterSk,
            CcCallCenterId,
            CcRecStartDateId,
            CcRecEndDateId,
            CcClosedDateId,
            CcOpenDateId,
            CcName,
            CcClass,
            CcEmployees,
            CcSqFt,
            CcHours,
            CcManager,
            CcMarketId,
            CcMarketClass,
            CcMarketDesc,
            CcMarketManager,
            CcDivision,
            CcDivisionName,
            CcCompany,
            CcCompanyName,
            CcStreetNumber,
            CcStreetName,
            CcStreetType,
            CcSuiteNumber,
            CcCity,
            CcCounty,
            CcState,
            CcZip,
            CcCountry,
            CcGmtOffset,
            CcAddress,
            CcTaxPercentage,
            CcScd,
            CcNulls,
        ];
        VALUES
    }

    /// Get the global column number and seeds per row for this generator column
    /// Values exactly match Java implementation
    fn get_column_info(&self) -> (i32, u32) {
        use CallCenterGeneratorColumn::*;
        match self {
            CcCallCenterSk => (1, 0),
            CcCallCenterId => (2, 15),
            CcRecStartDateId => (3, 10),
            CcRecEndDateId => (4, 1),
            CcClosedDateId => (5, 4),
            CcOpenDateId => (6, 10),
            CcName => (7, 0),
            CcClass => (8, 2),
            CcEmployees => (9, 1),
            CcSqFt => (10, 1),
            CcHours => (11, 1),
            CcManager => (12, 2),
            CcMarketId => (13, 1),
            CcMarketClass => (14, 50),
            CcMarketDesc => (15, 50),
            CcMarketManager => (16, 2),
            CcDivision => (17, 2),
            CcDivisionName => (18, 2),
            CcCompany => (19, 2),
            CcCompanyName => (20, 2),
            CcStreetNumber => (21, 0),
            CcStreetName => (22, 0),
            CcStreetType => (23, 0),
            CcSuiteNumber => (24, 0),
            CcCity => (25, 0),
            CcCounty => (26, 0),
            CcState => (27, 0),
            CcZip => (28, 0),
            CcCountry => (29, 0),
            CcGmtOffset => (30, 0),
            CcAddress => (31, 15),
            CcTaxPercentage => (32, 1),
            CcScd => (33, 1),
            CcNulls => (34, 2),
        }
    }
}

impl GeneratorColumn for CallCenterGeneratorColumn {
    fn get_table(&self) -> Table {
        Table::CallCenter
    }

    fn get_global_column_number(&self) -> i32 {
        self.get_column_info().0
    }

    fn get_seeds_per_row(&self) -> u32 {
        self.get_column_info().1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_center_generator_column_basics() {
        let column = CallCenterGeneratorColumn::CcCallCenterSk;
        assert_eq!(column.get_table(), Table::CallCenter);
        assert_eq!(column.get_global_column_number(), 1);
        assert_eq!(column.get_seeds_per_row(), 0);
    }

    #[test]
    fn test_generator_column_count() {
        let columns = CallCenterGeneratorColumn::values();
        assert_eq!(columns.len(), 34); // 31 regular columns + 3 special generator columns
    }

    #[test]
    fn test_global_column_numbers() {
        let columns = CallCenterGeneratorColumn::values();

        // Test first few columns match Java exactly
        assert_eq!(columns[0].get_global_column_number(), 1); // CcCallCenterSk
        assert_eq!(columns[1].get_global_column_number(), 2); // CcCallCenterId
        assert_eq!(columns[2].get_global_column_number(), 3); // CcRecStartDateId

        // Test last column
        assert_eq!(columns[33].get_global_column_number(), 34); // CcNulls
    }

    #[test]
    fn test_seeds_per_row() {
        // Test some key columns with different seed counts
        assert_eq!(
            CallCenterGeneratorColumn::CcCallCenterSk.get_seeds_per_row(),
            0
        );
        assert_eq!(
            CallCenterGeneratorColumn::CcCallCenterId.get_seeds_per_row(),
            15
        );
        assert_eq!(
            CallCenterGeneratorColumn::CcRecStartDateId.get_seeds_per_row(),
            10
        );
        assert_eq!(
            CallCenterGeneratorColumn::CcMarketClass.get_seeds_per_row(),
            50
        );
        assert_eq!(CallCenterGeneratorColumn::CcAddress.get_seeds_per_row(), 15);
    }

    #[test]
    fn test_special_generator_columns() {
        // Test columns that exist in generator but not in regular Column enum
        let address_col = CallCenterGeneratorColumn::CcAddress;
        assert_eq!(address_col.get_global_column_number(), 31);
        assert_eq!(address_col.get_seeds_per_row(), 15);

        let scd_col = CallCenterGeneratorColumn::CcScd;
        assert_eq!(scd_col.get_global_column_number(), 33);
        assert_eq!(scd_col.get_seeds_per_row(), 1);

        let nulls_col = CallCenterGeneratorColumn::CcNulls;
        assert_eq!(nulls_col.get_global_column_number(), 34);
        assert_eq!(nulls_col.get_seeds_per_row(), 2);
    }

    #[test]
    fn test_all_columns_have_table() {
        for column in CallCenterGeneratorColumn::values() {
            assert_eq!(column.get_table(), Table::CallCenter);
        }
    }

    #[test]
    fn test_column_ordering() {
        let columns = CallCenterGeneratorColumn::values();

        // Verify global column numbers are sequential
        for (i, column) in columns.iter().enumerate() {
            assert_eq!(column.get_global_column_number(), (i + 1) as i32);
        }
    }

    #[test]
    fn test_debug_display() {
        let column = CallCenterGeneratorColumn::CcCallCenterSk;
        let debug_str = format!("{:?}", column);
        assert_eq!(debug_str, "CcCallCenterSk");
    }

    #[test]
    fn test_generator_vs_regular_columns() {
        // Generator columns should include some columns not in regular Column enum
        let generator_count = CallCenterGeneratorColumn::values().len();

        // We know regular CallCenterColumn has 31 columns, generator has 34
        // The extra ones are CcAddress, CcScd, CcNulls
        assert_eq!(generator_count, 34);

        // Verify the extra columns exist
        let has_address = CallCenterGeneratorColumn::values()
            .iter()
            .any(|&col| matches!(col, CallCenterGeneratorColumn::CcAddress));
        let has_scd = CallCenterGeneratorColumn::values()
            .iter()
            .any(|&col| matches!(col, CallCenterGeneratorColumn::CcScd));
        let has_nulls = CallCenterGeneratorColumn::values()
            .iter()
            .any(|&col| matches!(col, CallCenterGeneratorColumn::CcNulls));

        assert!(has_address);
        assert!(has_scd);
        assert!(has_nulls);
    }
}
