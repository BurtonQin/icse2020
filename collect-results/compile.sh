
source ../exports.sh 

export RUST_BACKTRACE=1
export RUST_LOG=collect_results=DEBUG

cargo +$NIGHTLY $1 

