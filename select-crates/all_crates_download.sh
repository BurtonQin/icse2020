#/bin/bash

## Collect information for each crate 

export RUST_LOG=error

OUTPUT_DIR=/tmp/unsafe_analysis/crates.io-downloads

CRT_DIR=`pwd`

cd ../../
if [ -d crates.io-index ] 
then
	cd crates.io-index
	git pull
	CRATES_IO_INDEX_HOME=`pwd`
else 
	git clone https://github.com/rust-lang/crates.io-index.git
	CRATES_IO_INDEX_HOME=`pwd`/crates.io-index
fi

echo "crates.io-index updated, home $CRATES_IO_INDEX_HOME"

cd $CRT_DIR

OUTPUT_DIR=/tmp/unsafe_analysis/crates.io-downloads
mkdir -p $OUTPUT_DIR

for crate in $(find $CRATES_IO_INDEX_HOME -type f -printf '%f\n')
do
	echo "cloning crate $crate"
	cargo clone $crate --prefix $OUTPUT_DIR
done 

cd $CRT_DIR


