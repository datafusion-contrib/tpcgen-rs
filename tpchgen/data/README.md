# TPCH Prepared Test Files

This folder stores test files used to ensure consistency with Apache Trino's and
OLTPBenchmarks outputs.

The files are stored as gzipped tbl files and we plan to potentially add support
for parquet in the futures.

## tbl Test Files

The folders follow the `sf-{scale-factor}` pattern.

| Folder   | Description                             |
| -------- | --------------------------------------- |
| sf-0.01  | TPCH dataset of a scale factor of 0.01  |
| sf-0.001 | TPCH dataset of a scale factor of 0.001 |

The tbl files are all named after the tables they represent.

| File         | Description         |
| ------------ | ------------------- |
| parts.tbl    | TPCH parts table    |
| customer.tbl | TPCH customer table |
| lineitem.tbl | TPCH linetime table |
| nation.tbl   | TPCH nation table   |
| orders.tbl   | TPCH order table    |
| partsupp.tbl | TPCH partsupp table |
| region.tbl   | TPCH region table   |
| supplier.tbl | TPCH supplier table |

## The TPCH schema

```
+-----------------+        +-------------------+       +--------------------+       +-------------------+
| PART (P_)       |        | PARTSUPP (PS_)    |       | LINEITEM (L_)      |       | ORDERS (O_)       |
| SF*200,000      |        | SF*800,000        |       | SF*6,000,000       |       | SF*1,500,000      |
+-----------------+        +-------------------+       +--------------------+       +-------------------+
| PARTKEY    PK   |------->| PARTKEY      FK   |----+  | ORDERKEY      FK   |<------| ORDERKEY      PK  |
| NAME            |   +--->| SUPPKEY      FK   |--+ +->| PARTKEY       FK   |   +-->| CUSTKEY       FK  |
| MFGR            |   |    | AVAILQTY          |  +--->| SUPPKEY       FK   |   |   | ORDERSTATUS       |
| BRAND           |   |    | SUPPLYCOST        |       | LINENUMBER         |   |   | TOTALPRICE        |
| TYPE            |   |    | COMMENT           |       | QUANTITY           |   |   | ORDERDATE         |
| SIZE            |   |    +-------------------+       | EXTENDEDPRICE      |   |   | ORDERPRIORITY     |
| CONTAINER       |   |                                | DISCOUNT           |   |   | CLERK             |
| RETAILPRICE     |   |                                | TAX                |   |   | SHIPPRIORITY      |
| COMMENT         |   |                                | RETURNFLAG         |   |   | COMMENT           |
+-----------------+   |                                | LINESTATUS         |   |   +-------------------+
                      |                                | SHIPDATE           |   |           ^
+-----------------+   |    +-------------------+       | COMMITDATE         |   |           |
| SUPPLIER (S_)   |   |    | CUSTOMER (C_)     |       | RECEIPTDATE        |   |           |
| SF*10,000       |   |    | SF*150,000        |       | SHIPINSTRUCT       |   |           |
+-----------------+   |    +-------------------+       | SHIPMODE           |   |           |
| SUPPKEY    PK   |---.    | CUSTKEY     PK    |---+-->| COMMENT            |   |           |
| NAME            |   |    | NAME              |   |   +--------------------+   |           |
| ADDRESS         |   |    | ADDRESS           |   +----------------------------+           |
| NATIONKEY  FK   |---+--->| NATIONKEY    FK   |--------------------------------------------+
| PHONE           |        | PHONE             |
| ACCTBAL         |        | ACCTBAL           |
| COMMENT         |        | MKTSEGMENT        |
+-----------------+        | COMMENT           |
         ^                 +-------------------+
         |                         |
         |                         v
+-----------------+       +-------------------+
| NATION (N_)     |       | REGION (R_)       |
| 25              |       | 5                 |
+-----------------+       +-------------------+
| NATIONKEY  PK   |       | REGIONKEY    PK   |
| NAME            |       | NAME              |
| REGIONKEY  FK   |------>| COMMENT           |
| COMMENT         |       +-------------------+
+-----------------+
```

# Comparing with other TPCH dbgen programs

The classic TPC-H data generator is written in a older dialect of C. However
it is important that this data generator produces the same output.

We can compare the results in these directories with the results produced by the
C data generator to verify they are the same. To do so:

Step 1: create `tbl` files.

One way to do this is using a docker container that has the classic
data generator prebuilt, though you could also build it from
[source](https://github.com/electrum/tpch-dbgen):

```shell
docker run -v `pwd`:/data -it  ghcr.io/scalytics/tpch-docker:main -vf -s 0.001
```

This produces data that matches what is currently checked in here.

Here is an example from `customers.tbl`:

```text
1|Customer#000000001|IVhzIApeRb ot,c,E|15|25-989-741-2988|711.56|BUILDING|to the even, regular platelets. regular, ironic epitaphs nag e|
2|Customer#000000002|XSTf4,NCwDVaWNe6tEgvwfmRchLXak|13|23-768-687-3665|121.65|AUTOMOBILE|l accounts. blithely ironic theodolites integrate boldly: caref|
3|Customer#000000003|MG9kdTD2WBHm|1|11-719-748-3364|7498.12|AUTOMOBILE| deposits eat slyly ironic, even instructions. express foxes detect slyly. blithely even accounts abov|
...
```

Thus the data must be normalized to compare with what is checked in
here. For example, one way to do so is

```shell
# unzip and write the files to a temporary directory.
cat sf-0.001/customer.tbl.gz | gunzip > /tmp/customer.java.tbl
```

And then compare with `diff`

```shell
diff -du /tmp/customer.c.tbl /tmp/customer.java.tbl
```