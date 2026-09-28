#!/bin/bash
#
# Runs the duckdb to generate parquet at various scale factors

set -x
set -e

LOGFILE=parquet_duckdb.txt
echo "***********Timings**********" >> $LOGFILE
uname -a >> $LOGFILE

SCALE_FACTORS="1 10 100"
rm -rf out_duckdb
mkdir out_duckdb
for sf in $SCALE_FACTORS ; do
    echo "SF=$sf" >> $LOGFILE
    /usr/bin/time -a -o $LOGFILE duckdb out_duckdb/$sf.duckdb "\
INSTALL tpch;\
LOAD tpch;\
CALL dbgen(sf = $sf);\
EXPORT DATABASE 'out_duckdb' (FORMAT parquet);"

done
