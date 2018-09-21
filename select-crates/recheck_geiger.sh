
source ../exports.sh

CRT_DIR=`pwd`

export RUST_BACKTRACE=1
export RUST_LOG=error

cd $GEIGER_CRATES

for d in $(find $GEIGER_CRATES -maxdepth 1 -mindepth 1 -type d -printf '%f\n')
do 
	echo "Compiling $d"
	if [ -d "$GEIGER_CRATES/$d" ]
	then
                cd $GEIGER_CRATES/$d
                cargo +$NIGHTLY build
                RESULT=$?
                if [ $RESULT -eq 0 ]; then
			echo "$d: NORA Passed"
			cd ..
			cp -r $GEIGER_CRATES/$d $CRATES_DIR/
			rm -rf $GEIGER_CRATES/$d
                else
                        echo "$d: NORA Failed"
                fi
                cargo clean
                cd $GEIGER_CRATES
	else 
		echo "$d still does not compile"
	fi
done


