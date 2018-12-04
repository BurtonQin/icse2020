#!/usr/bin/env Rscript
library(ggplot2)
library(reshape2)
library(plyr)
library(Hmisc)
library(scales)

plot <- function (all,b90,filename) {
  summary <- quantile(all, c(.90,.95,.995))
  ggdata_all <- ddply( melt(data.frame(all)), 
                       .(variable), transform, ecdf=ecdf(value)(value))
  ggdata_90 <- ddply( melt(data.frame(b90)), 
                      .(variable), transform, ecdf=ecdf(value)(value))
  
  min_y <- min( length( all[all==0] ) / length(all), length( b90[b90==0] ) / length(b90))
  first_y <- ceiling(min_y*10)/10
  x_max <- summary[3]
  #outliers <- subset(res,blocks>summary[3])
  
  ggplot() +
    geom_point(data=ggdata_all, aes(x=value, y=ecdf))+
    geom_point(data=ggdata_90, aes(x=value, y=ecdf), color='grey')+
    xlab("Unsafe Blocks") +
    ylab("Percent of Crates") +
    labs(title="Cumulative Distribution of Unsafe Blocks") +
    scale_x_continuous(
      breaks=c(seq(0,x_max-50,50),x_max)
      , limits = c(0,x_max+1)
      , labels = comma
    ) +
    theme(axis.text.x=element_text(angle=90, hjust=1)) +
    scale_y_continuous(
      limits = c(min_y-0.01,1)
      , breaks = c(min_y, seq(first_y,1,0.05))
      ,labels = percent
    )
  
  ggsave(file.path(output_dir,filename), plot = last_plot(), device = "eps")
}

output_dir <- "~/work/unsafe-analysis-data/paper/"
res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq01"
                     , header=FALSE
                     , sep='\t'
                     , comment.char = "#"
                     , col.names=c("blocks", "user", "total", "name"))
res90 <- read.table( file="~/unsafe_analysis/analysis-data/research-questions-90-percent/rq01"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("blocks", "user", "total", "name"))
plot(res$blocks,res90$blocks,"rq01_cdf.eps")
plot(res$user,res90$user,"rq01_user_only_cdf.eps")
  
summary <- quantile(res$blocks, c(.90,.95,.995))
base_filename <- paste0(output_dir, "rq01_")
fn <- paste0(base_filename,"n",".txt")
write(nrow(res),file=fn)
p90 <- paste0(base_filename,"90",".txt")
write(summary[1],file=p90)
p95 <- paste0(base_filename,"95",".txt")
write(summary[2],file=p95)

