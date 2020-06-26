#  Is Rust Used Safely by Software Developers?

This repository contains code used for the ICSE 2020 paper ``Is Rust Used Safely by Software Developers?''.

The sources of the crates we analyzed are available [here](http://www.cs.virginia.edu/~ans5k/icse2020/dataset/crates-2018/)

We implemented a Rust compiler plugin to analyze the sources. The output is available [here](http://www.cs.virginia.edu/~ans5k/icse2020/v1/raw-data/). To reproduce the execution, you will need to install the rust compiler nightly-2018-09-11 and the libraries needed to compile the crates. Next, edit the paths in the ```exports.sh``` file. To launch the compilation use the command ```cd select-crates; ./compile.sh```.

The output is going to be a directory for each crate that contains the analysis results for each crate compiled along with the current crate.

The next step is to select only the top crate by running the command ```cd collect-results; ./copy.sh```. The output is available [here](http://www.cs.virginia.edu/~ans5k/icse2020/v1/results.tgz).

Finally, to get the data ready for the R-scripts that produce the figures and data used in the paper, ```cd collect-results; ./compile.sh run```. The output is available [here](http://www.cs.virginia.edu/~ans5k/icse2020/v1/research-questions.tgz).


<h1>Tools Needed</h1>
1. cargo install clone


