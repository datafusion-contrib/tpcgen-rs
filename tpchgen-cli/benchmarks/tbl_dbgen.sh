#!/bin/bash
#
# Runs the classic dbgen c program to generate tbl data at various scales

set -x
set -e

LOGFILE=tbl_dbgen.txt

# checkout and build classic dbgen
rm -rf tpch-dbgen
git clone https://github.com/electrum/tpch-dbgen.git
pushd tpch-dbgen
make
popd 


echo "***********Timings**********" >> $LOGFILE
date >> $LOGFILE
uname -a >> $LOGFILE

SCALE_FACTORS="1 10 100 1000"
for sf in $SCALE_FACTORS ; do
    echo "SF=$sf" >> $LOGFILE
    # dbgen needs to run in the same directory as the .dss file
    pushd tpch-dbgen
    rm -f *.tbl
    /usr/bin/time -a -o ../$LOGFILE ./dbgen -s $sf -f
    popd
done
