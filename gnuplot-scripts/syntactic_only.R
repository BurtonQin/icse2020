#!/usr/bin/env Rscript
library(ggplot2)
library(Hmisc)
library(scales)

func_filename <- "~/work/unsafe_study/paper/syntactic_func_none.txt"
block_filename <- "~/work/unsafe_study/paper/syntactic_block_none.txt"
trait_filename <- "~/work/unsafe_study/paper/syntactic_trait_none.txt"

res <- read.table( file="~/unsafe_analysis/analysis-data/syntactic_only_results.txt"
                   , header=FALSE
                   , sep=','
                   , comment.char = "#"
                   , col.names=c("functions", "traits", "blocks", "name"))

f <- nrow(res[res$functions==0])
b <- nrow(res[res$blocks==0])
t <- nrow(res[res$traits==0])

write(f, filename=func_filename)
write(b, filename=block_filename)
write(t, filename=trait_filename)