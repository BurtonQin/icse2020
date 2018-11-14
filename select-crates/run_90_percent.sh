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
	if [ -d "$CRATES_DIR/$p" ] 
	then 
		cd "$CRATES_DIR/$p"
		echo "$p : compiling"
		cargo +$NIGHTLY clean
		RUST_BACKTRACE=1 cargo +$NIGHTLY build
		RESULT=$?
	        if [ $RESULT -eq 0 ]; then
        	        echo "$p">>$PROJECT_OUT/90p_pass.txt
	        else
			echo "$p">>$PROJECT_OUT/90p_fails.txt
		fi

	else 
		echo "$p : removed"
	fi
done <$PROJECT_HOME/select-crates/crates.io-90-percent

