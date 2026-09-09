use arrow::record_batch::RecordBatchReader;
use tpcdsgen::config::Session;
use tpcdsgen_arrow::{CatalogReturnsArrow, IncomeBandArrow, ReasonArrow, WebReturnsArrow};

#[test]
fn tpcds_schemas_use_canonical_column_names() {
    let session = Session::default();
    let schemas = [
        (
            CatalogReturnsArrow::new(session.clone()).schema(),
            "cr_return_amt_inc_tax",
            "cr_return_amount_inc_tax",
        ),
        (
            IncomeBandArrow::new(session.clone()).schema(),
            "ib_income_band_sk",
            "ib_income_band_id",
        ),
        (
            ReasonArrow::new(session.clone()).schema(),
            "r_reason_desc",
            "r_reason_description",
        ),
        (
            WebReturnsArrow::new(session).schema(),
            "wr_account_credit",
            "wr_store_credit",
        ),
    ];

    for (schema, canonical, obsolete) in schemas {
        assert!(
            schema.field_with_name(canonical).is_ok(),
            "missing canonical column {canonical}"
        );
        assert!(
            schema.field_with_name(obsolete).is_err(),
            "obsolete column {obsolete} remains in schema"
        );
    }
}
