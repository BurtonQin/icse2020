

# Both compiler generated and user introduced unsafe
res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq04"
                   , header=FALSE
                   , sep=','
                   , comment.char = "#"
                   , col.names=c("blockid", "source","user","crate"))
res90 <- read.table( file="~/unsafe_analysis/analysis-data/research-questions-90-percent/rq04"
                     , header=FALSE
                     , sep=','
                     , comment.char = "#"
                     , col.names=c("blockid", "source","user","crate"))