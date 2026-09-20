use crate::row::table_row::DatField;
use std::fmt;

/// Ship mode table row (ShipModeRow)
#[derive(Debug, Clone)]
pub struct ShipModeRow {
    null_bit_map: i64,
    pub(crate) sm_ship_mode_sk: i64,
    pub(crate) sm_ship_mode_id: String,
    pub(crate) sm_type: String,
    pub(crate) sm_code: String,
    pub(crate) sm_carrier: String,
    pub(crate) sm_contract: String,
}

impl ShipModeRow {
    pub fn new(
        null_bit_map: i64,
        sm_ship_mode_sk: i64,
        sm_ship_mode_id: String,
        sm_type: String,
        sm_code: String,
        sm_carrier: String,
        sm_contract: String,
    ) -> Self {
        ShipModeRow {
            null_bit_map,
            sm_ship_mode_sk,
            sm_ship_mode_id,
            sm_type,
            sm_code,
            sm_carrier,
            sm_contract,
        }
    }

    /// Check if a column should be null based on the null bitmap (TableRowWithNulls logic)
    pub(crate) fn should_be_null(&self, column_position: i32) -> bool {
        ((self.null_bit_map >> column_position) & 1) == 1
    }

    pub fn null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    pub fn get_sm_ship_mode_sk(&self) -> i64 {
        self.sm_ship_mode_sk
    }

    pub fn get_sm_ship_mode_id(&self) -> &str {
        &self.sm_ship_mode_id
    }

    pub fn get_sm_type(&self) -> &str {
        &self.sm_type
    }

    pub fn get_sm_code(&self) -> &str {
        &self.sm_code
    }

    pub fn get_sm_carrier(&self) -> &str {
        &self.sm_carrier
    }

    pub fn get_sm_contract(&self) -> &str {
        &self.sm_contract
    }
}

/// DAT field helper: NULL is driven purely by the null bit
/// (ship_mode applies no key sentinel check, only the null bit).
impl ShipModeRow {
    pub(crate) fn field<T>(&self, value: T, column_position: i32) -> DatField<T> {
        DatField::new(value, self.should_be_null(column_position))
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces one
/// `|`-terminated field per column.
impl fmt::Display for ShipModeRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}|{}|{}|",
            self.field(self.sm_ship_mode_sk, 0),
            self.field(&self.sm_ship_mode_id, 1),
            self.field(&self.sm_type, 2),
            self.field(&self.sm_code, 3),
            self.field(&self.sm_carrier, 4),
            self.field(&self.sm_contract, 5),
        )
    }
}
