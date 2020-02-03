library(dplyr)

blocks <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq01-blocks"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("blocks", "user", "total", "name"))

no_unsafe_blocks <- (subset(blocks, user==0))[c(4)]

functions <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq01-func"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("functions", "name"))

no_usafe_functions <- (subset(functions, functions==0))[c(2)]

res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq02-opt"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("crate", "no"))

res1 <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq02-pes"
                    , header=FALSE
                    , sep='\t'
                    , comment.char = "#"
                    , col.names=c("crate", "no"))

with_possibly_unsafe_opt <- (subset(res,no>0))[1]
with_possibly_unsafe_pes <- (subset(res1,no>0))[1]

colnames(with_possibly_unsafe_opt) <- c("name")
colnames(with_possibly_unsafe_pes) <- c("name")

no_unsafe <- intersect(no_unsafe_blocks, no_usafe_functions)
common_opt <- intersect( with_possibly_unsafe_opt, no_unsafe )
common_pes <- intersect( with_possibly_unsafe_pes, no_unsafe )

print(nrow(common_opt)/nrow(no_unsafe))
print(nrow(common_pes)/nrow(no_unsafe))

