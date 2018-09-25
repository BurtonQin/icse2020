#/bin/bash

source ../exports.sh

CRT_DIR=`pwd`

# compile plugin
unset RUSTFLAGS

mkdir -p $UNSAFE_ANALYSIS_DIR

rm -f $UNSAFE_ANALYSIS_DIR/syntactic_only_*

for d in $(ls -d $SYNTACTIC_ONLY_CRATES/*)
do
	echo "Checking $d"
	cd $OSHA_HOME
	OUT=`cargo run -- $(find $d/src -name *.rs)`
	if [ -z "$OUT" ] 
	then
		echo $d >> $UNSAFE_ANALYSIS_DIR/syntactic_only_fails.txt
	else 
		echo "$OUT, $d" >>  $UNSAFE_ANALYSIS_DIR/syntactic_only_results.txt
	fi
done


cd $CRT_DIR

