#!/usr/bin/env Rscript
library(ggplot2)
library(Hmisc)

res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq01"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("blocks", "name"))

cdf_filename <- "~/work/unsafe_study/paper/rq01_cdf.eps"
nonzero_filename <- "~/work/unsafe_study/paper/rq01_some.txt" 
base_filename <- "~/work/unsafe_study/paper/rq01_"

#table
summary <- quantile(res$blocks, c(.90,.95))
fn <- paste0(base_filename,"n",".txt")
write(nrow(res),file=fn)
p90 <- paste0(base_filename,"90",".txt")
write(summary[1],file=p90)
p95 <- paste0(base_filename,"95",".txt")
write(summary[2],file=p95)

#graph
top5_x <- res$blocks[order(res$blocks,decreasing = TRUE )[1:5]]
min_y <- length( res$blocks[res$blocks==0] ) / length(res$blocks)
first_y <- ceiling(min_y*10)/10

blocks.q <- quantile(res$blocks)

ggplot(NULL, aes(x=res$blocks)) +
  geom_step(stat="ecdf") +
#  geom_vline(aes(xintercept=blocks.q[2:4]), linetype="dashed") +
  xlab("Unsafe Blocks") + 
  ylab("Percent of Crates") +
  labs(title="Cumulative Distribution of Unsafe Block") +
  scale_x_continuous(
    breaks=c(seq(0,top5_x[5],100),top5_x[1])
    , labels = comma
  ) +
  theme(axis.text.x=element_text(angle=90, hjust=1)) +
  scale_y_continuous(
    limits = c(min_y-0.01,1)
    , breaks = c(min_y, seq(first_y,1,0.05))
    ,labels = percent
  )

ggsave(cdf_filename, plot = last_plot(), device = "eps")

#save number of crates with at least one unsafe
options(digits = 4)
nonzero <- 100 - min_y*100
write(nonzero,file=nonzero_filename)
