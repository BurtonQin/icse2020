#!/usr/bin/env Rscript
library(ggplot2)
library(Hmisc)
library(DescTools)

#Traits

res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq03-traits"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("crate", "count"))


nonzero_filename <- "~/work/unsafe_study/paper/rq03_traits_some.txt" 
zero_filename <- "~/work/unsafe_study/paper/rq03_traits_none.txt" 
all_filename <- "~/work/unsafe_study/paper/rq03_traits_n.txt" 

zero_frame <- subset(res,res$count==0)
some_frame <- subset(res,res$count!=0)

nonzero <- nrow(some_frame)
zero <- nrow(zero_frame)
all <- nrow(res)

write(formatC(nonzero/all*100,digits = 1, format = "f"), file=nonzero_filename)
write(formatC(zero/all*100,digits = 1, format = "f"), file=zero_filename)
write(all, file=all_filename)

# Implementations

res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq03-impls"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("crate", "count"))


nonzero_filename <- "~/work/unsafe_study/paper/rq03_impls_some.txt" 
zero_filename <- "~/work/unsafe_study/paper/rq03_impls_none.txt" 
all_filename <- "~/work/unsafe_study/paper/rq03_impls_n.txt" 

zero_frame <- subset(res,res$count==0)
some_frame <- subset(res,res$count!=0)

nonzero <- nrow(some_frame)
zero <- nrow(zero_frame)
all <- nrow(res)

write(formatC(nonzero/all*100,digits = 1, format = "f"), file=nonzero_filename)
write(formatC(zero/all*100,digits = 1, format = "f"), file=zero_filename)
write(all, file=all_filename)

# Sync and Send
res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq03-impls-names"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("crate", "name"))

sync_filename <- "~/work/unsafe_study/paper/rq03_impls_sync.txt" 
send_filename <- "~/work/unsafe_study/paper/rq03_impls_send.txt" 
sync_frame <- subset(res, res$name %like any% c("%marker::Sync%"))
sync_no <- nrow(sync_frame)
send_frame <- subset(res, res$name %like any% c("%marker::Send%"))
send_no <- nrow(send_frame)
write(formatC(sync_no/all*100,digits = 1, format = "f"),sync_filename)
write(formatC(send_no/all*100,digits = 1, format = "f"),send_filename)
