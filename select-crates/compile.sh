#/bin/bash

CRT_DIR=`pwd`
PROJECT_HOME="$HOME/work/external_calls"

CRATES_DIR=/tmp/unsafe_analysis/crates.io-downloads/
COMPILER_OUTPUT_DIR=/tmp/unsafe_analysis/compiler_output

rm -rf $COMPILER_OUTPUT_DIR
mkdir $COMPILER_OUTPUT_DIR

rustup override set nightly-2018-08-29

export RUSTFLAGS="--extern hidden_unsafe=$PROJECT_HOME/hidden_unsafe/target/debug/libhidden_unsafe.so -Z extra-plugins=hidden_unsafe --emit mir"
export RUST_BACKTRACE=1

cd $CRATES_DIR
for d in $(ls $CRATES_DIR)
do
	echo "Compiling $d"
	cd $d
	cargo build &> "$COMPILER_OUTPUT_DIR/$crate"
	cd ..
done

cd $CRT_DIR

