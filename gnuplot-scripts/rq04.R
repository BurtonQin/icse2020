#!/usr/bin/env Rscript
library(ggplot2)
library(Hmisc)

res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq04"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("fn_call", "deref_ptr","asm","static_access","borrow_packed",
                                 "assign_union", "access_union", "extern_static", 
                                 "crate", "block_id"
                                 ))