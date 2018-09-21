source ../exports.sh

unset RUSTFLAGS

cargo +$NIGHTLY $1 

