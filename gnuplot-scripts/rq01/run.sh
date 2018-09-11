#/bin/bash

if [ -z "$SCRIPTS_HOME" ]
then
        SCRIPTS_HOME="$HOME/work/unsafe_study/gnuplot-scripts"
fi
source $SCRIPTS_HOME/common.sh

# format:
# crate_name unsafe_bb total_bb unsafe total
FILE=/tmp/unsafe_analysis/research_questions/rq01
rm -f data/*.txt
mkdir -p data

count $FILE 2 "$SCRIPTS_HOME/data/rq01-1.txt"
count $FILE 3 "$SCRIPTS_HOME/data/rq01-2.txt"
count $FILE 4 "$SCRIPTS_HOME/data/rq01-3.txt"
count $FILE 5 "$SCRIPTS_HOME/data/rq01-4.txt"

gnuplot "$SCRIPTS_HOME/rq01/count.p"

gnuplot "$SCRIPTS_HOME/rq01/freq.p"
