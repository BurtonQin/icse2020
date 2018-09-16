#/bin/bash

source ../exports.sh

## Collect information for each crate 

export RUST_LOG=error


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

mkdir -p $CRATES_DIR

for crate in $(find $CRATES_IO_INDEX_HOME -type d -printf '%f\n')
do
	echo "cloning crate $crate"
	cargo clone $crate --prefix $OUTPUT_DIR
done 

cd $CRT_DIR


