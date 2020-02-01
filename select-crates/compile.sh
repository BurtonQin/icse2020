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

for x in {b..z}
do
	for d in $(ls -d $CRATES_DIR/$x*)
	do
	    echo "Compiling $d"

	    #delete old files
	    crate=`basename $d`
	    export FULL_ANALYSIS_DIR=${UNSAFE_ANALYSIS_DIR}/full-analysis/$crate
	    rm -rf $FULL_ANALYSIS_DIR
	    mkdir -p $FULL_ANALYSIS_DIR

	    echo "FULL_ANALYSIS_DIR=$FULL_ANALYSIS_DIR"
	    
	    cd $d
	    cargo +$NIGHTLY clean
	    RUST_BACKTRACE=1 cargo +$NIGHTLY build
	    RESULT=$?
	    if [ $RESULT -eq 0 ]; then
        	echo "$d">>$PROJECT_OUT/analysis_pass.txt
	    else
		echo "Compilation FAILED ... removing $FULL_ANALYSIS_DIR"
		rm -rf $FULL_ANALYSIS_DIR
		echo "$d">>$PROJECT_OUT/analysis_fails.txt
        	
	    fi
	    cargo +$NIGHTLY clean
	done
	#pushd ${UNSAFE_ANALYSIS_DIR}/full-analysis/
	#tar czf ${x}.tgz ${x}*
	#mv ${x}.tgz ${UNSAFE_ANALYSIS_DIR}
	#rm -rf ${x}*
done

cd $CRT_DIR

