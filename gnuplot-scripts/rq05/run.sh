#/bin/bash

if [ -z "$SCRIPTS_HOME" ]
then
	SCRIPTS_HOME="$HOME/work/unsafe_study/gnuplot-scripts"
fi
source $SCRIPTS_HOME/common.sh

#crate_name unsafe_functions_percentage unsafe_functions_count
FILE=/tmp/unsafe_analysis/research_questions/rq05
TOP=20

rm -f data/*.txt
mkdir -p data

TOTAL=`awk 'BEGIN {FS = "\t"} ; {sum+=$2} END {print sum}' $FILE`

echo "\\begin{table}[h]"
echo "\\centering"
echo "\\begin{tabular}{| c | c |}"
echo "\\hline"
echo  "Function & Count \\\\ "
echo "\\hline"

while IFS= read -r f; do
	read -ra arr <<<"$f"	
    	echo "${arr[0]} & ${arr[1]} \\\\"
done < <(sort -k2 -n -r "$FILE" | head -"$TOP")

echo "\\hline"
echo "\\end{tabular}"
echo "\\caption{Most Frequent Calls (Total $TOTAL)}"
echo "\\label{table:top-unsafe-calls}"
echo "\\end{table}"

