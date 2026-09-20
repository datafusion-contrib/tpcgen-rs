use crate::row::table_row::DatField;
use crate::types::Address;
use std::fmt;

/// Warehouse table row (WarehouseRow)
#[derive(Debug, Clone)]
pub struct WarehouseRow {
    null_bit_map: i64,
    pub(crate) w_warehouse_sk: i64,
    pub(crate) w_warehouse_id: String,
    pub(crate) w_warehouse_name: String,
    pub(crate) w_warehouse_sq_ft: i32,
    pub(crate) w_address: Address,
}

impl WarehouseRow {
    pub fn new(
        null_bit_map: i64,
        w_warehouse_sk: i64,
        w_warehouse_id: String,
        w_warehouse_name: String,
        w_warehouse_sq_ft: i32,
        w_address: Address,
    ) -> Self {
        WarehouseRow {
            null_bit_map,
            w_warehouse_sk,
            w_warehouse_id,
            w_warehouse_name,
            w_warehouse_sq_ft,
            w_address,
        }
    }

    /// Check if a column should be null based on the null bitmap (TableRowWithNulls logic)
    pub(crate) fn should_be_null(&self, column_position: i32) -> bool {
        ((self.null_bit_map >> column_position) & 1) == 1
    }

    pub fn null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    pub fn get_w_warehouse_sk(&self) -> i64 {
        self.w_warehouse_sk
    }

    pub fn get_w_warehouse_id(&self) -> &str {
        &self.w_warehouse_id
    }

    pub fn get_w_warehouse_name(&self) -> &str {
        &self.w_warehouse_name
    }

    pub fn get_w_warehouse_sq_ft(&self) -> i32 {
        self.w_warehouse_sq_ft
    }

    pub fn get_w_address(&self) -> &Address {
        &self.w_address
    }
}

/// DAT field helper: NULL is driven purely by the null bit (warehouse
/// applies no key sentinel check).
impl WarehouseRow {
    pub(crate) fn field<T>(&self, value: T, column_position: i32) -> DatField<T> {
        DatField::new(value, self.should_be_null(column_position))
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces one
/// `|`-terminated field per column.
impl fmt::Display for WarehouseRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|",
            self.field(self.w_warehouse_sk, 0),
            self.field(&self.w_warehouse_id, 1),
            self.field(&self.w_warehouse_name, 2),
            self.field(self.w_warehouse_sq_ft, 3),
            self.field(self.w_address.get_street_number(), 4),
            self.field(self.w_address.get_street_name(), 5),
            self.field(self.w_address.get_street_type(), 6),
            self.field(self.w_address.get_suite_number(), 7),
            self.field(self.w_address.get_city(), 8),
            self.field(self.w_address.get_county().unwrap_or(""), 9),
            self.field(self.w_address.get_state(), 10),
            DatField::zip(self.w_address.get_zip(), self.should_be_null(11)),
            self.field(self.w_address.get_country(), 12),
            self.field(self.w_address.get_gmt_offset(), 13),
        )
    }
}
