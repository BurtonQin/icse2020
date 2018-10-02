#!/usr/bin/env Rscript
library(ggplot2)
library(Hmisc)

res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq03"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("impls", "name"))


nonzero_filename <- "~/work/unsafe_study/paper/rq03_some.txt" 
zero_filename <- "~/work/unsafe_study/paper/rq03_none.txt" 
all_filename <- "~/work/unsafe_study/paper/rq03_n.txt" 

zero_frame <- subset(res,res$impls==0)
some_frame <- subset(res,res$impls!=0)

nonzero <- sum(some_frame$impls)
zero <- nonzero <- sum(zero_frame$impls)
all <- sum(res$impls)

write(nonzero, file=nonzero_filename)
write(zero, file=zero_filename)
write(all, file=all_filename)