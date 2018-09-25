#/bin/bash

source ../exports.sh

CRT_DIR=`pwd`

export RUST_BACKTRACE=1
export RUST_LOG=error

cd $CRATES_DIR

while read -r d
do 
	echo "Compiling $d"
	if [ -d "$d" ]
	then
                cd $d
                cargo +$NIGHTLY build
                RESULT=$?
                if [ $RESULT -eq 0 ]; then
                        echo "$d: Passed"
                else
                        echo "$d">>$PROJECT_OUT/recheck_fails.txt
                        echo "$d: Failed"
                fi
                cargo clean
                cd ..
	else 
		echo "$d deleted"
	fi
done < $1


