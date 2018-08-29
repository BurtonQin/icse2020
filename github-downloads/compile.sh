CRT_DIR=`pwd`

HOME_DIR=/tmp/unsafe_analysis/github-downloads
PLUGIN_HOME=/home/ans5k/work/external_calls

cd $HOME_DIR


export RUSTFLAGS="--extern hidden_unsafe=$PLUGIN_HOME/hidden_unsafe/target/debug/libhidden_unsafe.so -Z extra-plugins=hidden_unsafe --emit mir"

cd xi-editor/rust
cargo +nightly build

cd $CRT_DIR

