#!/bin/bash

source ../exports.sh

mkdir -p $ANALYSIS_RESULTS_DIR

for d in $(ls -d $FULL_ANALYSIS_DIR/*)
do
    cd $d
    cp -r * $ANALYSIS_RESULTS_DIR
done
