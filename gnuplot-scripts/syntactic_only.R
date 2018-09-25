#!/usr/bin/env Rscript
library(ggplot2)
library(Hmisc)

table_filename <- "~/work/unsafe_study/paper/syntactic_func_table.txt"
dot_filename <- "~/work/unsafe_study/paper/syntactic_func_scatter_plot.eps"
hist_filename <- "~/work/unsafe_study/paper/syntactic_func_hist.eps"

print (hist_filename)

res <- read.table( file="/home/ans5k/unsafe_analysis/analysis-data/syntactic_only_results.txt"
                   , header=FALSE
                   , sep=','
                   , comment.char = "#"
                   , col.names=c("functions", "traits", "blocks", "name"))


ft <- describe(res$functions)

latex(ft,file="~/work/unsafe_study/paper/syntactic_func_table.txt")

ggplot(res,
       aes(functions, ..count.. ) ) + 
  geom_point(stat = "count", size = 1) + 
  xlab("Number of Unsafe Functions in Crate") + 
  ylab("Number of Crates") +
  labs(title="Unsafe Functions") +
  scale_y_log10(breaks = scales::trans_breaks("log10", function(x) 10^x),
                labels = scales::trans_format("log10", scales::math_format(10^.x)))

ggsave(dot_filename, plot = last_plot(), device = "eps")


ggplot(data=res, aes(res$functions)) + 
  geom_histogram()
ggsave(hist_filename, plot = last_plot(), device = "eps")
