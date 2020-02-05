
source ../exports-2019.sh 

export RUST_BACKTRACE=full
export RUST_LOG=info

# process all
cargo +$NIGHTLY $1

sed "s/'/\`/g" $RQ_DIR/rq03-impls-names > out
mv out $RQ_DIR/rq03-impls-names

sed "s/'/\`/g" $RQ_DIR/rq06 > out
mv out $RQ_DIR/rq06

export UNSAFE_ANALYSIS_DIR=${PROJECT_OUT}/analysis-data
export FULL_ANALYSIS_DIR=${UNSAFE_ANALYSIS_DIR}/full-analysis
#export CRATES_FILE=${PROJECT_HOME}/select-crates/crates.io-90-percent-2018
export RQ_DIR=${UNSAFE_ANALYSIS_DIR}/research-questions-90-percent
## process top crates
cargo +$NIGHTLY $1  

sed "s/'/\`/g" $RQ_DIR/rq03-impls-names > out
mv out $RQ_DIR/rq03-impls-names

sed "s/'/\`/g" $RQ_DIR/rq06 > out
mv out $RQ_DIR/rq06


