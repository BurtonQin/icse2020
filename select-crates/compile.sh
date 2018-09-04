#/bin/bash

CRT_DIR=`pwd`
PROJECT_HOME="/home/ans5k/work/external_calls"

CRATES_DIR=/tmp/unsafe_analysis/crates.io-downloads/

rustup override set nightly-2018-08-29

export RUSTFLAGS="--extern hidden_unsafe=$PROJECT_HOME/hidden_unsafe/target/debug/libhidden_unsafe.so -Z extra-plugins=hidden_unsafe --emit mir"

cd $CRATES_DIR
for d in $(ls $CRATES_DIR)
do
	echo "Compiling $d"
	cd $d
	cargo build
	cd ..
done

cd $CRT_DIR

