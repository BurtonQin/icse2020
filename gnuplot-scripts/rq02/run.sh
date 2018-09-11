#/bin/bash

if [ -z "$SCRIPTS_HOME" ]
then
        SCRIPTS_HOME="$HOME/work/unsafe_study/gnuplot-scripts"
fi
source $SCRIPTS_HOME/common.sh

#crate_name unsafe_functions_percentage unsafe_functions_count
FILE=/tmp/unsafe_analysis/research_questions/rq02
rm -f data/*.txt
mkdir -p data

count $FILE 3 "$SCRIPTS_HOME/data/rq02-count.txt"
count $FILE 2 "$SCRIPTS_HOME/data/rq02-freq.txt"

gnuplot "$SCRIPTS_HOME/rq02/plot.p"
