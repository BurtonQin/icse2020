#/bin/bash

CRT_DIR=`pwd`
PROJECT_HOME="/home/nora/work/external_calls"

CRATES_DIR=/tmp/unsafe_analysis/crates.io-downloads/

cd $CRATES_DIR
for d in $(ls $CRATES_DIR)
do
	echo "Compiling $d"
	cd $d
	export RUSTFLAGS="--extern hidden_unsafe=$PROJECT_HOME/hidden_unsafe/target/debug/libhidden_unsafe.so -Z extra-plugins=hidden_unsafe --emit mir"
	cargo +nightly build
	cd ..
done

cd $CRT_DIR

