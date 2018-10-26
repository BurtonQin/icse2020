#!/usr/bin/env Rscript
library(ggplot2)
library(reshape2)
library(plyr)
library(Hmisc)
library(scales)

cdf_filename <- "~/work/unsafe_study/paper/rq07_cdf.eps"

res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq07_coarse_opt"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("crate", "def_path", "notsafe", "name"))
coarse_opt <- aggregate(res$notsafe, by=list(res$crate), FUN=sum)

res1 <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq07_coarse_pes"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("crate", "def_path", "notsafe", "name"))
coarse_pes <- aggregate(res1$notsafe, by=list(crate=res1$crate), FUN=sum)


#graph
ggdata1 <- data.frame(coarse_opt$x)
ggdata1 <- melt(ggdata1)
ggdata1 <- ddply(ggdata1, .(variable), transform, ecdf=ecdf(value)(value))

ggdata2 <- data.frame(coarse_pes$x)
ggdata2 <- melt(ggdata2)
ggdata2 <- ddply(ggdata2, .(variable), transform, ecdf=ecdf(value)(value))

y0 <- length( coarse_opt$x[coarse_opt$x==0] ) / length(coarse_opt$x)
y1 <- length( coarse_pes$x[coarse_pes$x==0] ) / length(coarse_pes$x)

min_y <- max(
    y0,y1
)
first_y <- ceiling(min_y*10)/10
summary <- quantile(coarse_pes$x, c(.90,.95,.995))
x_max <- summary[2]

ggplot() +
  geom_point(data=ggdata1, aes(x=value, y=ecdf), shape=20, colour="grey")+
  geom_point(data=ggdata2, aes(x=value, y=ecdf), shape=20)+
  xlab("Not Safe Functions") +
  ylab("Percent of Crates") +
  labs(title="Cumulative Distribution of Not Safe Functions") +
  scale_x_continuous(
    breaks=c(seq(0,x_max-100,100),x_max)
    , limits = c(0,x_max+1)
    , labels = comma
  ) +
  theme(axis.text.x=element_text(angle=90, hjust=1)) +
  scale_y_continuous(
    limits = c(y1-0.01,1)
    , breaks = c(y1,y0, seq(first_y,1,0.05))
    ,labels = percent
  )
ggsave(cdf_filename, plot = last_plot(), device = "eps")

