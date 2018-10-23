#/bin/bash

source ../exports.sh

CRT_DIR=`pwd`

# compile plugin
unset RUSTFLAGS
cd ../unsafe-analysis
cargo +${NIGHTLY} build

source ../rust_flags.sh

#rm $PROJECT_OUT/analysis_pass.txt
#rm $PROJECT_OUT/analysis_fails.txt

cd $CRATES_DIR

for x in {a..m}
do
	for d in $(ls -d $CRATES_DIR/$x*)
	do
		echo "Compiling $d"
		cd $d
		cargo +$NIGHTLY clean
		RUST_BACKTRACE=1 cargo +$NIGHTLY build
		RESULT=$?
	        if [ $RESULT -eq 0 ]; then
        	        echo "$d: Passed">>$PROJECT_OUT/analysis_pass.txt
	        else
		#	export DO_NOT_USE_INSTANCE=true
		#	RUST_BACKTRACE=1 cargo +$NIGHTLY build
	        #        RESULT=$?
        	#        if [ $RESULT -eq 0 ]; then
                #	        echo "$d: Passed">>$PROJECT_OUT/analysis_pass.txt
                #	else
        	        	echo "$d">>$PROJECT_OUT/analysis_fails.txt
                #		echo "$d: Failed"
		#	fi
		#	unset DO_NOT_USE_INSTANCE
        	fi
        	cargo +$NIGHTLY clean
		cd ..
	done
done


cd $CRT_DIR

