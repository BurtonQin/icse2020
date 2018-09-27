#!/usr/bin/env Rscript
library(ggplot2)
library(Hmisc)
library(scales)

table_filename <- "~/work/unsafe_study/paper/syntactic_func_table.txt"
cdf_filename <- "~/work/unsafe_study/paper/syntactic_func_cdf.eps"

res <- read.table( file="~/unsafe_analysis/analysis-data/syntactic_only_results.txt"
                   , header=FALSE
                   , sep=','
                   , comment.char = "#"
                   , col.names=c("functions", "traits", "blocks", "name"))

#summary table for unsafe functions
latex(describe(res$functions),file=table_filename)

#ecdf for unsafe functions
top5_x <- res$functions[order(res$functions,decreasing = TRUE )[1:5]]
min_y <- length( res$functions[res$functions==0] ) / length(res$functions)
first_y <- ceiling(min_y*10)/10
functions.q <- mean(res$functions)
ggplot(NULL, aes(x=res$functions)) +
  geom_step(stat="ecdf") +
  geom_vline(aes(xintercept=functions.q), linetype="dashed") +
  xlab("Unsafe Functions") + 
  ylab("Percent of Crates") +
  labs(title="Cumulative Distribution of Unsafe Functions") +
  scale_x_continuous(
    breaks=c(seq(0,top5_x[5],100))
    , limits = c(0,top5_x[5])
    , labels = comma
  ) +
  theme(axis.text.x=element_text(angle=90, hjust=1)) +
  scale_y_continuous(
    limits = c(min_y-0.01,1)
    , breaks = c(min_y, seq(first_y,1,0.01))
    ,labels = percent
  )

#blocks files
table_filename <- "~/work/unsafe_study/paper/syntactic_block_table.txt"
cdf_filename <- "~/work/unsafe_study/paper/syntactic_block_cdf.eps"

#summary table for blocks 
latex(describe(res$blocks),file=table_filename)

#ecdf for blocks 
top5_x <- res$blocks[order(res$blocks,decreasing = TRUE )[1:5]]
min_y <- length( res$blocks[res$blocks==0] ) / length(res$blocks)
first_y <- ceiling(min_y*10)/10

blocks.q <- mean(res$blocks)

ggplot(NULL, aes(x=res$blocks)) +
  geom_step(stat="ecdf") +
  geom_vline(aes(xintercept=blocks.q), linetype="dashed") +
  xlab("Unsafe Blocks") + 
  ylab("Percent of Crates") +
  labs(title="Cumulative Distribution of Unsafe Block") +
  scale_x_continuous(
    breaks=c(seq(0,top5_x[4],100),top5_x[1:4])
    , labels = comma
  ) +
  theme(axis.text.x=element_text(angle=90, hjust=1)) +
  scale_y_continuous(
    limits = c(min_y-0.01,1)
    , breaks = c(min_y, seq(first_y,1,0.05))
    ,labels = percent
  )
ggsave(cdf_filename, plot = last_plot(), device = "eps")
