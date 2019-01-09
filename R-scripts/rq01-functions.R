#!/usr/bin/env Rscript
library(ggplot2)
library(reshape2)
library(plyr)
library(Hmisc)
library(scales)

res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions-servo-all/rq02"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("functions", "name"))
res90 <- read.table( file="~/unsafe_analysis/analysis-data/research-questions-servo-only/rq02"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("functions", "name"))

cdf_filename <- "~/work/unsafe-analysis-data/paper/rq01_servo_functions_cdf.eps"
base_filename <- "~/work/unsafe-analysis-data/paper/rq01_servo_functions_"

summary <- quantile(res$functions, c(.90,.95,.99))
write(summary[1],file=paste0(base_filename,"90",".txt"))
write(summary[2],file=paste0(base_filename,"95",".txt"))

#graph 
ggdata <- ddply(melt(data.frame(res$functions)), 
                .(variable), transform, ecdf=ecdf(value)(value))
ggdata90 <- ddply(melt(data.frame(res90$functions)), 
                .(variable), transform, ecdf=ecdf(value)(value))

none <- min(ggdata$ecdf)
none90 <- min(ggdata90$ecdf)
min_y <- min(none,none90)
first_y <- ceiling(min_y*10)/10

x_max <- summary["95%"]

ggplot() +
  geom_point(data=ggdata, aes(x=value, y=ecdf)) +
  geom_point(data=ggdata90, aes(x=value, y=ecdf), color='grey60')+
  xlab("Declared Unsafe Functions") +
  ylab("Percent of Crates") +
  labs(title="Cumulative Distribution of Declared Unsafe Functions") +
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
    limits = c(min_y-0.01,1)
    , breaks = c(none90, seq(first_y,1,0.05))
    ,labels = percent
  )
ggsave(cdf_filename, plot = last_plot(), device = "eps")


write(summary["90%"],file=paste0(base_filename,"90",".txt"))
p95 <- paste0(base_filename,"95",".txt")
write(summary["95%"],file=p95)


#save number of crates with no unsafe functions
options(digits = 4)
write(none*100,paste0(base_filename,"none",".txt"))
write(none90*100,paste0(base_filename,"none90",".txt"))
