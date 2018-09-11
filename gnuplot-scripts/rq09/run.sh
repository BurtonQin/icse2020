#/bin/bash

if [ -z "$SCRIPTS_HOME" ]
then
	SCRIPTS_HOME="$HOME/work/unsafe_study/gnuplot-scripts"
fi
source $SCRIPTS_HOME/common.sh

#crate_name unsafe_functions_percentage unsafe_functions_count
FILE=/tmp/unsafe_analysis/research_questions/rq09
DATA="$SCRIPTS_HOME/rq09/data"

rm -f $DATA/*.txt
mkdir -p $DATA

values[0]=`cat $FILE |  wc -l`
for i in `seq 3 12`
do
    values[$((i-2))]=`cat $FILE | cut -f $i | grep true |  wc -l`
    if [ -z  values[$((i-2))] ]
    then
	values[$((i-2))]=0
    fi
done

strs=( "Total Functions" "Unsafe Function Call" "Dereference Raw Pointer" "Asm Block" "Access to Static" "Packed Borrow" "Assignment to Non-Copy Union Field" "Access To Union" "Extern Static" "Argument" "FromTrait")

echo "\\begin{table}[h]"
echo "\\centering"
echo "\\begin{tabular}{| l | l |}"
echo "\\hline"

for i in `seq 0 10`
do
	echo "${strs[i]}  & ${values[i]} \\\\"
	echo "\\hline"
done


echo "\\end{tabular}"
echo "\\caption{Percentage of Unsafe Blocks for Each Reason}"
echo "\\label{table:unsafe-sources-blocks}"
echo "\\end{table}"



# for f in $(cut -f1 $FILE | sort -u) #for each crate
# do
#     t=`grep "$f" $FILE | cut -f3 | grep true | wc -l`
#     echo "$f $t" >> $DATA/unsafe_calls.txt
# done
# $1 the filename of data per crate
# $2 the column number to be processed
# $3 ouput filename
# each line has format: 
# percentage_of_crates value
function count_absolute {
    for f in $(cut -f1 $1 | sort -u) #for each crate
    do
	#1. select lines for current crate
	#2. select the column
	#3. keep only the true values
	#4. count
        t=`grep "$f" "$1" | cut -f$2 | grep true | wc -l`
	echo "$f $t" >> $3
    done
}


count_absolute "$FILE" 3 "$DATA/unsafe_calls.data"
count_absolute "$FILE" 4 "$DATA/deref_raw.data"
count_absolute "$FILE" 6 "$DATA/static.data"

gnuplot "$SCRIPTS_HOME/rq09/plot.p"
