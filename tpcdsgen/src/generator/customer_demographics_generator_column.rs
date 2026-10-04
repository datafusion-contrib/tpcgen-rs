use crate::column::Table;
use crate::generator::GeneratorColumn;

/// Generator columns for CUSTOMER_DEMOGRAPHICS table (CustomerDemographicsGeneratorColumn)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomerDemographicsGeneratorColumn {
    CdDemoSk,
    CdGender,
    CdMaritalStatus,
    CdEducationStatus,
    CdPurchaseEstimate,
    CdCreditRating,
    CdDepCount,
    CdDepEmployedCount,
    CdDepCollegeCount,
    CdNulls,
}

impl CustomerDemographicsGeneratorColumn {
    /// Get all generator columns in order (values())
    pub fn values() -> &'static [CustomerDemographicsGeneratorColumn] {
        use CustomerDemographicsGeneratorColumn::*;
        static VALUES: &[CustomerDemographicsGeneratorColumn] = &[
            CdDemoSk,
            CdGender,
            CdMaritalStatus,
            CdEducationStatus,
            CdPurchaseEstimate,
            CdCreditRating,
            CdDepCount,
            CdDepEmployedCount,
            CdDepCollegeCount,
            CdNulls,
        ];
        VALUES
    }
}

impl GeneratorColumn for CustomerDemographicsGeneratorColumn {
    fn get_global_column_number(&self) -> i32 {
        match self {
            Self::CdDemoSk => 149,
            Self::CdGender => 150,
            Self::CdMaritalStatus => 151,
            Self::CdEducationStatus => 152,
            Self::CdPurchaseEstimate => 153,
            Self::CdCreditRating => 154,
            Self::CdDepCount => 155,
            Self::CdDepEmployedCount => 156,
            Self::CdDepCollegeCount => 157,
            Self::CdNulls => 158,
        }
    }

    fn get_seeds_per_row(&self) -> u32 {
        match self {
            Self::CdDemoSk => 1,
            Self::CdGender => 1,
            Self::CdMaritalStatus => 1,
            Self::CdEducationStatus => 1,
            Self::CdPurchaseEstimate => 1,
            Self::CdCreditRating => 1,
            Self::CdDepCount => 1,
            Self::CdDepEmployedCount => 1,
            Self::CdDepCollegeCount => 1,
            Self::CdNulls => 2,
        }
    }

    fn get_table(&self) -> Table {
        Table::CustomerDemographics
    }
}
