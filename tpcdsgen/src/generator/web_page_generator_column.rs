use crate::column::Table;
use crate::generator::GeneratorColumn;

/// Generator columns for the WEB_PAGE table (WebPageGeneratorColumn)
/// Maps to the Java enum with the same name
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WebPageGeneratorColumn {
    WpPageSk,
    WpPageId,
    WpRecStartDateId,
    WpRecEndDateId,
    WpCreationDateSk,
    WpAccessDateSk,
    WpAutogenFlag,
    WpCustomerSk,
    WpUrl,
    WpType,
    WpCharCount,
    WpLinkCount,
    WpImageCount,
    WpMaxAdCount,
    WpNulls,
    WpScd,
}

impl GeneratorColumn for WebPageGeneratorColumn {
    fn get_table(&self) -> Table {
        Table::WebPage
    }

    fn get_global_column_number(&self) -> i32 {
        match self {
            Self::WpPageSk => 367,
            Self::WpPageId => 368,
            Self::WpRecStartDateId => 369,
            Self::WpRecEndDateId => 370,
            Self::WpCreationDateSk => 371,
            Self::WpAccessDateSk => 372,
            Self::WpAutogenFlag => 373,
            Self::WpCustomerSk => 374,
            Self::WpUrl => 375,
            Self::WpType => 376,
            Self::WpCharCount => 377,
            Self::WpLinkCount => 378,
            Self::WpImageCount => 379,
            Self::WpMaxAdCount => 380,
            Self::WpNulls => 381,
            Self::WpScd => 382,
        }
    }

    fn get_seeds_per_row(&self) -> u32 {
        match self {
            Self::WpCreationDateSk => 2,
            Self::WpNulls => 2,
            _ => 1,
        }
    }
}

impl WebPageGeneratorColumn {
    /// Get all generator column values (for Table integration)
    pub fn values() -> Vec<WebPageGeneratorColumn> {
        vec![
            Self::WpPageSk,
            Self::WpPageId,
            Self::WpRecStartDateId,
            Self::WpRecEndDateId,
            Self::WpCreationDateSk,
            Self::WpAccessDateSk,
            Self::WpAutogenFlag,
            Self::WpCustomerSk,
            Self::WpUrl,
            Self::WpType,
            Self::WpCharCount,
            Self::WpLinkCount,
            Self::WpImageCount,
            Self::WpMaxAdCount,
            Self::WpNulls,
            Self::WpScd,
        ]
    }
}
