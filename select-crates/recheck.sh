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

while read p; do
	cd "$p"
	if [ -d "$p" ] 
	then 
		echo "$p : compiling"
		cargo +$NIGHTLY clean
		RUST_BACKTRACE=1 cargo +$NIGHTLY build
	else 
		echo "$p : removed"
	fi
done <$PROJECT_OUT/analysis_fails.txt


