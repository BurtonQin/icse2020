#!/usr/bin/env Rscript
library(plyr)
library(ggplot2)
library(reshape2)
library(Hmisc)
library(scales)

#dir <- '~/unsafe_analysis/analysis-data/research-questions/'
#dir <- '~/unsafe_analysis/analysis-data/research-questions-90-percent/'
dir <- '~/work/unsafe-analysis-data/research-questions/servo/research-questions-servo-all'

res <- read.table( file=paste(dir,"rq01", sep='')
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("blocks", "user", "total", "name"))
crates <- res$name

blocks <- subset(res, user>0)
crates_blocks <- blocks$name

res <- read.table( file=paste(dir,"rq02", sep='')
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("functions", "name"))
functions <- subset(res,functions>0)
crates_functions <- functions$name

crates <- union(crates, res$name)


res <- read.table(  file=paste(dir,"rq03-traits", sep='')
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("crate", "count"))
traits <- subset(res, count>0)
crates_traits <- traits$crate
crates <- union(crates, res$crate)

res <- read.table( file=paste(dir,"rq03-impls", sep='')
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("crate", "count"))
impls <- subset(res, count>0)
crates_impls <- impls$crate
crates <- union(crates, res$crate)

all_crates <- union(crates_blocks, crates_functions)
all_crates <- union(all_crates, crates_traits)
all_crates <- union(all_crates, crates_impls)

print(length(all_crates)/length(crates))
