#!/usr/bin/env Rscript
library(ggplot2)
library(Hmisc)
library(plyr)
library('scales')

res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq05"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("fn_call", "deref_ptr","asm","static_access","borrow_packed",
                                 "assign_union", "access_union", "extern_static", "argument",
                                 "from_crate","crate"
                   ))
all_filename <- "~/work/unsafe_study/paper/rq05_all.eps"
source_base_filename <- "~/work/unsafe_study/paper/rq05_source_"
calls_filename <- "~/work/unsafe_study/paper/rq05_calls.eps"

no_reason_frame <- subset( res, res$fn_call == 0 && 
                             res$deref_ptr == 0 &&
                             res$asm == 0 &&
                             res$static_access == 0 &&
                             res$borrow_packed == 0 &&
                             res$assign_union == 0 &&
                             res$access_union == 0 && 
                             res$extern_static == 0 &&
                             res$argument == 0 &&
                             res$from_crate == 0
                           )

labels <- c("Unsafe Function Call", "Derefence Raw Pointer",
            "Static Variable Use", "Assembly",
            "Access to Union", "From Arguments", "From Trait") #should improve the names 
values <- c( sum(res$fn_call), sum(res$deref_ptr), 
             sum(res$static_access), sum(res$asm),
             sum(res$access_union), sum(res$argument), sum(res$from_crate)
             )

n <- sum(values)
values <- values/n

all_frame <- data.frame(names = labels, data = values)
all_frame$names <- factor(all_frame$names, levels = all_frame$names[order(all_frame$data)])

ggplot(all_frame, aes(x=names, y=data))+
  geom_bar(stat = "identity") +
  geom_text(aes(x = names, 
                y = data + 0.02, label = sprintf("%1.2f%%", 100*data),
  )
  ) +
  scale_y_continuous(limits = c(0,0.60))+
  theme (
    legend.title = element_blank(),
    axis.text.x=element_text(angle=45, hjust=1),
    axis.text.y = element_blank()
  ) +
  labs(title="Unsafe Rust Operations in Unsafe Functions") +
  labs(x="Unsafety Rust Operations", y="Percentage") 

ggsave(all_filename, plot = last_plot(), device = "eps")

#save each number individually
fn <- paste0(source_base_filename,"n",".txt")
write(nrow(all_frame),fn,append=FALSE)
for (i in 1:length(values)) {
  fn <- paste0(source_base_filename,labels[i],".txt")
  write(percent(values[i]),fn,append=FALSE)
}