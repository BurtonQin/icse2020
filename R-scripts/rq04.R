#!/usr/bin/env Rscript
library(ggplot2)
library(Hmisc)
library(plyr)
library(scales)
library(xtable)

#p <- pipe(paste0('sed \'s/"\'"/"`"/g\' "', "~/unsafe_analysis/analysis-data/research-questions/rq06", '"'))

res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq04"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , quote="\\"
                   , col.names=c("abi", "crate", "full_path", "name", "user"))
res90 <- read.table( file="~/unsafe_analysis/analysis-data/research-questions-90-percent/rq04"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , quote="\\"
                   , col.names=c("abi", "crate", "full_path", "name", "user"))

###################################################3
# user introduced unsafe
user_only <- res[which(res$user=="true"),]
user_only90 <- res90[which(res90$user=="true"),]

user_aggregate <- count(user_only, c("abi"))
user_aggregate$freq <- user_aggregate$freq / nrow(user_only) 
user_aggregate$type <- "All"


user90_aggregate <- count(user_only90, c("abi"))
user90_aggregate$freq <- user90_aggregate$freq / nrow(user_only90) 
user90_aggregate$type <- "Most Downloaded"

exclude <- (subset(user_aggregate, freq < 0.001))[,"abi"]
user_aggregate <- subset( user_aggregate, !is.element(abi,exclude) )
user90_aggregate <- subset( user90_aggregate, !is.element(abi,exclude) )

total_frame <- rbind(user_aggregate,user90_aggregate)
options(digits = 4)

ggplot(total_frame, aes(x = abi, y = freq, group = interaction(type,abi), fill = type))+
  geom_bar(position = "dodge",stat="identity") +
  geom_text(aes(label=scales::percent(freq, scale = 100)), position=position_dodge(width=0.9), vjust=-0.25) + 
  theme (
    legend.title = element_blank(),
    axis.text.x=element_text(angle=45, hjust=1),
    axis.text.y = element_blank()
  ) +
  labs(title="Unsafe Rust in Blocks (User Introduced Unsafe Only)") +
  labs(x="Unsafe Rust Operations", y="Percentage") +
  scale_fill_grey()

ggsave("~/work/unsafe-analysis-data/paper/rq03_blocks_user.eps", plot = last_plot(), device = "eps")

## top intrinsics
#user only
intrinsics_aggregate <- count((subset(res, abi == "RustIntrinsic" && user == TRUE))[,"name"])
intrinsics_aggregate$freq <- intrinsics_aggregate$freq / length((subset(res, abi == "RustIntrinsic" && user == TRUE))[,"name"])

top5 <- (intrinsics_aggregate[order(intrinsics_aggregate$freq, decreasing = TRUE),])[1:5,]
colnames(top5) <- c("Function", "Percentage")

filename <- "~/work/unsafe-analysis-data/paper/rq04_intrinsics_user_table.txt" 
xx <- xtable(top5,caption = "Top intrinsics calls", label="tbl:allintrinsics-user", filename=filename)
print(xx,file=filename)

