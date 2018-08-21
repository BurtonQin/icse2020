# external_calls

hidden_insafe contains the rustc linters

cargo-safety_check contains the cargo plugin (not yet implemented)

<b>To compile the compiler plugins: </b>

cd hidden_unsafe; cargo build

<b>To see the run on a copy of TockOS elf2tbf tool: </b>

cd examples/elf2tbf; cargo build

<b>To add the analysis to a new project:</b>

1. Add in the source file:

#![feature(plugin)]

#![plugin(hidden_unsafe)]


2. Add in the Cargo.toml:

[dependencies]

hidden_unsafe = { path = "/home/nora/work/external_calls/hidden_unsafe" }

3. Compile with: cargo +nightly build

<b> TockOS Analysis: </b>
1. tock_cells

cd cd libraries/tock-cells/; cargo +nightly build

2. kernel 
Edits: removed unique feature

cd kernel; export TOCK_KERNEL_VERSION=$(git describe --always || echo notgit); cargo +nightly build

3. core

cd $RUST_SRC/src/libcore; cargo +nightly build

Issues:
1. calls.rs::Operand Type NOT handled move _51

calls.rs::Operand Type NOT handled move _24

calls.rs::Operand Type NOT handled move _3

calls.rs::Operand Type NOT handled move _2

find_callee::Operand Type NOT handled move _3

find_callee::Operand Type NOT handled move _2

2. The package name is bootstrap, instead of core
