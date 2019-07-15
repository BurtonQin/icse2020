#/bin/bash

source ../exports-2019.sh

CRT_DIR=`pwd`

export RUST_LOG=error

export OUT_DIR=$PROJECT_OUT/test-coverage/
rm -rf $OUT_DIR
mkdir $OUT_DIR


cd $CRATES_DIR

for x in {a..z}
do
	for d in $(ls -d $CRATES_DIR/$x*)
	do
	    echo "Compiling $d"
	    crate=`basename $d`
	    cd $d
	    rustup override set $NIGHTLY
	    cargo clean
	    RUST_BACKTRACE=1 cargo build > $OUT_DIR/${crate}-compile.log
	    RESULT=$?
	    if [ $RESULT -eq 0 ]; then
        	echo "$d">>$PROJECT_OUT/compile_pass.txt
	    else
		echo "$d">>$PROJECT_OUT/compile_fails.txt
        	
	    fi
	    cargo tarpaulin -v > $OUT_DIR/${crate}-coverage.txt
	done
done

cd $CRT_DIR

