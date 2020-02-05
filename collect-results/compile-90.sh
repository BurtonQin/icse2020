
source ../exports.sh 

export RUST_BACKTRACE=1
export RUST_LOG=info

export NIGHTLY=nightly-2019-07-01

export CRATES_FILE=${PROJECT_HOME}/select-crates/crates.io-90-percent-2018
export RQ_DIR=${UNSAFE_ANALYSIS_DIR}/research-questions-90-percent
### process top crates
cargo +$NIGHTLY $1  

sed "s/'/\`/g" $RQ_DIR/rq01-impls-names > out
mv out $RQ_DIR/rq01-impls-names

sed "s/'/\`/g" $RQ_DIR/rq04 > out
mv out $RQ_DIR/rq04



