# external_calls

<h1>Tools Needed</h1>
1. cargo install clone

<h1>Download top N crates from crates.io</h1>

Execute: cd select-crates

Clean up to run a fresh counter download:<br> rm crates.io-fixed

Download top N crates (N is the parameter passed): <br>
./crates_select_and_download.sh N

If the file crates.io-fixed exists then it uses it, otherwise it is
created. This file contains the information downloaded from crates.io
for each crate from crates.io-index repository.

The script parses the file and retains the top N crate names and the
downloads in top-N-crates.io.

Next, it downloads each crate in top-N-crates.io in the directory:
/tmp/unsafe_analysis/crates.io-downloads.It uses cargo clone.

<h1>Compilation</h1>

cd unsafe-analysis/; ./compile.sh build <br>

<h1>Run Analysis</h1>
export PROJECT_HOME="$HOME/work/unsafe_study" #change this to your path<br>
cd $PROJECT_HOME/select-crates<br>
./crates_select_and_download.sh 500<br>
./compile.sh<br>

cd $PROJECT_HOME/github-downloads<br>
./download.sh<br>
./compile.sh<br>

<h1>Run the plugin on one crate</h1>

rustup override set nightly-2018-08-29<br>
export PROJECT_HOME="$HOME/work/unsafe_study" #change this to your path<br>
export RUSTFLAGS="--extern unsafe_analysis=$PROJECT_HOME/unsafe-analysis/target/debug/libunsafe-analysis.so -Z extra-plugins=unsafe-analysis --emit mir"<br>
cargo build

<h1>Run examples from repository</h1>

rustup override set nightly-2018-08-29<br>
export PROJECT_HOME="$HOME/work/unsafe_study" #change this to your path<br>
export RUSTFLAGS="--extern unsafe_analysis=$PROJECT_HOME/unsafe-analysis/target/debug/libunsafe_analysis.so -Z extra-plugins=unsafe_analysis --emit mir"<br>

cd $PROJECT_HOME/examples/elf2tbf; cargo build

cd $PROJECT_HOME/examples/tests; ./compile.sh

<h1>Issues:</h1> 