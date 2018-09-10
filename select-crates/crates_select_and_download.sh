#/bin/bash

## Collect information for each crate 

export RUST_LOG=error

if [ -z "$1" ]
then 
	TOP=500
else 
	TOP=$1
fi

OUTPUT_DIR=/tmp/unsafe_analysis/crates.io-downloads

CRT_DIR=`pwd`

if [ ! -f crates.io-fixed ] 
then 
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

	rm -f crates.io

	for crate in $(find $CRATES_IO_INDEX_HOME -type f -printf '%f\n')
	do
		echo "Processing crate $crate"
		curl https://crates.io/api/v1/crates/$crate >> crates.io
		res=$?
		if test "$res" != "0"
		then
	 		echo "Query failed for crate: $crate"
		fi
		echo "" >> crates.io
	done
	grep -v "\"detail\":\"Not Found\"" crates.io >> crates.io-fixed
fi

FILE=`pwd`/crates.io-fixed
cargo run -- "$TOP" $FILE | sed 's/\"//g' > top-N-crates.io

rm -rf $OUTPUT_DIR
mkdir -p $OUTPUT_DIR

while read line
do
	crate=$(echo $line | cut -d' ' -f 1)
	echo "cloning crate $crate"
	cargo clone $crate --prefix $OUTPUT_DIR
done <top-N-crates.io

cd $CRT_DIR


