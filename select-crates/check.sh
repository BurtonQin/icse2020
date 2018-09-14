#/bin/bash

CRT_DIR=`pwd`
PROJECT_HOME="$HOME/work/unsafe_study/"

CRATES_DIR=/tmp/unsafe_analysis/crates.io-downloads/

NIGHTLY=nightly-2018-09-08

export RUST_BACKTRACE=1
export RUST_LOG=error

cd $CRATES_DIR
for d in $(ls $CRATES_DIR)
do
	echo "Compiling $d"
	cd $d
	cargo +$NIGHTLY build 
	RESULT=$?
	if [ $RESULT -eq 0 ]; then
		echo "$d: Passed"
	else
  		echo "$d">>$CRATES_DIR/fails.txt
		echo "$d: Failed"
	fi
	cargo clean
	cd ..
done

cd $CRT_DIR

