#!/usr/bin/env Rscript
library(plyr)
library(ggplot2)
library(reshape2)
library(Hmisc)
library(scales)

## do not try to use stat_ecf again

output_dir <- "~/unsafe_analysis/camera-ready/"
res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq01-blocks"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("blocks", "user", "total", "name"))
res90 <- read.table( file="~/unsafe_analysis/analysis-data/research-questions-90-percent/rq01-blocks"
                     , header=FALSE
                     , sep='\t'
                     , comment.char = "#"
                     , col.names=c("blocks", "user", "total", "name"))

blocks <- res$user
blocks90 <- res90$user
summary <- quantile(blocks, c(.90,.95,.995))

ggdata_all <- ddply( melt(data.frame(blocks)),
                       .(variable), transform, ecdf=ecdf(value)(value))
ggdata_90 <- ddply( melt(data.frame(blocks90)),
                      .(variable), transform, ecdf=ecdf(value)(value))
none <- min(ggdata_all$ecdf)
none90 <- min(ggdata_90$ecdf)
min_y <- min( none, none90)
first_y <- ceiling(min_y*10)/10

x_max <- summary["99.5%"]


ggplot() +
    geom_point(data=ggdata_all, aes(x=value, y=ecdf))+
    geom_point(data=ggdata_90, aes(x=value, y=ecdf), color='grey60')+
    xlab("Unsafe Blocks") +
    ylab("Percent of Crates") +
    scale_x_continuous(
      breaks=c(seq(0,x_max-50,50),x_max)
      , limits = c(0,x_max+1)
      , labels = comma
    ) +
    scale_y_continuous(
      limits = c(min_y-0.01,1)
      , breaks = seq(first_y,1,0.05) #c(none, none90, seq(first_y,1,0.05))
      ,labels = percent
    ) +
    theme(
      text = element_text(size=25),
      axis.text.x=element_text(angle=90, hjust=1),
      panel.background = element_rect(fill = "white",
                                      colour = "white",
                                      size = 0.5, linetype = "solid"),
      panel.grid.major = element_line(size = 0.5, linetype = 'solid',
                                      colour = "grey"), 
      panel.grid.minor = element_line(size = 0.25, linetype = 'solid',
                                      colour = "white")
    ) 

ggsave(file.path(output_dir,"rq01_blocks_cdf.eps"), plot = last_plot(), device = "eps")
