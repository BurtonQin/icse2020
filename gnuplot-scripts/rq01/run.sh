#/bin/bash

if [ -z "$SCRIPTS_HOME" ]
then
        SCRIPTS_HOME="$HOME/work/unsafe_study/gnuplot-scripts"
fi
source $SCRIPTS_HOME/common.sh

# format:
# crate_name unsafe_bb_percent unsafe_bb unsafe_percent unsafe
FILE=/tmp/unsafe_analysis/research_questions/rq01

DATA=/tmp/unsafe_analysis/
rm -f data/*.txt
mkdir -p data

TOTAL_CRATES=`cat $FILE | wc -l`
awk -F '\t' '{print $3}' $FILE | sort | uniq -c | sort -nr

# count $FILE 3 "$DATA/rq01-2.txt"
# count $FILE 5 "$DATA/rq01-4.txt"

# gnuplot "$SCRIPTS_HOME/rq01/count.p" 

