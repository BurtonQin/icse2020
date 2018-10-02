#!/usr/bin/env Rscript
library(ggplot2)
library(Hmisc)
library(plyr)
library('scales')

p <- pipe(paste0('sed \'s/"\'"/"`"/g\' "', FILENAME, '"'))

res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq06"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , quote="\\"
                   , col.names=c("abi", "call"))

c_calls <-  subset( res, res$abi == "C" )
c_calls_aggregate <- count(c_calls,'call')

intrinsics <- subset( res, res$abi == "RustIntrinsic" )
intrinsics_aggregate <- count(intrinsics,'call')

rust <- subset( res, res$abi == "Rust" )
rust_aggregate <- count(rust,'call')