#/bin/bash

source ../exports.sh

CRT_DIR=`pwd`

export RUST_BACKTRACE=1
export RUST_LOG=error

cd $CRATES_DIR
for x in {a..z}
do
	for d in $(ls -d ${x}*)
	do
		echo "Compiling $d"
		cd $d
		cargo +$NIGHTLY build 
		RESULT=$?
		if [ $RESULT -eq 0 ]; then
			echo "$d: Passed"
		else
  			echo "$d">>$PROJECT_OUT/fails.txt
			echo "$d: Failed"
		fi
		cargo clean
		cd $CRATES_DIR
	done
done

cd $CRT_DIR

