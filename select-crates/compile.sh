#/bin/bash

source ../exports.sh

CRT_DIR=`pwd`

export RUST_LOG=error

# compile plugin
unset RUSTFLAGS
cd ../unsafe-analysis
cargo +${NIGHTLY} build

source ../rust_flags.sh

rm -f $PROJECT_OUT/analysis_pass.txt
rm -f $PROJECT_OUT/analysis_fails.txt

cd $CRATES_DIR

echo $CRATES_DIR

# $1 the file name
function process_file() {
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
        		        echo "$p">>$PROJECT_OUT/analysis_pass.txt
		        else
				rm -rf $FULL_ANALYSIS_DIR
				echo "$p">>$PROJECT_OUT/analysis_fails.txt
			fi
			cargo +$NIGHTLY clean
		else 
			echo "$p : removed"
		fi
	done < $1
}


for x in {a..a}
do
	process_file ${PROJECT_HOME}/select-crates/files/${x}.txt
	export  OPENSSL_DIR=/usr/local/openssl-1.0.1
	export LLVM_CONFIG_PATH=/usr/local/llvm-5-c3/bin/llvm-config
	export LIBCLANG_INCLUDE_PATH=/usr/local/llvm-5-c3/include/
	export LIBCLANG_STATIC_PATH=/usr/local/llvm-5-c3/lib/
	process_file ${PROJECT_HOME}/select-crates/files/${x}-old-ssl.txt
	unset OPENSSL_DIR
	unset LLVM_CONFIG_PATH
	unset LIBCLANG_INCLUDE_PATH
	unset LIBCLANG_STATIC_PATH
	pushd ${UNSAFE_ANALYSIS_DIR}/full-analysis/
	tar czf ${x}.tgz ${x}*
	mv ${x}.tgz ${UNSAFE_ANALYSIS_DIR}
	rm -rf ${x}*
done

cd $CRT_DIR

