#/bin/bash

source ../exports.sh

CRT_DIR=`pwd`

export LD_LIBRARY_PATH=$(rustc --print sysroot)/lib:$LD_LIBRARY_PATH

mkdir -p $EXCLUDED_CRATES
mkdir -p $GEIGER_CRATES

for crate in $(find $CRATES_IO_INDEX_HOME -type f -printf '%f\n')
do
	FOUND=`find $CRATES_DIR  -maxdepth 1  -name $crate`
	if [ -z "$FOUND" ] 
	then
		echo "cloning crate $crate"
		rm -rf $GEIGER_CRATES/$crate
		cargo clone $crate --prefix $GEIGER_CRATES

		if [ -d $GEIGER_CRATES/$crate ]
		then 

			cd $GEIGER_CRATES/$crate
			find . -type f -exec sed -i '/license = \"MIT\"/d' {} \;
			find . -type f -exec sed -i '/\[experimental\]/d' {} \;

			cargo geiger --no-indent --quiet true | grep $crate >> $PROJECT_OUT/geiger_results.txt
	
			RES=$?
			if [ -z $RES ]
			then
				echo "$crate analysed by geiger"
			else
				cd $PROJECT_OUT
				cp -r $GEIGER_CRATES/$crate $EXCLUDED_CRATES
				rm -rf $GEIGER_CRATES/$crate
				echo "$crate is excluded"
				echo $crate >> $PROJECT_OUT/excluded_crates.txt
				cd $EXCLUDED_CRATES/$crate/src
				FUNCTIONS=`grep -r -w "\<unsafe[[:space:]]fn\>" *.rs | wc -l`
				TRAITS=`grep -r -w "\<unsafe[[:space:]]trait\>" *.rs | wc -l`
				IMPLS=`grep -r -w "\<unsafe[[:space:]]impl\>" *.rs | wc -l`
				ALL=`grep -r -w "\<unsafe\>" *.rs | wc -l`
				echo "$FUNCTIONS $TRAITS $IMPLS $ALL $crate" >> $PROJECT_OUT/grep_results.txt
			fi
		fi
	else
		echo "$crate is included"
	fi
done
