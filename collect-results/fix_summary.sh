#!/bin/bash 

source ../exports.sh

for crate in $(ls -d $ANALYSIS_RESULTS_DIR/*) 
do 
	for ver in $(ls -d $crate/*)
	do
		if [ -f $ver/41_blocks_summary ]
		then
			echo "Ok: $ver/41_blocks_summary"
			sed -n '1,1p' $ver/41_blocks_summary > $ANALYSIS_RESULTS_DIR/41_blocks_summary_tmp
			mv $ANALYSIS_RESULTS_DIR/41_blocks_summary_tmp $ver/41_blocks_summary
			sed -n '1,1p' $ver/02_summary_functions > $ANALYSIS_RESULTS_DIR/02_summary_functions_tmp
			mv $ANALYSIS_RESULTS_DIR/02_summary_functions_tmp $ver/02_summary_functions
		else 
			echo "Missing: $ver/41_blocks_summary"
			rm -rf $ver
		fi
	done
done

#for f in $(find $ANALYSIS_RESULTS_DIR -name 41_blocks_summary)
#do
#	sed -n '1,1p' $f > $ANALYSIS_RESULTS_DIR/41_blocks_summary_tmp
#	mv $ANALYSIS_RESULTS_DIR/41_blocks_summary_tmp $f
#done

#for f in $(find $ANALYSIS_RESULTS_DIR -name 02_summary_functions)
#do
#        sed -n '1,1p' $f > $ANALYSIS_RESULTS_DIR/02_summary_functions_tmp
#        mv $ANALYSIS_RESULTS_DIR/02_summary_functions_tmp $f
#done

