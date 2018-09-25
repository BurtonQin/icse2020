#/bin/bash

source ../exports.sh

CRT_DIR=`pwd`

# compile plugin
unset RUSTFLAGS

rm -f $UNSAFE_ANALYSIS_DIR/grep_results.txt

for d in $(ls -d $EXCLUDED_CRATES/*)
do
	echo "Checking $d"
	cd $d/src
	FUNCTIONS=`grep -r --include "*.rs" -w "\<unsafe[[:space:]]fn\>" . | wc -l`
	TRAITS=`grep -r --include "*.rs" -w "\<unsafe[[:space:]]trait\>" . | wc -l`
	IMPLS=`grep -r --include "*.rs" -w "\<unsafe[[:space:]]impl\>" . | wc -l`
	ALL=`grep -r --include "*.rs" -w "\<unsafe\>" . | wc -l`
	echo "$FUNCTIONS $TRAITS $IMPLS $ALL $crate" >> $UNSAFE_ANALYSIS_DIR/grep_results.txt
done


cd $CRT_DIR

