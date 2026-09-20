use crate::row::table_row::DatField;
use crate::types::Date;
use std::fmt;

/// Row structure for the WEB_PAGE table (WebPageRow)
#[derive(Debug, Clone)]
pub struct WebPageRow {
    null_bit_map: i64,
    pub(crate) wp_page_sk: i64,
    pub(crate) wp_page_id: String,
    pub(crate) wp_rec_start_date_id: i64,
    pub(crate) wp_rec_end_date_id: i64,
    pub(crate) wp_creation_date_sk: i64,
    pub(crate) wp_access_date_sk: i64,
    pub(crate) wp_autogen_flag: bool,
    pub(crate) wp_customer_sk: i64,
    pub(crate) wp_url: String,
    pub(crate) wp_type: String,
    pub(crate) wp_char_count: i32,
    pub(crate) wp_link_count: i32,
    pub(crate) wp_image_count: i32,
    pub(crate) wp_max_ad_count: i32,
}

impl WebPageRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        null_bit_map: i64,
        wp_page_sk: i64,
        wp_page_id: String,
        wp_rec_start_date_id: i64,
        wp_rec_end_date_id: i64,
        wp_creation_date_sk: i64,
        wp_access_date_sk: i64,
        wp_autogen_flag: bool,
        wp_customer_sk: i64,
        wp_url: String,
        wp_type: String,
        wp_char_count: i32,
        wp_link_count: i32,
        wp_image_count: i32,
        wp_max_ad_count: i32,
    ) -> Self {
        Self {
            null_bit_map,
            wp_page_sk,
            wp_page_id,
            wp_rec_start_date_id,
            wp_rec_end_date_id,
            wp_creation_date_sk,
            wp_access_date_sk,
            wp_autogen_flag,
            wp_customer_sk,
            wp_url,
            wp_type,
            wp_char_count,
            wp_link_count,
            wp_image_count,
            wp_max_ad_count,
        }
    }

    // Getters for SCD comparison (needed in WebPageRowGenerator)
    pub fn get_wp_creation_date_sk(&self) -> i64 {
        self.wp_creation_date_sk
    }

    pub fn get_wp_access_date_sk(&self) -> i64 {
        self.wp_access_date_sk
    }

    pub fn get_wp_autogen_flag(&self) -> bool {
        self.wp_autogen_flag
    }

    pub fn get_wp_customer_sk(&self) -> i64 {
        self.wp_customer_sk
    }

    pub fn get_wp_char_count(&self) -> i32 {
        self.wp_char_count
    }

    pub fn get_wp_link_count(&self) -> i32 {
        self.wp_link_count
    }

    pub fn get_wp_image_count(&self) -> i32 {
        self.wp_image_count
    }

    pub fn get_wp_max_ad_count(&self) -> i32 {
        self.wp_max_ad_count
    }

    /// Check if a column should be null based on the null bit map (shouldBeNull)
    pub(crate) fn should_be_null(&self, column_position: i32) -> bool {
        (self.null_bit_map & (1 << column_position)) != 0
    }

    pub fn null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    pub fn get_wp_page_sk(&self) -> i64 {
        self.wp_page_sk
    }

    pub fn get_wp_page_id(&self) -> &str {
        &self.wp_page_id
    }

    pub fn get_wp_rec_start_date_id(&self) -> i64 {
        self.wp_rec_start_date_id
    }

    pub fn get_wp_rec_end_date_id(&self) -> i64 {
        self.wp_rec_end_date_id
    }

    pub fn get_wp_url(&self) -> &str {
        &self.wp_url
    }

    pub fn get_wp_type(&self) -> &str {
        &self.wp_type
    }
}

impl WebPageRow {
    /// DAT field for a surrogate key: NULL when the null bit is set or the
    /// key is -1.
    pub(crate) fn key_field(&self, value: i64, column_position: i32) -> DatField<i64> {
        DatField::new(value, self.should_be_null(column_position) || value == -1)
    }

    /// DAT field for a regular value: NULL when the null bit is set.
    pub(crate) fn field<T>(&self, value: T, column_position: i32) -> DatField<T> {
        DatField::new(value, self.should_be_null(column_position))
    }

    /// DAT field for an SCD date: NULL when the null bit is set or the
    /// julian day is negative.
    pub(crate) fn date_field(&self, julian_days: i64, column_position: i32) -> DatField<Date> {
        DatField::from(
            (!(self.should_be_null(column_position) || julian_days < 0))
                .then(|| Date::from_julian_days(julian_days as i32)),
        )
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces one
/// `|`-terminated field per column.
impl fmt::Display for WebPageRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|",
            self.key_field(self.wp_page_sk, 0),
            self.field(&self.wp_page_id, 1),
            self.date_field(self.wp_rec_start_date_id, 2),
            self.date_field(self.wp_rec_end_date_id, 3),
            self.key_field(self.wp_creation_date_sk, 4),
            self.key_field(self.wp_access_date_sk, 5),
            DatField::yes_no(self.wp_autogen_flag, self.should_be_null(6)),
            self.key_field(self.wp_customer_sk, 7),
            self.field(&self.wp_url, 8),
            self.field(&self.wp_type, 9),
            self.field(self.wp_char_count, 10),
            self.field(self.wp_link_count, 11),
            self.field(self.wp_image_count, 12),
            self.field(self.wp_max_ad_count, 13),
        )
    }
}
