#/bin/bash

# saved source changes for later
# find . -type f -exec sed -i '/license = \"MIT\"/d' {} \;
# find . -type f -exec sed -i '/\[experimental\]/d' {} \;

source ../exports.sh

CRT_DIR=`pwd`

export LD_LIBRARY_PATH=$(rustc --print sysroot)/lib:$LD_LIBRARY_PATH

cd $OSHA_HOME

for crate in $(find $EXCLUDED_CRATES -maxdepth 1 -mindepth 1 -type d -printf '%f\n')
do
	cargo run -- $EXCLUDED_CRATES/$crate/**.rs
	#cargo geiger --no-indent --quiet true | grep $EXCLUDED_CRATES >> $PROJECT_OUT/geiger_results.txt
	RES=$?
	echo "osha returned $RES"
	if [ "$RES" -eq 0 ]
	then
		echo "$crate analysed by osha"
		OUT=`cargo run -- $EXCLUDED_CRATES/$crate/**.rs | tail -n 5 | cut -d' ' -f 3 | sed ':loop;N;s/\n/ /g;t loop'`
		echo $OUT >>  $PROJECT_OUT/geiger_results.txt

		cd $EXCLUDED_CRATES/
		cp -r $crate $GEIGER_CRATES/
		rm -rf $crate

		cd $OSHA_HOME
	else
		echo "$crate is still excluded"
	fi
done
