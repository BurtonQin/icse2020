#!/usr/bin/env Rscript
library(ggplot2)
library(reshape2)
library(plyr)
library(Hmisc)
library(scales)

res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq01"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("blocks", "name"))

cdf_filename <- "~/work/unsafe_study/paper/rq01_cdf.eps"
nonzero_filename <- "~/work/unsafe_study/paper/rq01_some.txt" 
base_filename <- "~/work/unsafe_study/paper/rq01_"
outliers_filename <- "~/work/unsafe_study/paper/rq01_outliers.txt" 

#table
summary <- quantile(res$blocks, c(.90,.95,.995))
fn <- paste0(base_filename,"n",".txt")
write(nrow(res),file=fn)
p90 <- paste0(base_filename,"90",".txt")
write(summary[1],file=p90)
p95 <- paste0(base_filename,"95",".txt")
write(summary[2],file=p95)

#graph
ggdata <- data.frame(res$blocks)
ggdata <- melt(ggdata)
ggdata <- ddply(ggdata, .(variable), transform, ecdf=ecdf(value)(value))

min_y <- length( res$blocks[res$blocks==0] ) / length(res$blocks)
first_y <- ceiling(min_y*10)/10
x_max <- summary[3]
outliers <- subset(res,blocks>summary[3])

ggplot(ggdata, aes(x=value, y=ecdf)) +
  geom_point()+
  xlab("Unsafe Blocks") +
  ylab("Percent of Crates") +
  labs(title="Cumulative Distribution of Unsafe Blocks") +
  scale_x_continuous(
    breaks=c(seq(0,x_max,50),x_max)
    , limits = c(0,x_max+1)
    , labels = comma
  ) +
  theme(axis.text.x=element_text(angle=90, hjust=1)) +
  scale_y_continuous(
    limits = c(min_y-0.01,1)
    , breaks = c(min_y, seq(first_y,1,0.05))
    ,labels = percent
  )
ggsave(cdf_filename, plot = last_plot(), device = "eps")

#outliers
write(min(outliers$blocks), file=outliers_filename)
write(" and ", file=outliers_filename,append=TRUE)
write(max(outliers$blocks), file=outliers_filename,append=TRUE)
write(" (", file=outliers_filename,append=TRUE)
write(nrow(outliers), file=outliers_filename,append=TRUE)
write(" values).", file=outliers_filename,append=TRUE)


#save number of crates with at least one unsafe
options(digits = 4)
nonzero <- 100 - min_y*100
write(nonzero,file=nonzero_filename)
