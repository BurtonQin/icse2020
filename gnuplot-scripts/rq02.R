#!/usr/bin/env Rscript
library(ggplot2)
library(Hmisc)

res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq02"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("blocks", "name"))

table_filename <- "~/work/unsafe_study/paper/rq02_table.txt"
cdf_filename <- "~/work/unsafe_study/paper/rq02_cdf.eps"
ft <- describe(res$blocks)
latex(ft,file=table_filename)
top5_x <- res$blocks[order(res$blocks,decreasing = TRUE )[1:5]]
min_y <- nrow(subset(res,blocks==0)) / nrow(res)
first_y <- ceiling(min_y*100)/100 + 0.1
ggplot(res, aes(blocks)) + 
  stat_ecdf(geom = "point") +
  scale_x_continuous(
    breaks= c(seq(0,top5_x[5],100),rev(top5_x))
    , limits=c(0,top5_x[1]+1)
    ,labels = comma
  ) +
  scale_y_continuous(
    limits=c(min_y,1)
    ,breaks = c(min_y, seq(first_y,1,0.1))
    ,labels = percent
  ) +
  theme(axis.text.x=element_text(angle=90, hjust=1))+
  xlab("Cumulative Distribution") + 
  ylab("Percentage of Crates") +
  labs(title="Cumulative Distribution of Unsafe Functions") 
ggsave(cdf_filename, plot = last_plot(), device = "eps")