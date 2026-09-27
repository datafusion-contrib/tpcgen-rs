#!/bin/bash
#
# Runs the tpchgen-cli to generate parquet data at various scales,

set -x
set -e

LOGFILE=parquet_tpchgen.txt
echo "***********Timings**********" >> $LOGFILE
date >> $LOGFILE
uname -a >> $LOGFILE

SCALE_FACTORS="1 10 100 1000"
for sf in $SCALE_FACTORS ; do
    echo "SF=$sf" >> $LOGFILE
    /usr/bin/time -a -o $LOGFILE tpchgen-cli parquet -s $sf --output-dir=out_tpchgen
done
