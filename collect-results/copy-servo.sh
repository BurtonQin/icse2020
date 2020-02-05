#!/bin/bash

source ../exports.sh

export ANALYSIS_RESULTS_DIR=${PROJECT_OUT}/analysis-data/results-servo

mkdir -p $ANALYSIS_RESULTS_DIR

export FULL_ANALYSIS_DIR=${PROJECT_OUT}/analysis-data/full-analysis-servo

echo $FULL_ANALYSIS_DIR

for d in $(ls -d $FULL_ANALYSIS_DIR/*)
do
    cd $d
    CRATE=`basename $d`
    echo "cp -r $d/$CRATE $ANALYSIS_RESULTS_DIR"
    cp -r $d $ANALYSIS_RESULTS_DIR
    if [ $? -ne 0 ] 
    then 
	echo "Copy error: $d/$CRATE"
    fi
done
