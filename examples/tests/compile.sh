PROJECT_HOME="$HOME/work/unsafe_study"

export RUSTFLAGS="--extern unsafe_analysis=$PROJECT_HOME/unsafe-analysis/target/debug/libunsafe_analysis.so -Z extra-plugins=unsafe_analysis --emit mir"
export RUST_BACKTRACE=1
export RUST_LOG="unsafe_analysis="

cargo +nightly-2018-09-08  $1

