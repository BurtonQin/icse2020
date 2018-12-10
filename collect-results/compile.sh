
source ../exports.sh 

export RUST_BACKTRACE=1
export RUST_LOG=error

# process all
cargo +$NIGHTLY $1

export CRATES_FILE=${PROJECT_HOME}/select-crates/crates.io-90-percent
export RQ_DIR=${UNSAFE_ANALYSIS_DIR}/research-questions-90-percent
## process top crates
cargo +$NIGHTLY $1  

