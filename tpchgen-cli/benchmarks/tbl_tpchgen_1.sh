#!/bin/bash
#
# Runs the tpchgen-cli, restricted to a single core
# to generate tbl data at various scales

set -x
set -e

LOGFILE=tbl_tpchgen_1.txt
echo "***********Timings**********" >> $LOGFILE
date >> $LOGFILE
uname -a >> $LOGFILE

#SCALE_FACTORS="1 10 100 1000"
SCALE_FACTORS="1000"
for sf in $SCALE_FACTORS ; do
    echo "SF=$sf" >> $LOGFILE
    /usr/bin/time -a -o $LOGFILE tpchgen-cli --num-threads=1 -s $sf --output-dir=out_tpchgen
done
