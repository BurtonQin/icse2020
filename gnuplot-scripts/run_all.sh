#/bin/bash

source ../exports.sh

mkdir -p $PAPER_RESULTS_DIR

./syntactic_only.R

./rq01.R
./rq02.R
./rq03.R
./rq04.R
./rq05.R
#./rq06/run.sh
#./rq09/run.sh
