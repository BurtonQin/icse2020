#!/usr/bin/env Rscript
library(ggplot2)
library(reshape2)
library(plyr)
library(Hmisc)
library(scales)

res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq01-restricted-func"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("functions", "unsafe", "name"))
res90 <- read.table( file="~/unsafe_analysis/analysis-data/research-questions-90-percent/rq01-restricted-func"
                     , header=FALSE
                     , sep='\t'
                     , comment.char = "#"
                     , col.names=c("functions", "unsafe", "name"))
eq <- sum(res$functions == 0)
print(eq/nrow(res))

eq90 <- sum(res90$functions == 0)
print(eq90/nrow(res90))
