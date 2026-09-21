# tpcgen-cli

`tpcgen-cli` is a fast, parallel command line tool that generates both **TPC-H**
and **TPC-DS** benchmark data, in `tbl`/`dat`, CSV and [Apache Parquet] formats.

* See the [project README] for project details
* See [ARCHITECTURE.md] for how the crates in this workspace fit together
* See [BENCHMARKS.md] for performance details and benchmarking methodology

If you only need TPC-H data, [`tpchgen-cli`] remains available and is published
to crates.io and PyPI. It is a thin wrapper around this crate, so
`tpchgen-cli <args>` and `tpcgen-cli tpch <args>` generate identical data.

[Apache Parquet]: https://parquet.apache.org/
[project README]: https://github.com/datafusion-contrib/tpcgen-rs
[ARCHITECTURE.md]: https://github.com/datafusion-contrib/tpcgen-rs/blob/main/ARCHITECTURE.md
[BENCHMARKS.md]: https://github.com/datafusion-contrib/tpcgen-rs/blob/main/benchmarks/BENCHMARKS.md
[`tpchgen-cli`]: https://github.com/datafusion-contrib/tpcgen-rs/blob/main/tpchgen-cli/README.md

## Install

> **Note:** `tpcgen-cli` has not been released yet: it is not on PyPI, and
> crates.io only has a pre-release. The `uvx`, `pip`, and `cargo install
> tpcgen-cli` commands below start working with the v4.0.0 release, tracked in
> [#307]. Until then, install from a checkout of this repository (see
> [Install via Rust](#install-via-rust)), or use [`tpchgen-cli`] if you only
> need TPC-H data.

[#307]: https://github.com/datafusion-contrib/tpcgen-rs/issues/307

### Try with `uvx`

```shell
uvx tpcgen-cli tpcds parquet -s 1 --output-dir /tmp/tpcds
```

### Install with `pip`

```shell
python -m pip install tpcgen-cli
```

### Install via Rust

[Install Rust](https://www.rust-lang.org/tools/install) and compile:

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
RUSTFLAGS='-C target-cpu=native' cargo install --locked tpcgen-cli
```

Or, from a checkout of this repository:

```shell
RUSTFLAGS='-C target-cpu=native' cargo install --locked --path tpcgen-cli
```

## Usage

```shell
tpcgen-cli <tpch|tpcds> [<tbl|dat|csv|parquet>] [OPTIONS]
```

The benchmark (`tpch` or `tpcds`) comes first, then an optional output format
subcommand. When no format is given, TPC-H defaults to `tbl` and TPC-DS defaults
to `dat`.

Run `tpcgen-cli --help`, `tpcgen-cli tpch --help`, or `tpcgen-cli tpcds --help`
for the full set of options.

### TPC-H examples

```shell
# Scale factor 1, all tables, TBL (the default format)
tpcgen-cli tpch -s 1 --output-dir /tmp/tpch

# Scale factor 1, all tables, CSV with a tab delimiter
tpcgen-cli tpch csv -s 1 --delimiter='\t' --output-dir /tmp/tpch

# Scale factor 100, lineitem only, 10 Parquet files in /tmp/tpch/lineitem
tpcgen-cli tpch parquet -s 100 --tables lineitem --parts 10 --output-dir /tmp/tpch

# Per-column Parquet encodings (overrides the writer defaults for named columns)
tpcgen-cli tpch parquet -s 1 --tables lineitem \
  --column-encoding=l_comment=DELTA_LENGTH_BYTE_ARRAY --output-dir /tmp/tpch

# Write to stdout instead of files
tpcgen-cli tpch -s 1 --tables lineitem --stdout > lineitem.tbl
```

### TPC-DS examples

```shell
# Scale factor 1, all tables, DAT (the default format)
tpcgen-cli tpcds -s 1 --output-dir /tmp/tpcds

# Scale factor 1, all tables, CSV with a tab delimiter
tpcgen-cli tpcds csv -s 1 --delimiter='\t' --output-dir /tmp/tpcds

# Scale factor 100, store_sales only, 10 Parquet files in /tmp/tpcds/store_sales
tpcgen-cli tpcds parquet -s 100 --tables store_sales --parts 10 --output-dir /tmp/tpcds

# Match the C `dsdgen` reference implementation instead of the Java/Trino one
tpcgen-cli tpcds -s 1 --compat c --output-dir /tmp/tpcds
```

TPC-DS output is verified byte-for-byte against both the Java/Trino port of
`dsdgen` (`--compat trino`, the default) and the TPC-supplied C `dsdgen`
(`--compat c`). See [TESTING.md] and
[`scripts/tpcds/README.md`](scripts/tpcds/README.md) for details.

[TESTING.md]: https://github.com/datafusion-contrib/tpcgen-rs/blob/main/TESTING.md

## Progress bars

By default `tpcgen-cli` shows a per-table progress bar on stderr while data is
generated. Pass `--no-progress` to disable it. Bars are also disabled
automatically when `--quiet` is set, when `--stdout` is used, or when stderr is
not a terminal (for example in CI logs).
