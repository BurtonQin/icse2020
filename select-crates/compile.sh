#/bin/bash

source ../exports.sh

CRT_DIR=`pwd`

# compile plugin
unset RUSTFLAGS
cd ../unsafe-analysis
cargo +${NIGHTLY} build

source ../rust_flags.sh

cd $CRATES_DIR

for x in {a..z}
do
	for d in $(ls -d $CRATES_DIR/$x*)
	do
		echo "Compiling $d"
		cd $d
		cargo +$NIGHTLY clean
		cargo +$NIGHTLY build
		RESULT=$?
	        if [ $RESULT -eq 0 ]; then
        	        echo "$d: Passed">>$PROJECT_OUT/alaysis_pass.txt
	        else
        	        echo "$d">>$PROJECT_OUT/analysis_fails.txt
                	echo "$d: Failed"
        	fi
        	cargo +$NIGHTLY clean
		cd ..
	done
done


cd $CRT_DIR

