#/bin/bash

source ../common.sh

# format:
# crate_name unsafe_bb total_bb unsafe total
FILE=/tmp/unsafe_analysis/research_questions/rq01
rm -f data/*.txt
mkdir -p data

count $FILE 2 "data/rq01-1.txt"
count $FILE 3 "data/rq01-2.txt"
count $FILE 4 "data/rq01-3.txt"
count $FILE 5 "data/rq01-4.txt"

gnuplot count.p

gnuplot freq.p
