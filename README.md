# external_calls

hidden_insafe contains the rustc linters

cargo-safety_check contains the cargo plugin

To compile the compiler plugins: 

cd hidden_unsafe

cargo build

To see the run on a copy of TockOS elf2tbf tool:

cd examples/elf2tbf

cargo build

To add the analysis to a new project:

Add in the source file:

#![feature(plugin)]

#![plugin(hidden_unsafe)]


Add in the Cargo.toml:

[dependencies]

hidden_unsafe = { path = "/home/nora/work/external_calls/hidden_unsafe" }

Compile with: cargo +nightly build
