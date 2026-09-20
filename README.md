# tpcgen-rs

> [!NOTE]
> Originally written by [@clflushopt](https://github.com/clflushopt) at
> [`clflushopt/tpchgen-rs`](https://github.com/clflushopt/tpchgen-rs).
> The project is now maintained at
> [`datafusion-contrib/tpcgen-rs`](https://github.com/datafusion-contrib/tpcgen-rs).

[![Apache licensed][license-badge]][license-url]
[![Build Status][actions-badge]][actions-url]

[license-badge]: https://img.shields.io/badge/license-Apache%20v2-blue.svg
[license-url]: https://github.com/datafusion-contrib/tpcgen-rs/blob/main/LICENSE
[actions-badge]: https://github.com/datafusion-contrib/tpcgen-rs/actions/workflows/rust.yml/badge.svg
[actions-url]: https://github.com/datafusion-contrib/tpcgen-rs/actions?query=branch%3Amain

Blazing fast [TPCH] benchmark data generator, in pure Rust with zero dependencies.

[TPCH]: https://www.tpc.org/tpch/

## Features

1. Blazing Speed 🚀
2. Obsessively Tested 📋
3. Fully parallel, streaming, constant memory usage 🧠

## Try it now

Try with `uvx`:

```shell
uvx tpchgen-cli parquet -s 1 --output-dir /tmp/tpch
```

![Running tpcgen-cli](tpcgen-cli-run.gif)

Install with `pip`:

```shell
python -m pip install tpchgen-cli
```

Then generate TPC-H data:

```shell
tpchgen-cli parquet -s 1 --output-dir /tmp/tpch
```

`tpchgen-cli` is a command-line program distributed as a Python package. See the
[`tpchgen-cli`] README for more install options and examples.

## Performance

[`tpchgen-cli`] is more than 10x faster than the next fastest TPCH generator we
know of. On a 2023 Mac M3 Max laptop, it easily generates data faster than can
be written to SSD. See [BENCHMARKS.md](./benchmarks/BENCHMARKS.md) for more
details on performance and benchmarking.

[`tpchgen-cli`]: ./tpchgen-cli/README.md

Times to create TPCH tables in Parquet format using `tpchgen-cli` and `duckdb` for various scale factors.

| Scale Factor | `tpchgen-cli` | DuckDB     | DuckDB (proprietary) |
| ------------ | ------------- | ---------- | -------------------- |
| 1            | `0:02.24`     | `0:12.29`  | `0:10.68`            |
| 10           | `0:09.97`     | `1:46.80`  | `1:41.14`            |
| 100          | `1:14.22`     | `17:48.27` | `16:40.88`           |
| 1000         | `10:26.26`    | N/A (OOM)  | N/A (OOM)            |

- DuckDB (proprietary) is the time required to create TPCH data using the
  proprietary DuckDB format
- Creating Scale Factor 1000 using DuckDB [required 647 GB of memory](https://duckdb.org/docs/stable/extensions/tpch.html#resource-usage-of-the-data-generator),
  which is why it is not included in the table above.

![Parquet Generation Performance](parquet-performance.png)


## Answers

The core `tpchgen` crate provides answers for queries 1 to 22 and for a scale factor
of 1. The answers exposed were derived from the [TPC-H Tools](https://www.tpc.org/)
official distribution.

## Testing

This crate has extensive tests to ensure correctness and produces exactly the
same, byte-for-byte output as the original [`dbgen`] implementation. We compare
the output of this crate with [`dbgen`] as part of every checkin. See
[TESTING.md](TESTING.md) for more details on testing methodology

## Crates

- [`tpchgen`](tpchgen): the core data generator logic for TPC-H. It has no
  dependencies and is easy to embed in other Rust project.

- [`tpchgen-arrow`](tpchgen-arrow) generates TPC-H data in [Apache Arrow]
  format. It depends on the arrow-rs library

- [`tpchgen-cli`](tpchgen-cli) is a [`dbgen`] compatible CLI tool that generates
  benchmark dataset using multiple processes.

[Apache Arrow]: https://arrow.apache.org/
[`dbgen`]: https://github.com/electrum/tpch-dbgen

## Contributing

Pull requests are welcome. For major changes, please open an issue first for
discussion. See our [contributors guide](CONTRIBUTING.md) for more details.

## Architecture

Please see [architecture guide](ARCHITECTURE.md) for details on how the code
is structured.

## License

The project is licensed under the [APACHE 2.0](LICENSE) license.

## References

- The TPC-H Specification, see the specification [page](https://www.tpc.org/tpc_documents_current_versions/current_specifications5.asp).
- The Original `dbgen` Implementation you must submit an official request to access the software `dbgen` at their official [website](https://www.tpc.org/tpch/)
