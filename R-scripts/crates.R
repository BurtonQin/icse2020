#!/usr/bin/env Rscript
library(ggplot2)
library(reshape2)
library(plyr)
library(Hmisc)
library(scales)

res <- read.table( file="~/work/unsafe_study/select-crates/crates.io-sorted"
                   , header=FALSE
                   , sep=' '
                   , comment.char = "#"
                   , col.names=c("crate", "downloads"))

ggplot(data = res, aes(x = 1:nrow(res), y = downloads)) +
  geom_point() +
  labs(x = "Index")

s <- sum(res$downloads)
p <- s*0.9

pos <- 0

while (sum(res$downloads[0:pos]) < p) {
  pos = pos + 1
}

pp <- sum(res$downloads[0:pos])/s * 100

c <- res$downloads[512]