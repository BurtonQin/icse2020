PROJECT_HOME="$HOME/work/unsafe_study"
rustup override set nightly-2018-09-11

export RUST_BACKTRACE=1
export RUST_LOG="unsafe_analysis=debug"

cargo $1 

