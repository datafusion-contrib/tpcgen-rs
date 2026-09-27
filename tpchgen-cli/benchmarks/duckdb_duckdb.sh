#!/bin/bash
#
# Runs the duckdb to generate custom duckdb files at scale factors

set -x
set -e

LOGFILE=duckdb_duckdb.txt
echo "***********Timings**********" >> $LOGFILE
date >> $LOGFILE
uname -a >> $LOGFILE

SCALE_FACTORS="1 10 100"
rm -rf out_duckdb
mkdir out_duckdb
for sf in $SCALE_FACTORS ; do
    echo "SF=$sf" >> $LOGFILE
    /usr/bin/time -a -o $LOGFILE duckdb out_duckdb/$sf.duckdb "\
INSTALL tpch;\
LOAD tpch;\
CALL dbgen(sf = $sf);"

done
