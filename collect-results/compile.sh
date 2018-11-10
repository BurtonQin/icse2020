
source ../exports.sh 

export RUST_BACKTRACE=1
export RUST_LOG=collect_results=DEBUG

# use data from github
#export FULL_ANALYSIS_DIR=/home/nora/work/unsafe_study/data/raw

cargo +$NIGHTLY $1 

