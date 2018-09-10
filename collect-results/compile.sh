PROJECT_HOME="$HOME/work/unsafe_study"
#rustup override set nightly-2018-08-29

export RUST_BACKTRACE=1
export RUST_LOG=collect_results=DEBUG

cargo +nightly-2018-08-29 $1 

