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
tpcgen-cli tpch -s 1 --output-dir /tmp/tpch
tpcgen-cli tpch csv -s 1 --output-dir /tmp/tpch
tpcgen-cli tpch parquet -s 100 --tables lineitem --parts 10 --output-dir /tmp/tpch
tpcgen-cli tpch parquet -s 1 --tables lineitem --column-encoding=l_comment=DELTA_LENGTH_BYTE_ARRAY --output-dir /tmp/tpch
tpcgen-cli tpcds csv -s 1 --output-dir /tmp/tpcds
tpcgen-cli tpcds csv -s 1 --delimiter='\t' --output-dir /tmp/tpcds
```
