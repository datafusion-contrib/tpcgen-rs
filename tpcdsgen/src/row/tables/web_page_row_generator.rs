use crate::config::{Session, Table as ConfigTable};
use crate::distribution::web_page_use_distribution::WebPageUseDistribution;
use crate::error::Result;
use crate::generator::WebPageGeneratorColumn;
use crate::join_key_utils::generate_join_key;
use crate::random::RandomValueGenerator;
use crate::row::{AbstractRowGenerator, RowGenerator, RowGeneratorResult, WebPageRow};
use crate::slowly_changing_dimension_utils::{
    compute_scd_key, get_value_for_slowly_changing_dimension,
};
use crate::table::Table;
use crate::types::Date;

/// Row generator for the WEB_PAGE table (WebPageRowGenerator)
/// Pattern 2: SCD table with slowly changing dimension logic
pub struct WebPageRowGenerator {
    abstract_generator: AbstractRowGenerator,
    previous_row: Option<WebPageRow>,
}

impl Default for WebPageRowGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl WebPageRowGenerator {
    const WP_AUTOGEN_PERCENT: i32 = 30;

    /// Create a new WebPageRowGenerator
    pub fn new() -> Self {
        Self {
            abstract_generator: AbstractRowGenerator::new(Table::WebPage),
            previous_row: None,
        }
    }

    /// Generate a WebPageRow with SCD logic following Java implementation
    fn generate_web_page_row(&mut self, row_number: u64, session: &Session) -> Result<WebPageRow> {
        let row_number_i64 = i64::try_from(row_number).expect("row number fits in i64");

        // Create null bit map
        let nulls_stream = self
            .abstract_generator
            .get_random_number_stream(&WebPageGeneratorColumn::WpNulls);
        let threshold = RandomValueGenerator::generate_uniform_random_int(0, 9999, nulls_stream);
        let bit_map = RandomValueGenerator::generate_uniform_random_int(1, i32::MAX, nulls_stream);

        let null_bit_map = if threshold < Table::WebPage.get_null_basis_points() {
            (bit_map as i64) & !Table::WebPage.get_not_null_bit_map()
        } else {
            0
        };

        let wp_page_sk = row_number_i64;

        // Compute SCD key information
        let scd_key = compute_scd_key(Table::WebPage, row_number);
        let wp_page_id = scd_key.get_business_key().to_string();
        let wp_rec_start_date_id = scd_key.get_start_date();
        let wp_rec_end_date_id = scd_key.get_end_date();
        let is_new_key = scd_key.is_new_business_key();

        // Get field change flags for SCD
        let mut field_change_flags = self
            .abstract_generator
            .get_random_number_stream(&WebPageGeneratorColumn::WpScd)
            .next_random() as i32;

        // wp_creation_date_sk - join to DATE_DIM
        let mut wp_creation_date_sk = generate_join_key(
            &WebPageGeneratorColumn::WpCreationDateSk,
            self.abstract_generator
                .get_random_number_stream(&WebPageGeneratorColumn::WpCreationDateSk),
            ConfigTable::DateDim,
            row_number_i64,
            session.get_scaling(),
        )?;
        if let Some(prev) = &self.previous_row {
            wp_creation_date_sk = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_key,
                prev.get_wp_creation_date_sk(),
                wp_creation_date_sk,
            );
        }
        field_change_flags >>= 1;

        // wp_access_date_sk - today's date minus last access days
        let last_access = RandomValueGenerator::generate_uniform_random_int(
            0,
            100,
            self.abstract_generator
                .get_random_number_stream(&WebPageGeneratorColumn::WpAccessDateSk),
        );
        let mut wp_access_date_sk = Date::JULIAN_TODAYS_DATE as i64 - last_access as i64;
        if let Some(prev) = &self.previous_row {
            wp_access_date_sk = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_key,
                prev.get_wp_access_date_sk(),
                wp_access_date_sk,
            );
        }
        field_change_flags >>= 1;

        // wp_autogen_flag - 30% chance of auto-generated
        let random_int = RandomValueGenerator::generate_uniform_random_int(
            0,
            99,
            self.abstract_generator
                .get_random_number_stream(&WebPageGeneratorColumn::WpAutogenFlag),
        );
        let mut wp_autogen_flag = random_int < Self::WP_AUTOGEN_PERCENT;
        if let Some(prev) = &self.previous_row {
            wp_autogen_flag = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_key,
                prev.get_wp_autogen_flag(),
                wp_autogen_flag,
            );
        }
        field_change_flags >>= 1;

        // wp_customer_sk - join to CUSTOMER
        let mut wp_customer_sk = generate_join_key(
            &WebPageGeneratorColumn::WpCustomerSk,
            self.abstract_generator
                .get_random_number_stream(&WebPageGeneratorColumn::WpCustomerSk),
            ConfigTable::Customer,
            1,
            session.get_scaling(),
        )?;
        if let Some(prev) = &self.previous_row {
            wp_customer_sk = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_key,
                prev.get_wp_customer_sk(),
                wp_customer_sk,
            );
        }
        field_change_flags >>= 1;

        // wp_url - always returns the same value, so no need to check if it should change
        let wp_url = RandomValueGenerator::generate_random_url(
            self.abstract_generator
                .get_random_number_stream(&WebPageGeneratorColumn::WpUrl),
        );
        field_change_flags >>= 1;

        // wp_type - always uses a new value due to a bug in the C code
        let wp_type = WebPageUseDistribution::pick_random_web_page_use_type(
            self.abstract_generator
                .get_random_number_stream(&WebPageGeneratorColumn::WpType),
        )?;
        field_change_flags >>= 1;

        // wp_link_count
        let mut wp_link_count = RandomValueGenerator::generate_uniform_random_int(
            2,
            25,
            self.abstract_generator
                .get_random_number_stream(&WebPageGeneratorColumn::WpLinkCount),
        );
        if let Some(prev) = &self.previous_row {
            wp_link_count = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_key,
                prev.get_wp_link_count(),
                wp_link_count,
            );
        }
        field_change_flags >>= 1;

        // wp_image_count
        let mut wp_image_count = RandomValueGenerator::generate_uniform_random_int(
            1,
            7,
            self.abstract_generator
                .get_random_number_stream(&WebPageGeneratorColumn::WpImageCount),
        );
        if let Some(prev) = &self.previous_row {
            wp_image_count = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_key,
                prev.get_wp_image_count(),
                wp_image_count,
            );
        }
        field_change_flags >>= 1;

        // wp_max_ad_count
        let mut wp_max_ad_count = RandomValueGenerator::generate_uniform_random_int(
            0,
            4,
            self.abstract_generator
                .get_random_number_stream(&WebPageGeneratorColumn::WpMaxAdCount),
        );
        if let Some(prev) = &self.previous_row {
            wp_max_ad_count = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_key,
                prev.get_wp_max_ad_count(),
                wp_max_ad_count,
            );
        }
        field_change_flags >>= 1;

        // wp_char_count - calculated based on link and image counts
        let mut wp_char_count = RandomValueGenerator::generate_uniform_random_int(
            wp_link_count * 125 + wp_image_count * 50,
            wp_link_count * 300 + wp_image_count * 150,
            self.abstract_generator
                .get_random_number_stream(&WebPageGeneratorColumn::WpCharCount),
        );
        if let Some(prev) = &self.previous_row {
            wp_char_count = get_value_for_slowly_changing_dimension(
                field_change_flags,
                is_new_key,
                prev.get_wp_char_count(),
                wp_char_count,
            );
        }

        // Store current row for next iteration (before creating output row)
        self.previous_row = Some(WebPageRow::new(
            null_bit_map,
            wp_page_sk,
            wp_page_id.clone(),
            wp_rec_start_date_id,
            wp_rec_end_date_id,
            wp_creation_date_sk,
            wp_access_date_sk,
            wp_autogen_flag,
            wp_customer_sk,
            wp_url.clone(),
            wp_type.clone(),
            wp_char_count,
            wp_link_count,
            wp_image_count,
            wp_max_ad_count,
        ));

        // Return row with wp_customer_sk set to -1 if not autogenerated (line 155 in Java)
        Ok(WebPageRow::new(
            null_bit_map,
            wp_page_sk,
            wp_page_id,
            wp_rec_start_date_id,
            wp_rec_end_date_id,
            wp_creation_date_sk,
            wp_access_date_sk,
            wp_autogen_flag,
            if wp_autogen_flag { wp_customer_sk } else { -1 },
            wp_url,
            wp_type,
            wp_char_count,
            wp_link_count,
            wp_image_count,
            wp_max_ad_count,
        ))
    }
}

impl RowGenerator for WebPageRowGenerator {
    fn generate_row_and_child_rows(
        &mut self,
        row_number: u64,
        session: &Session,
        _parent_row_generator: Option<&mut dyn RowGenerator>,
        _child_row_generator: Option<&mut dyn RowGenerator>,
    ) -> Result<RowGeneratorResult> {
        let row = self.generate_web_page_row(row_number, session)?;
        Ok(RowGeneratorResult::new(row))
    }

    fn consume_remaining_seeds_for_row(&mut self) {
        self.abstract_generator.consume_remaining_seeds_for_row();
    }

    fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64) {
        self.abstract_generator
            .skip_rows_until_starting_row_number(starting_row_number);
    }
}
