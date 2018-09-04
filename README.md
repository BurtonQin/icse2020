# external_calls

<h1>Download top N crates from crates.io</h1>

Execute: cd select-crates

Clean up to run a fresh counter download:<br>
rm crates.io-fixed

Download top N crates (N is the parameter passed): <br>
./crate_info_query.sh N

If the file crates.io-fixed exists then it uses it, otherwise it is created. This file contains the information downloaded from crates.io for each crate from crates.io-index repository.

The script parses the file and retains the top N crate names and the downloads in top-N-crates.io.

Next, it downloads using cargo-clone each crate in top-N-crates.io in the directory: /tmp/unsafe_analysis/crates.io-downloads.

<h1>To compile the compiler plugins: </h1>

cd hidden_unsafe; cargo build

<h1>Run the plugin on one crate</h1>
rustup override set nightly-2018-08-29<br>
export PROJECT_HOME="/home/nora/work/external_calls" #change this to your path<br>
export RUSTFLAGS="--extern hidden_unsafe=$PROJECT_HOME/hidden_unsafe/target/debug/libhidden_unsafe.so -Z extra-plugins=hidden_unsafe  --emit mir"<br>
cargo build

<h1>Run examples from repository</h1>
rustup override set nightly-2018-08-29<br>
export PROJECT_HOME="/home/nora/work/external_calls" #change this to your path<br>
export RUSTFLAGS="--extern hidden_unsafe=$PROJECT_HOME/hidden_unsafe/target/debug/libhidden_unsafe.so -Z extra-plugins=hidden_unsafe  --emit mir"<br>

cd $PROJECT_HOME/examples/elf2tbf; cargo build

cd $PROJECT_HOME/examples/hidden_unsafe_tests; cargo build

<h1>Issues:</h1>
1. calls.rs::Operand Type NOT handled move _51

calls.rs::Operand Type NOT handled move _24

calls.rs::Operand Type NOT handled move _3

calls.rs::Operand Type NOT handled move _2

find_callee::Operand Type NOT handled move _3

find_callee::Operand Type NOT handled move _2

2. The package name is bootstrap, instead of core
