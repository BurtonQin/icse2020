source ../../exports.sh
source ../../rust_flags.sh

#echo $RUSTFLAGS

mkdir -p $UNSAFE_ANALYSIS_DIR

RUST_LOG=info

cargo +$NIGHTLY  $1

