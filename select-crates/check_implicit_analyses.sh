#!/bin/bash

source ../exports.sh

LST=$(ls -d $FULL_ANALYSIS_DIR/*)

for d in $LST
do
    	CRATE=`basename $d`
    	CO=`grep NormalNotSafe $d/$CRATE/*/11_coarse_opt_unsafe_in_call_tree_* | wc -l`
    	CP=`grep NormalNotSafe $d/$CRATE/*/11_coarse_pes_unsafe_in_call_tree_* | wc -l`
    	PO=`grep NormalNotSafe $d/$CRATE/*/11_precise_opt_unsafe_in_call_tree_* | wc -l`
	PP=`grep NormalNotSafe $d/$CRATE/*/11_precise_pes_unsafe_in_call_tree_* | wc -l`

	echo "$CRATE: $CO $PO $PP $CP"
done


