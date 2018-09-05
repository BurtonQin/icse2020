CRT_DIR=`pwd`

HOME_DIR=/tmp/unsafe_analysis/github-downloads
PLUGIN_HOME=$HOME/work/external_calls

cd $HOME_DIR

function compile_1 {
	echo "Processing $1"
	cd $1
	cargo +nightly build
	cd ../
}

function compile_2 {
	echo "Processing $1/$2"
        cd $1/$2
        cargo +nightly build
        cd ../../
}


export RUSTFLAGS="--extern hidden_unsafe=$PLUGIN_HOME/hidden_unsafe/target/debug/libhidden_unsafe.so -Z extra-plugins=hidden_unsafe --emit mir"

compile_2 xi-editor rust

#TODO add servo when the nighlty version is 2018-08-29

compile_1 alacritty
compile_1 ripgrep
compile_1 xray
compile_1 fd
compile_1 leaf
compile_1 Rocket

cd $CRT_DIR
