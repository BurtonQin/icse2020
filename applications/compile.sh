CRT_DIR=`pwd`

source ../exports.sh
source ../rust_flags.sh

export RUST_LOG=error


#servo
cd ~/unsafe_analysis/applications/servo
export FULL_ANALYSIS_DIR=$UNSAFE_ANALYSIS_DIR/applications/servo
rm -rf $FULL_ANALYSIS_DIR
mkdir -p $FULL_ANALYSIS_DIR
./mach build -d



cd $CRT_DIR

