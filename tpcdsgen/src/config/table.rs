use crate::TpcdsError;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Table {
    CallCenter,
    CatalogPage,
    CatalogReturns,
    CatalogSales,
    Customer,
    CustomerAddress,
    CustomerDemographics,
    DateDim,
    HouseholdDemographics,
    IncomeBand,
    Inventory,
    Item,
    Promotion,
    Reason,
    ShipMode,
    Store,
    StoreReturns,
    StoreSales,
    TimeDim,
    Warehouse,
    WebPage,
    WebReturns,
    WebSales,
    WebSite,
    DbgenVersion,
}

impl Table {
    /// Get the table name as used in command line
    pub fn get_name(&self) -> &'static str {
        match self {
            Table::CallCenter => "call_center",
            Table::CatalogPage => "catalog_page",
            Table::CatalogReturns => "catalog_returns",
            Table::CatalogSales => "catalog_sales",
            Table::Customer => "customer",
            Table::CustomerAddress => "customer_address",
            Table::CustomerDemographics => "customer_demographics",
            Table::DateDim => "date_dim",
            Table::HouseholdDemographics => "household_demographics",
            Table::IncomeBand => "income_band",
            Table::Inventory => "inventory",
            Table::Item => "item",
            Table::Promotion => "promotion",
            Table::Reason => "reason",
            Table::ShipMode => "ship_mode",
            Table::Store => "store",
            Table::StoreReturns => "store_returns",
            Table::StoreSales => "store_sales",
            Table::TimeDim => "time_dim",
            Table::Warehouse => "warehouse",
            Table::WebPage => "web_page",
            Table::WebReturns => "web_returns",
            Table::WebSales => "web_sales",
            Table::WebSite => "web_site",
            Table::DbgenVersion => "dbgen_version",
        }
    }

    /// All TPC-DS tables
    pub fn values() -> &'static [Table] {
        &[
            Table::CallCenter,
            Table::CatalogPage,
            Table::CatalogReturns,
            Table::CatalogSales,
            Table::Customer,
            Table::CustomerAddress,
            Table::CustomerDemographics,
            Table::DateDim,
            Table::HouseholdDemographics,
            Table::IncomeBand,
            Table::Inventory,
            Table::Item,
            Table::Promotion,
            Table::Reason,
            Table::ShipMode,
            Table::Store,
            Table::StoreReturns,
            Table::StoreSales,
            Table::TimeDim,
            Table::Warehouse,
            Table::WebPage,
            Table::WebReturns,
            Table::WebSales,
            Table::WebSite,
            Table::DbgenVersion,
        ]
    }

    pub fn keeps_history(&self) -> bool {
        // Tables that maintain historical data
        matches!(
            self,
            Table::CallCenter | Table::Item | Table::Store | Table::WebPage | Table::WebSite
        )
    }
}

impl FromStr for Table {
    type Err = TpcdsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let table_name = s.to_uppercase();
        match table_name.as_str() {
            "CALL_CENTER" => Ok(Table::CallCenter),
            "CATALOG_PAGE" => Ok(Table::CatalogPage),
            "CATALOG_RETURNS" => Ok(Table::CatalogReturns),
            "CATALOG_SALES" => Ok(Table::CatalogSales),
            "CUSTOMER" => Ok(Table::Customer),
            "CUSTOMER_ADDRESS" => Ok(Table::CustomerAddress),
            "CUSTOMER_DEMOGRAPHICS" => Ok(Table::CustomerDemographics),
            "DATE_DIM" => Ok(Table::DateDim),
            "HOUSEHOLD_DEMOGRAPHICS" => Ok(Table::HouseholdDemographics),
            "INCOME_BAND" => Ok(Table::IncomeBand),
            "INVENTORY" => Ok(Table::Inventory),
            "ITEM" => Ok(Table::Item),
            "PROMOTION" => Ok(Table::Promotion),
            "REASON" => Ok(Table::Reason),
            "SHIP_MODE" => Ok(Table::ShipMode),
            "STORE" => Ok(Table::Store),
            "STORE_RETURNS" => Ok(Table::StoreReturns),
            "STORE_SALES" => Ok(Table::StoreSales),
            "TIME_DIM" => Ok(Table::TimeDim),
            "WAREHOUSE" => Ok(Table::Warehouse),
            "WEB_PAGE" => Ok(Table::WebPage),
            "WEB_RETURNS" => Ok(Table::WebReturns),
            "WEB_SALES" => Ok(Table::WebSales),
            "WEB_SITE" => Ok(Table::WebSite),
            "DBGEN_VERSION" => Ok(Table::DbgenVersion),
            _ => Err(TpcdsError::new(&format!("Invalid table name: {}", s))),
        }
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_from_str() {
        assert_eq!(
            "CATALOG_SALES".parse::<Table>().unwrap(),
            Table::CatalogSales
        );
        assert_eq!(
            "catalog_sales".parse::<Table>().unwrap(),
            Table::CatalogSales
        );
        assert_eq!("STORE_SALES".parse::<Table>().unwrap(), Table::StoreSales);

        assert!("INVALID_TABLE".parse::<Table>().is_err());
    }

    #[test]
    fn test_table_name() {
        assert_eq!(Table::CatalogSales.get_name(), "catalog_sales");
        assert_eq!(Table::StoreSales.get_name(), "store_sales");
    }

    #[test]
    fn test_values() {
        let tables = Table::values();
        assert!(tables.contains(&Table::CatalogSales));
        assert!(tables.contains(&Table::StoreSales));
        assert_eq!(tables.len(), 25);
    }

    #[test]
    fn test_keeps_history() {
        assert!(Table::CallCenter.keeps_history());
        assert!(!Table::CatalogSales.keeps_history());
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Table::CatalogSales), "catalog_sales");
    }
}
