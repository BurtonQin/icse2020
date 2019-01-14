#!/usr/bin/env Rscript
library(ggplot2)
library(Hmisc)
library(plyr)
library(scales)
library(xtable)

#p <- pipe(paste0('sed \'s/"\'"/"`"/g\' "', "~/unsafe_analysis/analysis-data/research-questions/rq06", '"'))

res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq06"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , quote="\\"
                   , col.names=c("abi", "crate", "full_path", "name", "user"))
res90 <- read.table( file="~/unsafe_analysis/analysis-data/research-questions-90-percent/rq06"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , quote="\\"
                   , col.names=c("abi", "crate", "full_path", "name", "user"))

res_aggregate <- count(res, c("abi"))
res_aggregate$freq <- res_aggregate$freq / nrow(res) 
res_aggregate$type <- "All"

res90_aggregate <- count(res90, c("abi"))
res90_aggregate$freq <- res90_aggregate$freq / nrow(res90) 
res90_aggregate$type <- "Most Downloaded"

exclude <- (subset(res_aggregate, freq < 0.001))[,"abi"]
res_aggregate <- subset( res_aggregate, !is.element(abi,exclude) )
res90_aggregate <- subset( res90_aggregate, !is.element(abi,exclude) )

total_frame <- rbind(res_aggregate,res90_aggregate)
options(digits = 4)

ggplot(total_frame, aes(x = abi, y = freq, group = interaction(type,abi), fill = type))+
  geom_bar(position = "dodge",stat="identity") +
  geom_text(aes(label=scales::percent(freq, scale = 100)), position=position_dodge(width=0.9), vjust=-0.25) + 
  theme (
    legend.title = element_blank(),
    axis.text.x=element_text(angle=45, hjust=1),
    axis.text.y = element_blank(),
    panel.background = element_rect(fill = "white",
                                    colour = "white",
                                    size = 0.5, linetype = "solid"),
    panel.grid.major = element_line(size = 0.5, linetype = 'solid',
                                    colour = "grey"), 
    panel.grid.minor = element_line(size = 0.25, linetype = 'solid',
                                    colour = "white")
  ) +
  labs(title="Called Unsafe Function ABI") +
  labs(x="ABI", y="Percentage") +
  scale_fill_grey()

ggsave("~/work/unsafe-analysis-data/paper/rq04_all.eps", plot = last_plot(), device = "eps")

for (i in 1:nrow(total_frame)) {
  fn <- paste0("~/work/unsafe-analysis-data/paper/rq04_all_",
               total_frame$type[i], "_",
               total_frame$abi[i], 
               ".txt")
  write(percent(total_frame$freq[i]),fn,append=FALSE)
}

for (i in 1:nrow(total_frame)) {
  fn <- paste0("~/work/unsafe-analysis-data/paper/rq04_user_",
               total_frame$type[i], "_",
               total_frame$source[i], 
               ".txt")
  write(percent(total_frame$freq[i]),fn,append=FALSE)
}

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
intrinsics_aggregate <- count((subset(res, abi == "RustIntrinsic"))[,"name"])
intrinsics_aggregate$freq <- intrinsics_aggregate$freq / length((subset(res, abi == "RustIntrinsic"))[,"name"])

top5 <- (intrinsics_aggregate[order(intrinsics_aggregate$freq, decreasing = TRUE),])[1:5,]
colnames(top5) <- c("Function", "Percentage")

filename <- "~/work/unsafe-analysis-data/paper/rq04_intrinsics_table.txt" 
xx <- xtable(top5,caption = "Top intrinsics calls", label="tbl:allintrinsics", filename=filename)
print(xx,file=filename)

#user only
intrinsics_aggregate <- count((subset(res, abi == "RustIntrinsic" && user == TRUE))[,"name"])
intrinsics_aggregate$freq <- intrinsics_aggregate$freq / length((subset(res, abi == "RustIntrinsic" && user == TRUE))[,"name"])

top5 <- (intrinsics_aggregate[order(intrinsics_aggregate$freq, decreasing = TRUE),])[1:5,]
colnames(top5) <- c("Function", "Percentage")

filename <- "~/work/unsafe-analysis-data/paper/rq04_intrinsics_user_table.txt" 
xx <- xtable(top5,caption = "Top intrinsics calls", label="tbl:allintrinsics-user", filename=filename)
print(xx,file=filename)

