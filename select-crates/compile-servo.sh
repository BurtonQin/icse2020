#/bin/bash

source ../exports.sh

CRT_DIR=`pwd`

export RUST_LOG=error

# compile plugin
unset RUSTFLAGS
cd ../unsafe-analysis
cargo +${NIGHTLY} build

source ../rust_flags.sh

rm -f $PROJECT_OUT/servo_analysis_pass.txt
rm -f $PROJECT_OUT/servo_analysis_fails.txt

export FULL_ANALYSIS_DIR=${UNSAFE_ANALYSIS_DIR}/servo-full-analysis/
rm -rf $FULL_ANALYSIS_DIR
mkdir -p $FULL_ANALYSIS_DIR

echo "FULL_ANALYSIS_DIR=$FULL_ANALYSIS_DIR"
	    
cd $PROJECT_OUT/applications/servo/
cargo +$NIGHTLY clean
RUST_BACKTRACE=1 cargo +$NIGHTLY build
RESULT=$?
if [ $RESULT -eq 0 ]; then
	echo "$d">>$PROJECT_OUT/servo_analysis_pass.txt
else
	rm -rf $FULL_ANALYSIS_DIR
	echo "$d">>$PROJECT_OUT/servo_analysis_fails.txt
fi
 cargo +$NIGHTLY clean

cd $CRT_DIR

