#/bin/bash

# $1 the filename of data per crate
# $2 the column number to be processed
# $3 ouput filename
# each line has format: 
# percentage_of_crates value
function count {
	TOTAL_CRATES=`cat $1 | wc -l`
        for f in $(cut -f$2 $1 | sort -u)
        do
                t=`cut -f$2 "$1" | grep "$f" | wc -l`
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

