# tpcgen-cli

`tpcgen-cli` is a high-performance, parallel command line interface for
generating [TPC-H] and [TPC-DS] data.

> **Note:** See [`tpcgen-cli`] to create both TPC-H and TPC-DS data and the
> [tpcgen-rs project] for the full project details.

[TPC-H]: https://www.tpc.org/tpch/
[`tpcgen-cli`]: https://github.com/datafusion-contrib/tpcgen-rs/blob/main/tpcgen-cli/README.md
[tpcgen-rs project]: https://github.com/datafusion-contrib/tpcgen-rs
`tpcgen-cli` is a high performance, parallel command line interface for generating TPC-H
and TPC-DS benchmark data.

## Try with `uvx`

```shell
uvx tpcgen-cli tpch parquet -s 1 --output-dir /tmp/tpch
```

![Running tpcgen-cli TPC-DS](https://raw.githubusercontent.com/datafusion-contrib/tpcgen-rs/main/tpcgen-cli/tpcgen-cli-tpcds.gif)
![Running tpcgen-cli TPC-H](https://raw.githubusercontent.com/datafusion-contrib/tpcgen-rs/main/tpcgen-cli/tpcgen-cli-tpch.gif)


## Install with `pip`

```shell
python -m pip install tpcgen-cli
```

## Examples

```shell
# TPC-H Scale Factor 1, all tables, in `tbl` (pipe-delimited) format in the `/tmp/tpch` directory
# Note: `tpcgen-cli tpch tbl` also works explicitly
tpcgen-cli tpch -s 1 --output-dir /tmp/tpch

# TPC-H Scale Factor 1, all tables, in CSV format
tpcgen-cli tpch csv -s 1 --output-dir /tmp/tpch

# TPC-DS Scale Factor 10, all tables, in `dat` (pipe-delimited) format 
# 10 part(itions) written to `/tmp/tpcds/<table_name>/<table_name>.<part>.parquet
tpcgen-cli tpcds parquet -s 10 --parts 10 --output-dir /tmp/tpcds

# TPC-DS Scale Factor 100, store_sales table, in Apache Parquet format,
# 10 part(itions) written to `/tmp/tpcds/store_sales/store_sales.<part>.parquet`
tpcgen-cli tpcds parquet -s 100 --tables store_sales --parts 10 --output-dir /tmp/tpcds

# TPC-C Scale Factor 1, Per-column encodings 
# (overrides Parquet writer defaults for named columns)
tpcgen-cli tpch parquet -s 1 --tables lineitem --column-encoding=l_comment=DELTA_LENGTH_BYTE_ARRAY --output-dir /tmp/tpch

# TPC-DS Scale Factor 1, all tables, in CSV format in the `/tmp/tpcds` directory
# Note: `tpcgen-cli tpcds` with no format writes `dat` (pipe-delimited) files
tpcgen-cli tpcds csv -s 1 --output-dir /tmp/tpcds
```
