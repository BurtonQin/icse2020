#!/usr/bin/env Rscript
library(ggplot2)
library(Hmisc)

res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq02"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("functions", "name"))

table_filename <- "~/work/unsafe_study/paper/rq02_table.txt"
cdf_filename <- "~/work/unsafe_study/paper/rq02_cdf.eps"
nonzero_filename <- "~/work/unsafe_study/paper/rq02_some.txt" 

latex(describe(res$functions),file=table_filename)

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
    breaks=c(seq(0,top5_x[1],500))
    , limits = c(0,top5_x[1])
    , labels = comma
  ) +
  theme(axis.text.x=element_text(angle=90, hjust=1)) +
  scale_y_continuous(
    limits = c(min_y-0.01,1)
    , breaks = c(min_y, seq(first_y,1,0.01))
    ,labels = percent
  )
ggsave(cdf_filename, plot = last_plot(), device = "eps")

#save number of crates with at least one unsafe
options(digits = 4)
nonzero <- 100 - min_y*100
write(nonzero,file=nonzero_filename)
