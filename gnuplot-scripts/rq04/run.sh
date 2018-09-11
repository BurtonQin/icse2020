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

array=(`cat $FILE`)


echo "\\begin{table}[h]"
echo "\\centering"
echo "\\begin{tabular}{| c | c | c | c | c | c | c | c | c |}"
echo "\\hline"
echo  "Total Blocks & Unsafe Function Call & Dereference Raw Pointer & Asm Block & Access to Static & Packed Borrow & Assignment to Non-Copy Union Field & Access To Union & Extern Static  \\\\ "
echo "\\hline"


#if [ -z array[0] ]
#then 
#	echo "ERROR: no unsafe reasons in block"
#else 
#	echo -n `echo "scale=2; ${array[1]}/${array[0]}*100" | bc`
#	for i in `seq 2 8`
#	do
#		echo -n " & "
#		echo -n `echo "scale=2; ${array[i]}/${array[0]}*100" | bc`
#	done
#	echo ""
#fi
echo -n "${array[0]}"
for i in `seq 1 8`
do
	echo -n " & "
        echo -n "${array[i]}"
done
echo "\\\\"


echo "\\hline"
echo "\\end{tabular}"
echo "\\caption{Percentage of Unsafe Blocks for Each Reason}"
echo "\\label{table:unsafe-sources-blocks}"
echo "\\end{table}"

