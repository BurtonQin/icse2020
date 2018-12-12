#!/usr/bin/env Rscript
library(ggplot2)
library(reshape2)
library(plyr)
library(Hmisc)
library(scales)

cdf_filename <- "~/work/unsafe_study/paper/rq02_all_cdf.eps"

# TODO change this to precise
res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq07_opt"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("crate", "no"))

# TODO change this to precise
res1 <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq07_pes"
                    , header=FALSE
                    , sep='\t'
                    , comment.char = "#"
                    , col.names=c("crate", "no"))

#graph
ggdata1 <- data.frame(res$no)
ggdata1 <- melt(ggdata1)
ggdata1 <- ddply(ggdata1, .(variable), transform, ecdf=ecdf(value)(value))

ggdata2 <- data.frame(res1$no)
ggdata2 <- melt(ggdata2)
ggdata2 <- ddply(ggdata2, .(variable), transform, ecdf=ecdf(value)(value))

y0 <- length( res$no[res$no==0] ) / length(res$no)
y1 <- length( res1$no[res1$no==0] ) / length(res1$no)

min_y <- max(
  y0,y1
)
first_y <- ceiling(min_y*10)/10
summary <- quantile(res1$no, c(.90,.95,.99))
x_max <- summary["95%"]

ggplot() +
  geom_line(data=ggdata1, aes(x=value, y=ecdf), shape=20, colour="grey60")+
  geom_line(data=ggdata2, aes(x=value, y=ecdf), shape=20)+
  xlab("Possibly Unsafe Functions") +
  ylab("Percent of Crates") +
  labs(title="Cumulative Distribution of Possibly Unsafe Functions") +
  scale_x_continuous(
    breaks=c(seq(0,x_max-10,10),x_max)
    , limits = c(0,x_max+1)
    , labels = comma
  ) +
  theme(axis.text.x=element_text(angle=90, hjust=1),
        panel.background = element_rect(fill = "white",
                                        colour = "white",
                                        size = 0.5, linetype = "solid"),
        panel.grid.major = element_line(size = 0.5, linetype = 'solid',
                                        colour = "grey"), 
        panel.grid.minor = element_line(size = 0.25, linetype = 'solid',
                                        colour = "white")
        ) +
  scale_y_continuous(
    limits = c(y1-0.01,1)
    , breaks = c(y1,y0, seq(first_y,1,0.05))
    ,labels = percent
  )
ggsave(cdf_filename, plot = last_plot(), device = "eps")

options(digits = 4)
write(y1*100,file="~/work/unsafe_study/paper/safe_lower.txt")
write(y0*100,file="~/work/unsafe_study/paper/safe_upper.txt")

opt_max <- max(res$no)
write(opt_max,file="~/work/unsafe_study/paper/opt_max.txt")
cons_max <- max(res1$no)
write(cons_max,file="~/work/unsafe_study/paper/cons_max.txt")