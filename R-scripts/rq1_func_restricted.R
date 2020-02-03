#!/usr/bin/env Rscript
library(ggplot2)
library(reshape2)
library(plyr)
library(Hmisc)
library(scales)

res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq01-restricted-func"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("functions", "name"))
res90 <- read.table( file="~/unsafe_analysis/analysis-data/research-questions-90-percent/rq01-restricted-func"
                     , header=FALSE
                     , sep='\t'
                     , comment.char = "#"
                     , col.names=c("functions", "name"))

cdf_filename <- "~/work/unsafe-analysis-data/paper/rq01_restricted_functions_cdf.eps"

#graph 
ggdata <- ddply(melt(data.frame(res$functions)), 
                .(variable), transform, ecdf=ecdf(value)(value))
ggdata90 <- ddply(melt(data.frame(res90$functions)), 
                  .(variable), transform, ecdf=ecdf(value)(value))

none <- min(ggdata$ecdf)
none90 <- min(ggdata90$ecdf)
min_y <- min(none,none90)
first_y <- ceiling(min_y*10)/10

x_max <- summary["99%"]

ggplot() +
  geom_point(data=ggdata, aes(x=value, y=ecdf)) +
  geom_point(data=ggdata90, aes(x=value, y=ecdf), color='grey60')+
  xlab("Declared Unsafe Functions with Unsafe Operations") +
  ylab("Percent of Crates") +
  scale_x_continuous(
    breaks=c(seq(0,x_max-10,10),x_max)
    , limits = c(0,x_max+1)
    , labels = comma
  ) +
  theme(axis.text.x=element_text(angle=90, hjust=1),
        text = element_text(size=25),
        panel.background = element_rect(fill = "white",
                                        colour = "white",
                                        size = 0.5, linetype = "solid"),
        panel.grid.major = element_line(size = 0.5, linetype = 'solid',
                                        colour = "grey"), 
        panel.grid.minor = element_line(size = 0.25, linetype = 'solid',
                                        colour = "white")
  ) +
  scale_y_continuous(
    limits = c(min_y-0.01,1)
    , breaks = c(none90, seq(first_y,1,0.05))
    ,labels = percent
  )
ggsave(cdf_filename, plot = last_plot(), device = "eps")

