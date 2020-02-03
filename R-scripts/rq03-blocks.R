#!/usr/bin/env Rscript
library(ggplot2)
library(Hmisc)
library(plyr)
library(scales)


process_rq <- function(in_file) {
  res <- read.table( file=in_file
                     , header=FALSE
                     , sep=','
                     , comment.char = "#"
                     , col.names=c("blockid", "source","user","crate"))
  res <- res[which(res$user=="true"),]
  res_aggregate <- count(res, c("source"))
  res_aggregate$freq <- res_aggregate$freq / nrow(res) 
  res_aggregate$type <- "All"

  exclude <- (subset(res_aggregate, freq < 0.001))[,"source"]
  res_aggregate <- subset( res_aggregate, !is.element(source,exclude) )

  for (i in 1:nrow(res_aggregate)) {
    print( total_frame$source[i])
    println( percent(total_frame$freq[i]) )
  }
}

process_rq("~/unsafe_analysis/analysis-data/research-questions/rq03-blocks")
process_rq("~/unsafe_analysis/analysis-data/research-questions-90-percent/rq03-blocks")
process_rq("~/unsafe_analysis/analysis-data/research-questions-servo/rq03-blocks")


