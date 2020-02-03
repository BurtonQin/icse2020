
source ../exports.sh 

export RUST_BACKTRACE=1
export RUST_LOG=error

export NIGHTLY=nightly-2019-07-01

export CRATES_FILE=${PROJECT_HOME}/select-crates/crates.io-90-percent
export RQ_DIR=${UNSAFE_ANALYSIS_DIR}/research-questions-90-percent
### process top crates
cargo +$NIGHTLY $1  

#sed "s/'/\`/g" $RQ_DIR/rq03-impls-names > out
#mv out $RQ_DIR/rq03-impls-names

#sed "s/'/\`/g" $RQ_DIR/rq06 > out
#mv out $RQ_DIR/rq06


