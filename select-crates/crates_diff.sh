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

		cd $GEIGER_CRATES/$crate
		find . -type f -exec sed -i '/license = \"MIT\"/d' {} \;
		find . -type f -exec sed -i '/\[experimental\]/d' {} \;

		cargo geiger --no-indent --quiet true | grep $crate >> $PROJECT_OUT/geiger_results.txt

		RES=$?
		if [ -z $RES ]
		then
			echo "$crate analysed by geiger"
		else
			cd ../
			cp -r $crate $EXCLUDED_CRATES
			rm -rf $crate
			echo "$crate is excluded"
			echo $crate >> $PROJECT_OUT/excluded_crates.txt
		fi
	else
		echo "$crate is included"
	fi
done
