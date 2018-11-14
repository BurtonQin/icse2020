
source ../exports.sh 

export RUST_BACKTRACE=1
export RUST_LOG=collect_results=DEBUG

export CRATES_FILE=~/work/unsafe_study/select-crates/crates.io-90-percent
export RQ_DIR=${UNSAFE_ANALYSIS_DIR}/research-questions-90-percent

# use data from github
#export FULL_ANALYSIS_DIR=/home/nora/work/unsafe_study/data/raw

cargo +$NIGHTLY $1 

