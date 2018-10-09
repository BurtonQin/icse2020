#!/usr/bin/env Rscript
library(ggplot2)
library(Hmisc)
library(plyr)
library(scales)
library(data.table)

p <- pipe(paste0('sed \'s/"\'"/"`"/g\' "', "~/unsafe_analysis/analysis-data/research-questions/rq06", '"'))

res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq06"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , quote="\\"
                   , col.names=c("abi", "call"))

c_calls <-  subset( res, res$abi == "C" )
c_calls_aggregate <- count(c_calls,'call')
c_summary <- quantile(c_calls_aggregate$freq, c(.50,.75,.95))

intrinsics <- subset( res, res$abi == "RustIntrinsic" )
intrinsics_aggregate <- count(intrinsics,'call')

rust <- subset( res, res$abi == "Rust" )
rust_aggregate <- count(rust,'call')

core_sum <- sum(rust_aggregate[which(rust_aggregate$call %like% "^core::"),"freq"])
std_sum <- sum(rust_aggregate[which(rust_aggregate$call %like% "^std::"),"freq"])
all_rust <- nrow(rust)