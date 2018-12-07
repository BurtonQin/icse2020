source ../exports.sh

pushd $FULL_ANALYSIS_DIR

for x in {a..z}
do
	echo "archiving ${x}*"
	tar czf ${x}.tgz ${x}*
	mv ${x}.tgz $UNSAFE_ANALYSIS_DIR
done

popd
