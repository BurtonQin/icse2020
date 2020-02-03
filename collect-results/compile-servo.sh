
source ../exports.sh 

export RUST_BACKTRACE=1
export RUST_LOG=error

export NIGHTLY=nightly-2019-07-01

export FULL_ANALYSIS_DIR=${UNSAFE_ANALYSIS_DIR}/servo-full-analysis/
export RQ_DIR=RQ_DIR=${UNSAFE_ANALYSIS_DIR}/research-questions-servo

# process all
cargo +$NIGHTLY $1

#sed "s/'/\`/g" $RQ_DIR/rq03-impls-names > out
#mv out $RQ_DIR/rq03-impls-names

#sed "s/'/\`/g" $RQ_DIR/rq06 > out
#mv out $RQ_DIR/rq06


