#/bin/bash

# $1 the filename of data per crate
# $2 the column number to be processed
# $3 ouput filename
# each line has format: 
# percentage_of_crates value
function count {
    TOTAL_CRATES=`cat $1 | wc -l`
    for f in $(cut -f$2 $1 | sort -u) #for each unique value in column $2
    do
        t=`awk 'BEGIN {count=0};
 	{
		if ($1==0) {none=$2;}
	else if ($1==1) {one=$2;}
	else if ($1==2) {two=$2;}
	else {more+=$2;}
}
 
END {print none, one, two, more }' $SCRIPTS_HOME/rq03/data/rq03-count.txt`
        p=`echo "scale=4; $t/$TOTAL_CRATES*100" | bc`
        echo "$f $p" >> $3
    done
}

function count_absolute {
	for f in $(cut -f$2 $1 | sort -u)
        do
                t=`cut -f$2 "$1" | grep "$f" | wc -l`
		echo "$f $t" >> $3
	done
}

