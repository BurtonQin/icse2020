#/bin/bash

source ../exports.sh

EMPTY_CRATES_DIR=$PROJECT_OUT/empty-crates/
mkdir -p $EMPTY_CRATES_DIR

while read d; do

	echo $d
	echo "mv $CRATES_DIR/$d  $EMPTY_CRATES_DIR"
	mv $CRATES_DIR/$d  $EMPTY_CRATES_DIR
	#echo "git rm -r $PROJECT_HOME/data/raw/$d"
	#git rm -r $PROJECT_HOME/data/raw/$d
done <./empty_crates.txt
