export RUSTFLAGS="--extern unsafe_analysis=$PROJECT_HOME/unsafe-analysis-2019/target/debug/libunsafe_analysis.so -Z extra-plugins=unsafe_analysis -Z always-encode-mir"

