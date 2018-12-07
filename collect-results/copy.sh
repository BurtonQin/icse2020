#!/bin/bash

source ../exports.sh

mkdir -p $ANALYSIS_RESULTS_DIR

for d in $(ls -d $FULL_ANALYSIS_DIR/*)
do
    cd $d
    echo "cp -r * $ANALYSIS_RESULTS_DIR"
    cp -r * $ANALYSIS_RESULTS_DIR
    if [ $? -eq 0 ] 
    then
	    echo "OK: $d"
    else
	    echo "Copy error: $d"
    fi
done
