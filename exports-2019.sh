source ~/.bash_profile

export PROJECT_HOME=$HOME/work/unsafe_study
#export OSHA_HOME=$HOME/work/cargo-osha
export CRATES_IO_INDEX_HOME=${HOME}/work/crates.io-index

export PROJECT_OUT=${HOME}/unsafe_analysis
export CRATES_DIR=${PROJECT_OUT}/crates.io-downloads
#export EXCLUDED_CRATES=${PROJECT_OUT}/excluded-crates
#export SYNTACTIC_ONLY_CRATES=${PROJECT_OUT}/syntactic-only
#export GITHUB_APPS=${PROJECT_OUT}/github-downloads

export UNSAFE_ANALYSIS_DIR=${PROJECT_OUT}/analysis-data
export FULL_ANALYSIS_DIR=${UNSAFE_ANALYSIS_DIR}/full-analysis

export ANALYSIS_RESULTS_DIR=$UNSAFE_ANALYSIS_DIR/results

export RQ_DIR=${UNSAFE_ANALYSIS_DIR}/research-questions

export PAPER_RESULTS_DIR=$PROJECT_HOME/paper/

export NIGHTLY=nightly-2019-07-01
# targets: thumbv7em-none-eabihf

export RUST_LOG=unsafe_analysis=debug,rustc=error
#export RUST_LOG=unsafe_analysis=debug,rustc=debug

