#/bin/bash

source ../exports.sh

CRT_DIR=`pwd`

# compile plugin
unset RUSTFLAGS

rm -f $PROJECT_OUT/geiger_results.txt

for x in {a..z}
do
	for d in $(ls -d $GEIGER_CRATES/*)
	do
		echo "Checking $d"
		cd $OSHA_HOME
		OUT=`cargo run -- $d/**.rs | tail -n 5 | cut -d' ' -f 3 | sed ':loop;N;s/\n/ /g;t loop'`
		echo "$OUT $d" >>  $PROJECT_OUT/geiger_results.txt
	done
done


cd $CRT_DIR

