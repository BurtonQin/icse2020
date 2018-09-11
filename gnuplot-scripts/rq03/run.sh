#/bin/bash

if [ -z "$SCRIPTS_HOME" ]
then
	SCRIPTS_HOME="$HOME/work/unsafe_study/gnuplot-scripts"
fi
source $SCRIPTS_HOME/common.sh

#crate_name unsafe_functions_percentage unsafe_functions_count
FILE=/tmp/unsafe_analysis/research_questions/rq03

rm -f data/*.txt
mkdir -p data

count_absolute $FILE 2 "$SCRIPTS_HOME/rq03/data/rq03-count.txt"

STR=`awk 'BEGIN {none=0;one=0;two=0;more=0;};
 
 {
	if ($1==0) {none=$2;}
	else if ($1==1) {one=$2;}
	else if ($1==2) {two=$2;}
	else {more+=$2;}
}
 
END {print none, one, two, more }' $SCRIPTS_HOME/rq03/data/rq03-count.txt`
array=($STR)

echo "\\begin{table}[h]"
echo "\\centering"
echo "\\begin{tabular}{| c | c | c | c |}"
echo "\\hline"
echo  "No Unsafe Traits & One & Two & More Than Two  \\\\ "
echo "\\hline"
echo "${array[0]} & ${array[1]} & ${array[2]} & ${array[3]}\\\\"
echo "\\hline"
echo "\\end{tabular}"
echo "\\caption{Unsafe Traits}"
echo "\\label{table:unsafe-traits}"
echo "\\end{table}"

