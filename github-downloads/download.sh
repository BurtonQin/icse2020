function update {
	if [ -d $1 ]
	then
        	cd $1
	        git pull
	else
        	git clone $2
	fi
}

CRT_DIR=`pwd`

HOME_DIR=/tmp/unsafe_analysis/github-downloads

mkdir -p $HOME_DIR

cd $HOME_DIR

update xi-editor https://github.com/google/xi-editor
update servo https://github.com/servo/servo.git

cd $CRT_DIR
