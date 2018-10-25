source ../../exports.sh
source ../../rust_flags.sh

#echo $RUSTFLAGS

mkdir -p $UNSAFE_ANALYSIS_DIR

cargo +$NIGHTLY  $1

