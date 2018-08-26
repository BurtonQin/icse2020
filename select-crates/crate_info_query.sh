#/bin/bash

CRATES_IO_INDEX_HOME=/home/nora/work/crates.io-index
for crate in $(find $CRATES_IO_INDEX_HOME -printf '%p\n')
do
	if [ -f $crate ]
	then 
		curl https://crates.io/api/v1/crates/$crate
		#echo `basename $crate`
		echo ""
	fi
done
