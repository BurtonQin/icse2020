# external_calls

external_calls contains the rustc linters

cargo-safety_check contains the cargo plugin

To compile the compiler plugins: 
cd external_calls
cargo build

To see the run on a copy of TockOS:
cd examples/elf2tbf
cargo build
