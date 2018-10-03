#!/usr/bin/env Rscript
library(ggplot2)
library(Hmisc)
library(plyr)
library(scales)

res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq04"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("fn_call", "deref_ptr","asm","static_access","borrow_packed",
                                 "assign_union", "access_union", "extern_static", 
                                 "crate", "block_id"
                                 ))

all_filename <- "~/work/unsafe_study/paper/rq04_all.eps"
source_base_filename <- "~/work/unsafe_study/paper/rq04_source_"
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

ggplot(all_frame, aes(x=names, y=data))+
  geom_bar(stat='identity') +
  geom_text(aes(x = names, 
                y = data + 0.02, label = sprintf("%1.4f%%", 100*data))) +
  theme (
    legend.title = element_blank(),
    axis.text.x=element_text(angle=45, hjust=1),
    axis.text.y = element_blank()
  ) +
  labs(title="Unsafe Blocks") +
  labs(x="Unsafety Sources", y="Percentage") 
  
ggsave(all_filename, plot = last_plot(), device = "eps")

fn <- paste0(source_base_filename,"n",".txt")
write(nrow(all_frame),fn,append=FALSE)
for (i in 1:length(values)) {
  fn <- paste0(source_base_filename,labels[i],".txt")
  write(percent(values[i]),fn,append=FALSE)
}

# unsafe function calls classification
res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq04-calls"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("type", "block_id"))
cols <- unique(res$type)
values <- array(dim=length(cols))
for (i in 1:length(cols)) {
  dfi <- res[ which(res$type == cols[i]),]
  values[i] <- nrow(dfi)
}
n <- sum(values)
values <- values/n
all_frame <- data.frame(names = cols, data = values)
all_frame$names <- factor(all_frame$names, levels = all_frame$names[order(all_frame$data)])

ggplot(all_frame, aes(x=names, y=data))+
  geom_bar(stat = "identity") +
  geom_text(aes(x = names, 
                y = data + 0.02, label = sprintf("%1.4f%%", 100*data)
                )
            ) +
  theme (
    legend.title = element_blank(),
    axis.text.x=element_text(angle=45, hjust=1),
    axis.text.y = element_blank()
  ) +
  labs(title="Unsafe Function Calls in Unsafe Blocks") +
  labs(x="Unsafe Function Call Abi", y="Percentage") 

ggsave(calls_filename, plot = last_plot(), device = "eps")

fn <- paste0(source_base_filename,"calls_n",".txt")
write(nrow(all_frame),fn,append=FALSE)
for (i in 1:length(values)) {
  fn <- paste0(source_base_filename,cols[i],".txt")
  write(percent(values[i]),fn,append=FALSE)
}
