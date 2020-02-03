#/bin/bash

source ../exports.sh

CRT_DIR=`pwd`
cd $CRATES_DIR
for d in $(ls -d $CRATES_DIR/*)
do
	DIR=`basename $d`
	if [ -d $FULL_ANALYSIS_DIR/$DIR ] 
	then
		if [ -z "`find \"$FULL_ANALYSIS_DIR/$DIR\" -mindepth 1 -exec echo notempty \; -quit`" ] 
		then
    			echo "$CRATES_DIR/$DIR"
		else
			echo -n ""		
		fi
	else
		echo "$CRATES_DIR/$DIR"
	fi
done

cd $CRT_DIR
