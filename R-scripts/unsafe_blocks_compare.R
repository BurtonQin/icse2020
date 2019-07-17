#!/usr/bin/env Rscript
library(plyr)
library(ggplot2)
library(reshape2)
library(Hmisc)
library(scales)

## do not try to use stat_ecf again

output_dir <- "~/work/unsafe-analysis-data/paper/"
res2018 <- read.table( file="~/unsafe_analysis/analysis-data/research-questions-90-percent/rq01"
                     , header=FALSE
                     , sep='\t'
                     , comment.char = "#"
                     , col.names=c("blocks", "user", "total", "name"))

res2019 <- read.table( file="~/unsafe_analysis/analysis-data-2019/research-questions-90-percent/rq01"
                       , header=FALSE
                       , sep='\t'
                       , comment.char = "#"
                       , col.names=c("blocks", "user", "total", "name"))

increase <- 0
same <- 0
decrease <- 0

not_found <- 0
multiples <- 0

for (row in 1:nrow(res2018)) {
  # check if crate name is in res2019
  crate_name <- as.character(res2018[row,4])
  blocks2018 <- as.integer((res2018[row,2]))
  #select row from 2019
  new_data <- subset(res2019, name == crate_name)
  if (nrow(new_data) == 0) {
    not_found <- not_found + 1
  } else {
    if (nrow(new_data)>1) {
      multiples <- multiples + 1
    } else {
      blocks2019 <- as.integer(new_data[1,2])
      if (blocks2019 == blocks2018) {
        same <- same + 1
      } else {
        if (blocks2018<blocks2019) {
          increase <- increase + 1
        } else {
          decrease <- decrease + 1
        }
      }
    }
  }
}

print(same) 
print(increase)
print(decrease)
print(multiples)
print(not_found)