source ../../exports.sh
source ../../rust_flags.sh

#echo $RUSTFLAGS

mkdir -p $UNSAFE_ANALYSIS_DIR

export FULL_ANALYSIS_DIR=${UNSAFE_ANALYSIS_DIR}/full-analysis/tests

export RUST_LOG=unsafe_analysis=debug,rustc=error

cargo +$NIGHTLY  $1

