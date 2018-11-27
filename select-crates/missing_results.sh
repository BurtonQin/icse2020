#/bin/bash

source ../exports.sh

CRT_DIR=`pwd`
cd $CRATES_DIR
for d in $(ls -d $CRATES_DIR/*)
do
	DIR=`basename $d`
	if [ -d $FULL_ANALYSIS_DIR/$DIR ] 
	then 
		echo -n ""		
	else
		echo "$DIR"
	fi
done

cd $CRT_DIR