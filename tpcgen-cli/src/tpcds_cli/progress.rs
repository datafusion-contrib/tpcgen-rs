//! Progress registration shared by the TPC-DS row-generator outputs.
//!
//! The DAT and CSV outputs both drive the row generators directly and pair
//! sales tables with their returns table, so they register progress the same
//! way. Keeping that in one place stops the two from drifting.

use crate::progress::{ProgressHandle, ProgressTracker};
use std::sync::Arc;
use tpcdsgen::config::{Session, Table};

/// Progress handles for one requested table.
///
/// Sales tables are generated together with their returns table, so they
/// register two handles; the returns tables themselves register none.
#[derive(Debug)]
pub(super) enum TableProgress {
    None,
    Single(ProgressHandle),
    Paired {
        sales: ProgressHandle,
        returns: ProgressHandle,
    },
}

/// Register progress for one requested table.
pub(super) fn register_table(
    table: Table,
    session: &Session,
    progress: Arc<dyn ProgressTracker>,
) -> TableProgress {
    let register = |table: Table| {
        let row_count = session.get_scaling().get_row_count(table);
        progress.clone().register(table.get_name(), row_count)
    };

    match table {
        Table::StoreSales => TableProgress::Paired {
            sales: register(Table::StoreSales),
            returns: register(Table::StoreReturns),
        },
        Table::CatalogSales => TableProgress::Paired {
            sales: register(Table::CatalogSales),
            returns: register(Table::CatalogReturns),
        },
        Table::WebSales => TableProgress::Paired {
            sales: register(Table::WebSales),
            returns: register(Table::WebReturns),
        },
        Table::StoreReturns | Table::CatalogReturns | Table::WebReturns => TableProgress::None,
        _ => TableProgress::Single(register(table)),
    }
}
