function update {
	if [ -d $1 ]
	then
        	cd $1
	        git pull
		cd ..
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
update alacritty https://github.com/jwilm/alacritty.git
update ripgrep https://github.com/BurntSushi/ripgrep.git
update xray https://github.com/atom/xray.git
update fd https://github.com/sharkdp/fd.git
update leaf https://github.com/autumnai/leaf.git
update Rocket https://github.com/SergioBenitez/Rocket.git

cd $CRT_DIR
