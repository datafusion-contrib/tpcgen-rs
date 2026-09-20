use crate::row::table_row::DatField;
use std::fmt;

/// Customer demographics table row (CustomerDemographicsRow)
#[derive(Debug, Clone)]
pub struct CustomerDemographicsRow {
    null_bit_map: i64,
    pub(crate) cd_demo_sk: i64,
    pub(crate) cd_gender: String,
    pub(crate) cd_marital_status: String,
    pub(crate) cd_education_status: String,
    pub(crate) cd_purchase_estimate: i32,
    pub(crate) cd_credit_rating: String,
    pub(crate) cd_dep_count: i32,
    pub(crate) cd_dep_employed_count: i32,
    pub(crate) cd_dep_college_count: i32,
}

impl CustomerDemographicsRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        null_bit_map: i64,
        cd_demo_sk: i64,
        cd_gender: String,
        cd_marital_status: String,
        cd_education_status: String,
        cd_purchase_estimate: i32,
        cd_credit_rating: String,
        cd_dep_count: i32,
        cd_dep_employed_count: i32,
        cd_dep_college_count: i32,
    ) -> Self {
        CustomerDemographicsRow {
            null_bit_map,
            cd_demo_sk,
            cd_gender,
            cd_marital_status,
            cd_education_status,
            cd_purchase_estimate,
            cd_credit_rating,
            cd_dep_count,
            cd_dep_employed_count,
            cd_dep_college_count,
        }
    }

    pub fn null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    pub fn get_cd_demo_sk(&self) -> i64 {
        self.cd_demo_sk
    }

    pub fn get_cd_gender(&self) -> &str {
        &self.cd_gender
    }

    pub fn get_cd_marital_status(&self) -> &str {
        &self.cd_marital_status
    }

    pub fn get_cd_education_status(&self) -> &str {
        &self.cd_education_status
    }

    pub fn get_cd_purchase_estimate(&self) -> i32 {
        self.cd_purchase_estimate
    }

    pub fn get_cd_credit_rating(&self) -> &str {
        &self.cd_credit_rating
    }

    pub fn get_cd_dep_count(&self) -> i32 {
        self.cd_dep_count
    }

    pub fn get_cd_dep_employed_count(&self) -> i32 {
        self.cd_dep_employed_count
    }

    pub fn get_cd_dep_college_count(&self) -> i32 {
        self.cd_dep_college_count
    }

    /// Check if a column should be null based on the null bitmap (TableRowWithNulls logic)
    fn should_be_null(&self, column_position: i32) -> bool {
        ((self.null_bit_map >> column_position) & 1) == 1
    }
}

/// DAT field helper for this row's columns.
impl CustomerDemographicsRow {
    pub(crate) fn field<T>(&self, value: T, column_position: i32) -> DatField<T> {
        DatField::new(value, self.should_be_null(column_position))
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces one
/// `|`-terminated field per column.
impl fmt::Display for CustomerDemographicsRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|",
            self.field(self.cd_demo_sk, 0),
            self.field(&self.cd_gender, 1),
            self.field(&self.cd_marital_status, 2),
            self.field(&self.cd_education_status, 3),
            self.field(self.cd_purchase_estimate, 4),
            self.field(&self.cd_credit_rating, 5),
            self.field(self.cd_dep_count, 6),
            self.field(self.cd_dep_employed_count, 7),
            self.field(self.cd_dep_college_count, 8),
        )
    }
}
