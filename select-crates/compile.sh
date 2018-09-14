#/bin/bash

CRT_DIR=`pwd`
PROJECT_HOME="$HOME/work/unsafe_study/"

CRATES_DIR=/tmp/unsafe_analysis/crates.io-downloads/
COMPILER_OUTPUT_DIR=/tmp/unsafe_analysis/compiler_output

rm -rf $COMPILER_OUTPUT_DIR
mkdir $COMPILER_OUTPUT_DIR

NIGHTLY=nightly-2018-09-08

#export RUST_BACKTRACE=1

cd ../unsafe-analysis
cargo +$NIGHTLY build

export RUSTFLAGS="--extern unsafe_analysis=$PROJECT_HOME/unsafe-analysis/target/debug/libunsafe_analysis.so -Z extra-plugins=unsafe_analysis --emit mir"

cd $CRATES_DIR
for for x in {a..z}
do
	if [ -f $x.tgz ]
	then
		tar -pzxf $x.tgz
	fi
	for d in $(ls $CRATES_DIR/$x*)
	do
		echo "Compiling $d"
		cd $d
		cargo +$NIGHTLY build
		RESULT=$?
	        if [ $RESULT -eq 0 ]; then
        	        echo "$d: Passed"
	        else
        	        echo "$d">>$CRATES_DIR/analysis_fails.txt
                	echo "$d: Failed"
        	fi
        	cargo +NIGHTLY clean
	cd ..
	tar -pzcvf $x.tgz $CRATES_DIR/$x*
	rm -rf $CRATES_DIR/$x*
done


cd $CRT_DIR

