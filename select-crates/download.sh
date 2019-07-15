#!/bin/bash


export PROJECT_OUT=${HOME}/unsafe_analysis
export CRATES_DIR=${PROJECT_OUT}/crates.io-downloads

while read line
do
        crate=$(echo $line | cut -d' ' -f 1)
        echo "cloning crate $crate"
        cargo clone $crate --prefix $CRATES_DIR
	sleep 30
done <crates.io-90-percent-2018

