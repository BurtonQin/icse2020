#!/bin/bash

source ../exports.sh

mkdir -p $ANALYSIS_RESULTS_DIR

for d in $(ls -d $FULL_ANALYSIS_DIR/*)
do
    cd $d
    CRATE=`basename $d`
    echo "cp -r $d/$CRATE $ANALYSIS_RESULTS_DIR"
    cp -r $d/$CRATE $ANALYSIS_RESULTS_DIR
    if [ $? -ne 0 ] 
    then 
	echo "Copy error: $d/$CRATE"
    fi
done
