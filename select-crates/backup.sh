#/bin/bash

source ../exports.sh


pushd ${UNSAFE_ANALYSIS_DIR}/full-analysis/

echo "${UNSAFE_ANALYSIS_DIR}/full-analysis/"

for x in {a..z}
do
	pushd ${UNSAFE_ANALYSIS_DIR}/full-analysis/
	tar czf ${x}.tgz ${x}*
	scp ${x}.tgz ans5k@portal.cs.virginia.edu:~/
done

popd

