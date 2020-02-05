#!/usr/bin/env Rscript
library(ggplot2)
library(Hmisc)
library(DescTools)

#Traits
res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq01-traits"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("crate", "count"))
res90 <- read.table( file="~/unsafe_analysis/analysis-data/research-questions-90-percent/rq01-traits"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("crate", "count"))


nonzero <- length(res$count[res$count!=0])/nrow(res)
nonzero90 <- length(res90$count[res90$count!=0])/nrow(res90)

print("percentage of crates with unsafe traits")
print(formatC(nonzero*100,digits = 1, format = "f"))

print("percentage of most downloaded crates with unsafe traits")
println(formatC(nonzero*100,digits = 1, format = "f"))

#nonzero_filename <- "~/work/unsafe-analysis-data/paper/rq01_traits_some.txt" 
#nonzero90_filename <- "~/work/unsafe-analysis-data/paper/rq01_traits_some90.txt" 
#write(formatC(nonzero*100,digits = 1, format = "f"), file=nonzero_filename)
#write(formatC(nonzero90*100,digits = 1, format = "f"), file=nonzero90_filename)

# Implementations
res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq01-impls"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("crate", "count"))
res90 <- read.table( file="~/unsafe_analysis/analysis-data/research-questions-90-percent/rq01-impls"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("crate", "count"))

nonzero <- length(res$count[res$count!=0])/nrow(res)
nonzero90 <- length(res90$count[res90$count!=0])/nrow(res90)

print("percentage of crates with unsafe impls")
print(formatC(nonzero*100,digits = 1, format = "f"))

print("percentage of most downloaded crates with unsafe impls")
print(formatC(nonzero*100,digits = 1, format = "f"))

#nonzero_filename <- "~/work/unsafe-analysis-data/paper/rq01_impls_some.txt" 
#nonzero90_filename <- "~/work/unsafe-analysis-data/paper/rq01_impls_some90.txt" 
#write(formatC(nonzero*100,digits = 1, format = "f"), file=nonzero_filename)
#write(formatC(nonzero90*100,digits = 1, format = "f"), file=nonzero90_filename)

# Sync and Send

res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq01-impls-names"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("crate", "name"))

sync_frame <- subset(res, res$name %like any% c("%marker::Sync%"))
sync_no <- nrow(sync_frame)
send_frame <- subset(res, res$name %like any% c("%marker::Send%"))
send_no <- nrow(send_frame)

print("Send")
print(send_no/nrow(res)*100)

print("Sync")
print(sync_no/nrow(res)*100)

#sync_filename <- "~/work/unsafe-analysis-data/paper/rq01_impls_sync.txt" 
#send_filename <- "~/work/unsafe-analysis-data/paper/rq01_impls_send.txt" 
#write(formatC(sync_no/nrow(res)*100,digits = 1, format = "f"),sync_filename)
#write(formatC(send_no/nrow(res)*100,digits = 1, format = "f"),send_filename)
