#/bin/bash

source ../exports.sh

CRT_DIR=`pwd`

# compile plugin
unset RUSTFLAGS
cd ../unsafe-analysis
cargo +${NIGHTLY} build

source ../rust_flags.sh

export RUST_LOG=error

#export  OPENSSL_DIR=/usr/local/openssl-1.1.1/
#export  OPENSSL_DIR=/usr/local/openssl-0.9.8
#export  OPENSSL_DIR=/usr/local/openssl-1.0.0
export  OPENSSL_DIR=/usr/local/openssl-1.0.1
export LLVM_CONFIG_PATH=/usr/local/llvm-5-c3/bin/llvm-config
export LIBCLANG_INCLUDE_PATH=/usr/local/llvm-5-c3/include/
export LIBCLANG_STATIC_PATH=/usr/local/llvm-5-c3/lib/


rm $PROJECT_OUT/recheck_pass.txt
rm $PROJECT_OUT/recheck_fails.txt

cd $CRATES_DIR
while read p; do
	p=${CRATES_DIR}/$p
	if [ -d "$p" ] 
	then 
		cd "$p"

		d=`basename $p`
		export FULL_ANALYSIS_DIR=${UNSAFE_ANALYSIS_DIR}/full-analysis/$d
		echo "FULL_ANALYSIS_DIR=$FULL_ANALYSIS_DIR"
		rm -rf $FULL_ANALYSIS_DIR
	    	mkdir -p $FULL_ANALYSIS_DIR

		echo "$p : compiling"
		cargo +$NIGHTLY clean
		RUST_BACKTRACE=1 cargo +$NIGHTLY build
		RESULT=$?
	        if [ $RESULT -eq 0 ]; then
        	        echo "$d">>$PROJECT_OUT/recheck_pass.txt
	        else
			rm -rf $FULL_ANALYSIS_DIR
			echo "$d">>$PROJECT_OUT/recheck_fails.txt
		fi

	else 
		echo "$p : removed"
	fi
done < $1


