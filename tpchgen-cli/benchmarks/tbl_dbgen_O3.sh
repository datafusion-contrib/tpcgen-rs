#!/bin/bash
#
# Runs the classic dbgen c program, patched to compile with -O 3 to generate tbl data at various scales

set -x
set -e

LOGFILE=tbl_dbgen_O3.txt

# checkout and build classic dbgen
rm -rf tpch-dbgen_O3
git clone https://github.com/electrum/tpch-dbgen.git tpch-dbgen_O3
pushd tpch-dbgen_O3
# apply patch for better optimization
patch -p0 < ../tpch-dbgen_O3.patch
make
popd


echo "***********Timings**********" >> $LOGFILE
date >> $LOGFILE
uname -a >> $LOGFILE

SCALE_FACTORS="1 10 100 1000"
#SCALE_FACTORS="1"
for sf in $SCALE_FACTORS ; do
    echo "SF=$sf" >> $LOGFILE
    # dbgen needs to run in the same directory as the .dss file
    pushd tpch-dbgen_O3
    rm -f *.tbl
    /usr/bin/time -a -o ../$LOGFILE ./dbgen -s $sf -f
    popd
done
