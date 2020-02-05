#!/usr/bin/env Rscript
library(ggplot2)
library(Hmisc)
library(plyr)
library('scales')

res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq03-func"
                   , header=FALSE
                   , sep=','
                   , comment.char = "#"
                   , col.names=c("source", "user","crate"))

res90 <- read.table( file="~/unsafe_analysis/analysis-data/research-questions-90-percent/rq03-func"
                     , header=FALSE
                     , sep=','
                     , comment.char = "#"
                     , col.names=c("source", "user","crate"))

user_only <- res[which(res$user=="true"),]
user_only <- user_only[which(user_only$source != 'From Trait'),]
user_only <- user_only[which(user_only$source != 'Raw Pointer Argument'),]
user_only90 <- res90[which(res90$user=="true"),]
user_only90 <- user_only90[which(user_only90$source != 'From Trait'),]
user_only90 <- user_only90[which(user_only90$source != 'Raw Pointer Argument'),]

user_aggregate <- count(user_only, c("source"))
user_aggregate$freq <- user_aggregate$freq / nrow(user_only) 
user_aggregate$type <- "All"

user90_aggregate <- count(user_only90, c("source"))
user90_aggregate$freq <- user90_aggregate$freq / nrow(user_only90) 
user90_aggregate$type <- "Most Downloaded"

exclude <- (subset(user_aggregate, freq < 0.001))[,"source"]
user_aggregate <- subset( user_aggregate, !is.element(source,exclude) )
user90_aggregate <- subset( user90_aggregate, !is.element(source,exclude) )

total_frame <- rbind(user_aggregate,user90_aggregate)

for (i in 1:nrow(total_frame)) {
  print(total_frame$type[i])
  print(total_frame$source[i])
  print(percent(total_frame$freq[i]))
}
