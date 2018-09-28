#!/usr/bin/env Rscript
library(ggplot2)
library(Hmisc)
library(plyr)

res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq04"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("fn_call", "deref_ptr","asm","static_access","borrow_packed",
                                 "assign_union", "access_union", "extern_static", 
                                 "crate", "block_id"
                                 ))

all_filename <- "~/work/unsafe_study/paper/rq04_all.eps"
calls_filename <- "~/work/unsafe_study/paper/rq04_calls.eps"

labels <- c("Unsafe Function Call", "Derefence Raw Pointer",
            "Static Variable Use", "Assembly",
            "Borrow Packed", "Assign to Union",
            "Access to Union", "Use of Extern Static Variable") #should improve the names 
values <- c( sum(res$fn_call), sum(res$deref_ptr), 
             sum(res$static_access), sum(res$asm),
             sum(res$borrow_packed), sum(res$assign_union),
             sum(res$access_union), sum(res$extern_static))
n <- sum(values)
values <- values/n

all_frame <- data.frame(names = labels, data = values)
all_frame$names <- factor(all_frame$names, levels = all_frame$names[order(all_frame$data)])

#ggplot(all_frame, aes(x="", y=data, fill=names, ordered=TRUE))+
ggplot(all_frame, aes(x=names, y=data, fill=names, ordered=TRUE))+
  geom_bar(width = 1, stat = "identity") +
  theme (
    legend.title = element_blank()
  ) +
  scale_y_continuous(labels=percent, breaks = all_frame$data[1:3]) +
  labs(title="Unsafety Sources in Unsafe Blocks") +
  labs(x="Unsafety Sources in Unsafe Blocks", y="Number of accesses") 
  
ggsave(all_filename, plot = last_plot(), device = "eps")

# unsafe function calls classification

res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq04-calls"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("type", "block_id"))


