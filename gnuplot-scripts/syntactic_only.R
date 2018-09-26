#!/usr/bin/env Rscript
library(ggplot2)
library(Hmisc)
library(scales)

res <- read.table( file="~/unsafe_analysis/analysis-data/syntactic_only_results.txt"
                   , header=FALSE
                   , sep=','
                   , comment.char = "#"
                   , col.names=c("functions", "traits", "blocks", "name"))

table_filename <- "~/work/unsafe_study/paper/syntactic_func_table.txt"
cdf_filename <- "~/work/unsafe_study/paper/syntactic_func_cdf.eps"
ft <- describe(res$functions)
latex(ft,file="~/work/unsafe_study/paper/syntactic_func_table.txt")
top5_x <- res$functions[order(res$functions,decreasing = TRUE )[1:5]]
min_y <- nrow(subset(res,functions==0)) / nrow(res)
first_y <- ceiling(min_y*100)/100 + 0.01
ggplot(res, aes(functions)) + 
  stat_ecdf(geom = "step") +
  scale_x_continuous(
      breaks=c(seq(0,top5_x[4],100),top5_x[1:4])
      , labels = comma
    ) +
  scale_y_continuous(
      limits=c(min_y,1)
      ,breaks = c(min_y, seq(first_y,1,0.01))
      ,labels = percent
    ) +
  theme(axis.text.x=element_text(angle=90, hjust=1))+
  xlab("Unsafe Functions") + 
  ylab("Percent of Crates") +
  labs(title="Cumulative Distribution of Unsafe Functions") 
ggsave(cdf_filename, plot = last_plot(), device = "eps")


table_filename <- "~/work/unsafe_study/paper/syntactic_block_table.txt"
cdf_filename <- "~/work/unsafe_study/paper/syntactic_block_cdf.eps"
ft <- describe(res$blocks)
latex(ft,file="~/work/unsafe_study/paper/syntactic_block_table.txt")
top5_x <- res$blocks[order(res$blocks,decreasing = TRUE )[1:5]]
min_y <- nrow(subset(res,blocks==0)) / nrow(res)
first_y <- ceiling(min_y*100)/100 + 0.01
ggplot(res, aes(blocks)) + 
  stat_ecdf(geom = "step") +
  scale_x_continuous(
      breaks=c(seq(0,top5_x[4],100),top5_x[1:4])
      , labels = comma
    ) +
  scale_y_continuous(
      limits=c(min_y,1)
      ,breaks = c(min_y, seq(first_y,1,0.01))
      ,labels = percent
    ) +
  theme(axis.text.x=element_text(angle=90, hjust=1))+
  xlab("Number of Unsafe Blocks") + 
  ylab("Percent of Crates") +
  labs(title="Cumulative Distribution of Unsafe Blocks") 
ggsave(cdf_filename, plot = last_plot(), device = "eps")
