#!/bin/bash 

source ../exports.sh

for f in $(find $ANALYSIS_RESULTS_DIR -name 41_blocks_summary)
do
	sed -n '1,1p' $f > $ANALYSIS_RESULTS_DIR/41_blocks_summary_tmp
	mv $ANALYSIS_RESULTS_DIR/41_blocks_summary_tmp $f
done

for f in $(find $ANALYSIS_RESULTS_DIR -name 02_summary_functions)
do
        sed -n '1,1p' $f > $ANALYSIS_RESULTS_DIR/02_summary_functions_tmp
        mv $ANALYSIS_RESULTS_DIR/02_summary_functions_tmp $f
done

