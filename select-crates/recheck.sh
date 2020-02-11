#/bin/bash

source ../exports.sh

CRT_DIR=`pwd`

# compile plugin
unset RUSTFLAGS
cd ../unsafe-analysis
cargo +${NIGHTLY} build

source ../rust_flags.sh

export RUST_LOG=error

while read p; do
	echo "$p" 
	if [ -d "$p" ] 
	then 
		cd "$p"
		p=`basename $p`
		export FULL_ANALYSIS_DIR=${UNSAFE_ANALYSIS_DIR}/full-analysis/$p
		echo "FULL_ANALYSIS_DIR=$FULL_ANALYSIS_DIR"
		rm -rf $FULL_ANALYSIS_DIR
	    	mkdir -p $FULL_ANALYSIS_DIR

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


