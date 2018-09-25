source ../../exports.sh
source ../../rust_flags.sh

echo $RUSTFLAGS

mkdir -p $UNSAFE_ANALYSIS_DIR

RUST_LOG=debug

cargo +$NIGHTLY  $1

