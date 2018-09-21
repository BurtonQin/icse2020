CRT_DIR=`pwd`

source ../exports.sh

cd $GITHUB_APPS

function compile_1 {
	echo "Processing $1"
	cd $1
	cargo +NIGHTLY clean
	cargo +NIGHTLY build
	cd ../
}

function compile_2 {
	echo "Processing $1/$2"
        cd $1/$2
	cargo +NIGHTLY clean
        cargo +NIGHTLY build
        cd ../../
}


source ../rust_flags.sh

compile_1 servo

#compile_2 xi-editor rust

#TODO add servo when the nighlty version is 2018-08-29

#compile_1 alacritty
#compile_1 ripgrep
#compile_1 xray
#compile_1 fd
#compile_1 leaf
#compile_1 Rocket

cd $CRT_DIR

