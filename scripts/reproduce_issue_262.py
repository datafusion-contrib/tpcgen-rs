#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "polars>=1.35",
#   "pyarrow>=18",
#   "pyiceberg[sql-sqlite]>=0.9",
# ]
# ///

import argparse
import tempfile
from pathlib import Path

import polars as pl
import pyarrow.parquet as pq
from pyiceberg.catalog.sql import SqlCatalog


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Reproduce issue #262 using a tpcgen-cli lineitem.parquet file"
    )
    parser.add_argument("lineitem", type=Path)
    args = parser.parse_args()

    lineitem = args.lineitem.resolve()
    if not lineitem.is_file():
        parser.error(f"file not found: {lineitem}")

    with tempfile.TemporaryDirectory() as temp_dir:
        root = Path(temp_dir)
        catalog = SqlCatalog(
            "issue_262",
            uri=f"sqlite:///{root / 'catalog.db'}",
            warehouse=f"file://{root / 'warehouse'}",
        )
        catalog.create_namespace("tpch")
        table = catalog.create_table(
            "tpch.lineitem",
            schema=pq.read_schema(lineitem),
        )
        table.add_files([str(lineitem)])

        print(pl.scan_iceberg(table).collect())


if __name__ == "__main__":
    main()
