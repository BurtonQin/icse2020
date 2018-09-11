#/bin/bash

if [ -z "$SCRIPTS_HOME" ]
then
	SCRIPTS_HOME="$HOME/work/unsafe_study/gnuplot-scripts"
fi
source $SCRIPTS_HOME/common.sh

#crate_name unsafe_functions_percentage unsafe_functions_count
FILE=/tmp/unsafe_analysis/research_questions/rq06

rm -f data/*.txt
mkdir -p data

awk 'BEGIN {none=0;total=0;less_ten=0;};
 {
	total += 1;
	if ($2==0) {none+=1;}
	else if ($2<10) {less_ten+=1;}
} 
END {print "Total:",total,"None:", none, "Less than 10:", less_ten }' $FILE


echo "\\begin{table}[h]"
echo "\\centering"
echo "\\begin{tabular}{| l | l |}"
echo "\\hline"
awk 'BEGIN {};
 {
	if ($2>=10) {
	   print $1, "&", $2, "\\\\"
	   print "\\hline"
	}
} 
END {}' $FILE
echo "\\end{tabular}"
echo "\\caption{Crates with more than ten no reason for unsafe functions}"
echo "\\label{table:no-reason}"
echo "\\end{table}"
