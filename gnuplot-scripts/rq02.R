#!/usr/bin/env Rscript
library(ggplot2)
library(reshape2)
library(plyr)
library(Hmisc)
library(scales)

res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq02"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("functions", "name"))

table_filename <- "~/work/unsafe_study/paper/rq02_table.txt"
cdf_filename <- "~/work/unsafe_study/paper/rq02_cdf.eps"
base_filename <- "~/work/unsafe_study/paper/rq02_"
outliers_filename <- "~/work/unsafe_study/paper/rq02_outliers.txt" 

#table
summary <- quantile(res$functions, c(.90,.95,.99))
fn <- paste0(base_filename,"n",".txt")
write(nrow(res),file=fn)
p90 <- paste0(base_filename,"90",".txt")
write(summary[1],file=p90)
p95 <- paste0(base_filename,"95",".txt")
write(summary[2],file=p95)


#graph 
ggdata <- data.frame(res$functions)
ggdata <- melt(ggdata)
ggdata <- ddply(ggdata, .(variable), transform, ecdf=ecdf(value)(value))

min_y <- length( res$functions[res$functions==0] ) / length(res$functions)
first_y <- ceiling(min_y*10)/10

x_max <- summary[3]
outliers <- subset(res,functions>summary[3])

ggplot(ggdata, aes(x=value, y=ecdf)) +
  geom_point()+
  xlab("Unsafe Functions") +
  ylab("Percent of Crates") +
  labs(title="Cumulative Distribution of Unsafe Functions") +
  scale_x_continuous(
    breaks=c(seq(0,x_max-10,10),x_max)
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
write(min(outliers$functions), file=outliers_filename)
write(" and ", file=outliers_filename,append=TRUE)
write(max(outliers$functions), file=outliers_filename,append=TRUE)
write(" (", file=outliers_filename,append=TRUE)
write(nrow(outliers), file=outliers_filename,append=TRUE)
write(" values).", file=outliers_filename,append=TRUE)


#save number of crates with at least one unsafe
options(digits = 4)
nonzero <- 100 - min_y*100
write(nonzero,file=nonzero_filename)
