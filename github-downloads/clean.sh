source ../exports.sh

pushd $GITHUB_APPS

function compile_1 {
	echo "Processing $1"
	cd $1
	cargo +$NIGHTLY clean
	cd $GITHUB_APPS
}

function compile_2 {
	echo "Processing $1/$2"
        cd $1/$2
	cargo +$NIGHTLY clean
        cd $GITHUB_APPS
}


source ../rust_flags.sh

compile_1 servo

# TODO
cd redox
make -i clean
cd..

cd tock/boards/hail
make clean 
cd ../ek-tm4c1294xl
make clean
cd ../imix
make clean
cd ../launchxl
make clean
cd ../nordic
make clean

cd $GITHUB_APPS
compile_1 mdbook
compile_1 trust-dns
cd linkerd2-proxy
rm -rf target
compile_1 rsign
compile_1 flowgger
compile_1 alacritty
compile_1 collections-app
compile_1 polkadot
compile_1 mooneye-gb

# industrial automation
compile_1 tokio-modbus
compile_1 modbus-iiot-rust

# 
compile_1 parity-ethereum

# games
compile_1 rboy
compile_2 pinky pinky-libretro
compile_1 zemeroth

#graphics
compile_1 svgcleaner
compile_1 Image-Processing-CLI-in-Rust

#security tools
compile_1 rshijack
compile_1 badtouch
compile_1 sniffglue

#system tools
compile_1 tokei
compile_1 funzzy
compile_1 fblog
compile_1 fselect
compile_1 rrun
compile_1 zou
compile_1 concurr
compile_1 fontfinder
compile_1 parallel
compile_1 systemd-manager
compile_1 exa
compile_1 logram
compile_1 ion
compile_1 bat
compile_1 fd
compile_1 hex
compile_1 bingrep
compile_1 aliases

#text editors
compile_2 xi-editor rust
compile_1 xray
cd remacs
make clean
cd ../

#text processing
compile_1 ripgrep
compile_1 LanguageClient-neovim

compile_1 xsv

#video
compile_1 slingr
compile_1 learn-opengl-rs

#web server
compile_1 http
compile_1 miniserve
compile_1 simple-http-server
compile_1 naglfar

#web
compile_1 ruster
compile_1 muro
compile_1 webrender
compile_1 whitebox-tools
#SubstratumNode
compile_2 SubstratumNode dns_utility
compile_2 SubstratumNode entry_dns_lib
compile_2 SubstratumNode hopper_lib
compile_2 SubstratumNode neighborhood_lib
compile_2 SubstratumNode node
compile_2 SubstratumNode proxy_client_lib
compile_2 SubstratumNode proxy_server_lib
compile_2 SubstratumNode sub_lib
#
compile_1 substrate

#development tools
compile_1 clog-cli
compile_1 rusty-tags
compile_1 rustfix
compile_1 just
compile_1 git-journal
compile_1 ptags
compile_1 racer
compile_1 rustfmt
compile_1 rustup.rs
compile_1 fw
compile_1 semantic-rs

#static analysis
compile_1 static-assertions-rs
compile_1 super
compile_1 wasabi
compile_1 polonius

#testing
compile_1 quickcheck
compile_1 mockito
compile_1 speculate.rs
compile_1 afl.rs
compile_1 trust
compile_1 tarpaulin
compile_1 utest

#rustc
cd rust
./x.py clean
cd $GITHUB_APPS

#compile_1 leaf
#compile_1 Rocket

popd

