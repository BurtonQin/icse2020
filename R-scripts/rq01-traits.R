#!/usr/bin/env Rscript
library(ggplot2)
library(Hmisc)
library(DescTools)

#Traits
res <- read.table( #file="~/unsafe_analysis/analysis-data/research-questions/rq03-traits"
                   file="/home/nora/work/unsafe-analysis-data/research-questions/servo/research-questions-servo-all/rq03-traits"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("crate", "count"))
res90 <- read.table(# file="~/unsafe_analysis/analysis-data/research-questions-90-percent/rq03-traits"
  file="/home/nora/work/unsafe-analysis-data/research-questions/servo/research-questions-servo-all/rq03-traits"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("crate", "count"))

nonzero_filename <- "~/work/unsafe-analysis-data/paper/rq01_traits_some.txt" 
nonzero90_filename <- "~/work/unsafe-analysis-data/paper/rq01_traits_some90.txt" 

nonzero <- length(res$count[res$count!=0])/nrow(res)
nonzero90 <- length(res90$count[res90$count!=0])/nrow(res90)

write(formatC(nonzero*100,digits = 1, format = "f"), file=nonzero_filename)
write(formatC(nonzero90*100,digits = 1, format = "f"), file=nonzero90_filename)

# Implementations
res <- read.table( #file="~/unsafe_analysis/analysis-data/research-questions/rq03-impls"
  file="/home/nora/work/unsafe-analysis-data/research-questions/servo/research-questions-servo-all/rq03-impls"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("crate", "count"))
res90 <- read.table( #file="~/unsafe_analysis/analysis-data/research-questions-90-percent/rq03-impls"
  file="/home/nora/work/unsafe-analysis-data/research-questions/servo/research-questions-servo-all/rq03-impls"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("crate", "count"))


nonzero_filename <- "~/work/unsafe-analysis-data/paper/rq01_impls_some.txt" 
nonzero90_filename <- "~/work/unsafe-analysis-data/paper/rq01_impls_some90.txt" 
nonzero <- length(res$count[res$count!=0])/nrow(res)
nonzero90 <- length(res90$count[res90$count!=0])/nrow(res90)
write(formatC(nonzero*100,digits = 1, format = "f"), file=nonzero_filename)
write(formatC(nonzero90*100,digits = 1, format = "f"), file=nonzero90_filename)

# Sync and Send

library(DescTools)

res <- read.table( #file="~/unsafe_analysis/analysis-data/research-questions/rq03-impls-names"
  file="/home/nora/work/unsafe-analysis-data/research-questions/servo/research-questions-servo-all/rq03-impls-names"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("crate", "name"))

sync_filename <- "~/work/unsafe-analysis-data/paper/rq01_impls_sync.txt" 
send_filename <- "~/work/unsafe-analysis-data/paper/rq01_impls_send.txt" 
sync_frame <- subset(res, res$name %like any% c("%marker::Sync%"))
sync_no <- nrow(sync_frame)
send_frame <- subset(res, res$name %like any% c("%marker::Send%"))
send_no <- nrow(send_frame)
write(formatC(sync_no/nrow(res)*100,digits = 1, format = "f"),sync_filename)
write(formatC(send_no/nrow(res)*100,digits = 1, format = "f"),send_filename)
