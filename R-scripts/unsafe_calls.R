#!/usr/bin/env Rscript
library(ggplot2)
library(Hmisc)
library(plyr)
library(scales)
library(xtable)

#p <- pipe(paste0('sed \'s/"\'"/"`"/g\' "', "~/unsafe_analysis/analysis-data/research-questions/rq06", '"'))

res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq06"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , quote="\\"
                   , col.names=c("abi", "crate", "full_path", "name", "user"))

rust <- subset(res, abi == "Rust")
core <- subset(res, abi=="Rust" & crate=="core")

crates <- rust[c(2)]

crates_freq <- count(crates)
crates_freq$freq <- 100 * crates_freq$freq / nrow(crates) 

top_crates <- subset(crates_freq, freq > 1.0)

xtable(top_crates, type = "latex")

functions <- core[c(4)]

print(nrow(core)/nrow(rust))
