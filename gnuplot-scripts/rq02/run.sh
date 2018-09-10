#/bin/bash

source ../common.sh

#crate_name unsafe_functions_percentage unsafe_functions_count
FILE=/tmp/unsafe_analysis/research_questions/rq02
rm -f data/*.txt
mkdir -p data

count $FILE 3 "data/rq02-count.txt"
count $FILE 2 "data/rq02-freq.txt"

gnuplot plot.p
