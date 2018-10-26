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
                   , col.names=c("abi", "crate", "full_path", "name"))

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
all_rust <- nrow(rust)

core <- subset( rust, rust$crate == "core" )
core_sum <- nrow(core)  
core_percentage <- core_sum/all_rust * 100
filename <- "~/work/unsafe_study/paper/rq06_core_per.txt" 
write(formatC(core_percentage,digits = 1, format = "f"), file=filename)

std <- subset( rust, rust$crate == "std" )
std_sum <- nrow(std)
std_percentage <- std_sum/all_rust * 100
filename <- "~/work/unsafe_study/paper/rq06_std_per.txt" 
write(formatC(std_percentage,digits = 1, format = "f"), file=filename)

alloc <-  subset( rust, rust$crate == "alloc" )
alloc_sum <- nrow(alloc)
alloc_percentage <- alloc_sum/all_rust
filename <- "~/work/unsafe_study/paper/rq06_alloc_per.txt" 
write(formatC(alloc_percentage,digits = 1, format = "f"), file=filename)


unsafe_fn_ptr <- subset(rust, rust$name == "Unsafe_Call_Fn_Ptr")
unsafe_fn_ptr_percentage <- nrow(unsafe_fn_ptr) / all_rust
filename <- "~/work/unsafe_study/paper/rq06_unsafe_ptr_per.txt" 
write(formatC(unsafe_fn_ptr_percentage,digits = 1, format = "f"), file=filename)
