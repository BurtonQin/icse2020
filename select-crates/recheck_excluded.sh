
source ../exports.sh

CRT_DIR=`pwd`

export RUST_BACKTRACE=1
export RUST_LOG=error

cd $EXCLUDED_CRATES

for d in $(find $EXCLUDED_CRATES -maxdepth 1 -mindepth 1 -type d -printf '%f\n')
do 
	echo "Compiling $d"
	if [ -d "$EXCLUDED_CRATES/$d" ]
	then
                cd $EXCLUDED_CRATES/$d
                cargo +$NIGHTLY build
                RESULT=$?
                if [ $RESULT -eq 0 ]; then
			echo "$d: NORA Passed"
			cd ..
			cp -r $EXCLUDED_CRATES/$d $CRATES_DIR/
			rm -rf $EXCLUDED_CRATES/$d
                else
                        echo "$d: NORA Failed"
                fi
                cargo clean
                cd $EXCLUDED_CRATES
	else 
		echo "$d does not exists"
	fi
done


