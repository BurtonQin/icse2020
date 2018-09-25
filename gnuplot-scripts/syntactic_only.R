#!/usr/bin/env Rscript
library(ggplot2)
library(Hmisc)

table_filename <- "~/work/unsafe_study/paper/syntactic_func_table.txt"
cdf_filename <- "~/work/unsafe_study/paper/syntactic_func_cdf.eps"

res <- read.table( file="/home/nora/unsafe_analysis/analysis-data/syntactic_only_results.txt"
                   , header=FALSE
                   , sep=','
                   , comment.char = "#"
                   , col.names=c("functions", "traits", "blocks", "name"))


ft <- describe(res$functions)
latex(ft,file="~/work/unsafe_study/paper/syntactic_func_table.txt")

ggplot(res, aes(functions)) + 
  stat_ecdf(geom = "step") +
  xlab("Cumulative Distribution") + 
  ylab("Number of Crates") +
  labs(title="Cumulative Distribution of Unsafe Functions") +

ggsave(cdf_filename, plot = last_plot(), device = "eps")

