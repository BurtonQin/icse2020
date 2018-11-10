#/bin/bash

source ../exports.sh

CRT_DIR=`pwd`

# compile plugin
unset RUSTFLAGS
cd ../unsafe-analysis
cargo +${NIGHTLY} build

source ../rust_flags.sh

export RUST_LOG=error

#rm $PROJECT_OUT/analysis_pass.txt
#rm $PROJECT_OUT/analysis_fails.txt

cd $CRATES_DIR

while read p; do
	cd "$p"
	if [ -d "$p" ] 
	then 
		echo "$p : compiling"
		cargo +$NIGHTLY clean
		RUST_BACKTRACE=1 cargo +$NIGHTLY build
		RESULT=$?
	        if [ $RESULT -eq 0 ]; then
        	        echo "$p">>$PROJECT_OUT/recheck_pass.txt
	        else
			echo "$p">>$PROJECT_OUT/recheck_fails.txt
		fi

	else 
		echo "$p : removed"
	fi
done <$PROJECT_OUT/analysis_fails.txt


