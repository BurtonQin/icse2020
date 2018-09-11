#/bin/bash

if [ -z "$SCRIPTS_HOME" ]
then
	SCRIPTS_HOME="$HOME/work/unsafe_study/gnuplot-scripts"
fi
source $SCRIPTS_HOME/common.sh

#crate_name unsafe_functions_percentage unsafe_functions_count
FILE=/tmp/unsafe_analysis/research_questions/rq04-summary

rm -f data/*.txt
mkdir -p data

values=(`cat $FILE`)
strs=( "Total Blocks" "Unsafe Function Call" "Dereference Raw Pointer" "Asm Block" "Access to Static" "Packed Borrow" "Assignment to Non-Copy Union Field" "Access To Union" "Extern Static" )

echo "\\begin{table}[h]"
echo "\\centering"
echo "\\begin{tabular}{| l | l |}"
echo "\\hline"

for i in `seq 0 8`
do
	echo "${strs[i]}  & ${values[i]} \\\\"
	echo "\\hline"
done


echo "\\end{tabular}"
echo "\\caption{Percentage of Unsafe Blocks for Each Reason}"
echo "\\label{table:unsafe-sources-blocks}"
echo "\\end{table}"

