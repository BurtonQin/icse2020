source ../../exports.sh

cd ../../unsafe-analysis
./compile.sh build

cd ../examples/tests/

source ../../rust_flags.sh

UNSAFE_ANALYSIS_DIR=/tmp/unsafe-analysis

mkdir -p $UNSAFE_ANALYSIS_DIR

export RUST_LOG=unsafe_analysis=debug,rustc=error

cargo +$NIGHTLY clean; cargo +$NIGHTLY build
