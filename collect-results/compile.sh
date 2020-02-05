
source ../exports.sh 

export RUST_BACKTRACE=1
export RUST_LOG=info

export NIGHTLY=nightly-2019-07-01

echo $ANALYSIS_RESULTS_DIR

# process all
cargo +$NIGHTLY $1

sed "s/'/\`/g" $RQ_DIR/rq01-impls-names > out
mv out $RQ_DIR/rq01-impls-names

sed "s/'/\`/g" $RQ_DIR/rq04 > out
mv out $RQ_DIR/rq04


