#!/usr/bin/env Rscript
library(ggplot2)
library(Hmisc)
library(dplyr)
library(scales)
library(data.table)
library(DescTools)
library(xtable)

p <- pipe(paste0('sed \'s/"\'"/"`"/g\' "', "~/unsafe_analysis/analysis-data/research-questions/rq06", '"'))

res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq06"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , quote="\\"
                   , col.names=c("abi", "full_path", "name"))

c_calls <-  subset( res, res$abi == "C" )
c_calls_aggregate <- summarise( group_by( c_calls,full_path), n=n())

filename <- "~/work/unsafe_study/paper/rq06_c.txt" 
write(formatC(nrow(c_calls)/nrow(res)*100,digits = 1, format = "f"), file=filename)

# intrinsics

intrinsics <- subset( res, res$abi == "RustIntrinsic" )
intrinsics_aggregate <- summarise( group_by( intrinsics,name), n=n())

filename <- "~/work/unsafe_study/paper/rq06_intrinsics_percent.txt" 
write(formatC(nrow(intrinsics)/nrow(res)*100,digits = 1, format = "f"), file=filename)

top5 <- top_n( intrinsics_aggregate, n=5 )
top5$n <- formatC(top5$n/nrow(intrinsics)*100,digits=1, format = "f")
colnames(top5) <- c("Function", "Percentage")

filename <- "~/work/unsafe_study/paper/rq06_intrinsics_table.txt" 
xx <- xtable(top5,caption = "Top intrinsics calls", label="tbl:allintrinsics", filename=filename)
print(xx,file=filename)

# Rust
rust <- subset( res, res$abi == "Rust" )
rust_aggregate <- summarise( group_by( rust,name), n=n())

top5 <- top_n( rust_aggregate, n=5 )
top5$n <- formatC(top5$n/nrow(rust)*100,digits=1, format = "f")
colnames(top5) <- c("Function", "Percentage")

core_sum <- nrow(rust[which(rust$name %like any% c("^core::%","^<core::%"))])
std_sum <- nrow(rust[which(rust$name %like any% c("^std::%","^<std::%"))])
alloc_sum <- nrow(rust[which(rust$name %like any% c("^alloc::%","^<alloc::%"))])
all_rust <- nrow(rust)
core_percentage <- core_sum/all_rust
std_percentage <- std_sum/all_rust
alloc_percentage <- alloc_sum/all_rust
unsafe_fn_ptr <- sum(rust_aggregate[which(rust_aggregate$call %like% c("Unsafe_Call_Fn_Ptr")),"freq"]) / all_rust
