
source ../exports.sh 

export RUST_BACKTRACE=1
export RUST_LOG=error

export RQ_DIR=${UNSAFE_ANALYSIS_DIR}/research-questions-servo-all
export ANALYSIS_RESULTS_DIR=~/unsafe_analysis/analysis-data/applications/servo

# process all
cargo +$NIGHTLY $1

sed "s/'/\`/g" $RQ_DIR/rq03-impls-names > out
mv out $RQ_DIR/rq03-impls-names

sed "s/'/\`/g" $RQ_DIR/rq06 > out
mv out $RQ_DIR/rq06

export CRATES_FILE=${PROJECT_HOME}/select-crates/servo-crates
export RQ_DIR=${UNSAFE_ANALYSIS_DIR}/research-questions-servo-only
## process top crates
cargo +$NIGHTLY $1  

sed "s/'/\`/g" $RQ_DIR/rq03-impls-names > out
mv out $RQ_DIR/rq03-impls-names

sed "s/'/\`/g" $RQ_DIR/rq06 > out
mv out $RQ_DIR/rq06


