#!/usr/bin/env Rscript
library(ggplot2)
library(Hmisc)
library(plyr)
library(scales)
library(data.table)
library(DescTools)

p <- pipe(paste0('sed \'s/"\'"/"`"/g\' "', "~/unsafe_analysis/analysis-data/research-questions/rq06", '"'))

res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq06"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , quote="\\"
                   , col.names=c("abi", "full_path", "name"))

c_calls <-  subset( res, res$abi == "C" )
c_calls_aggregate <- count(c_calls,'full_path')
c_summary <- quantile(c_calls_aggregate$freq, c(.50,.75,.95))

intrinsics <- subset( res, res$abi == "RustIntrinsic" )
intrinsics_aggregate <- count(intrinsics,'call')

rust <- subset( res, res$abi == "Rust" )
rust_aggregate <- count(rust,'call')

core_sum <- sum(rust_aggregate[which(rust_aggregate$call %like any% c("^core::%","^<core::%")),"freq"])
std_sum <- sum(rust_aggregate[which(rust_aggregate$call %like any% c("^std::%","^<std::%")),"freq"])
alloc_sum <- sum(rust_aggregate[which(rust_aggregate$call %like any% c("^alloc::%","^<alloc::%")),"freq"])
all_rust <- nrow(rust)
core_percentage <- core_sum/all_rust
std_percentage <- std_sum/all_rust
alloc_percentage <- alloc_sum/all_rust
unsafe_fn_ptr <- sum(rust_aggregate[which(rust_aggregate$call %like% c("Unsafe_Call_Fn_Ptr")),"freq"]) / all_rust
