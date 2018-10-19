source ../../exports.sh
source ../../rust_flags.sh

#echo $RUSTFLAGS

mkdir -p $UNSAFE_ANALYSIS_DIR

export RUST_LOG=unsafe_analysis=debug,rustc=error

cargo +$NIGHTLY  $1

