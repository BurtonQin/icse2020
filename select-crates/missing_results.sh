#/bin/bash

source ../exports.sh

while read d; do
	if [ -d $FULL_ANALYSIS_DIR/$d ] 
	then
		if [ -z "`find \"$FULL_ANALYSIS_DIR/$d/$d\" -mindepth 1 -exec echo notempty \; -quit 2>/dev/null`" ] 
		then
    			echo "$d"
		else
			echo -n ""		
		fi
	else
		echo "$d"
	fi
done < "${PROJECT_HOME}/select-crates/crates_list.txt"
