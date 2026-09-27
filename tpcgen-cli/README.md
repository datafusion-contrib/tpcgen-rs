# tpcgen-cli

`tpcgen-cli` provides the command line interface for generating TPC-H
and TPC-DS benchmark data.

## Install

```shell
pip install tpcgen-cli
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
