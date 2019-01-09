#!/usr/bin/env Rscript
library(ggplot2)
library(Hmisc)
library(plyr)
library(scales)
library(xtable)

res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions-servo-all/rq06"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , quote="\\"
                   , col.names=c("abi", "crate", "full_path", "name", "user"))
res90 <- read.table( file="~/unsafe_analysis/analysis-data/research-questions-servo-only/rq06"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , quote="\\"
                   , col.names=c("abi", "crate", "full_path", "name", "user"))

res_aggregate <- count(res, c("abi"))
res_aggregate$freq <- res_aggregate$freq / nrow(res) 
res_aggregate$type <- "All Dependencies"

res90_aggregate <- count(res90, c("abi"))
res90_aggregate$freq <- res90_aggregate$freq / nrow(res90) 
res90_aggregate$type <- "Servo Crates"

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
    axis.text.y = element_blank()
  ) +
  labs(title="Called Unsafe Functions ABI in Servo") +
  labs(x="ABI", y="Percentage") +
  scale_fill_grey()

ggsave("~/work/unsafe-analysis-data/paper/rq04_servo_all.eps", plot = last_plot(), device = "eps")

for (i in 1:nrow(total_frame)) {
  fn <- paste0("~/work/unsafe-analysis-data/paper/rq04_servo_all_",
               total_frame$type[i], "_",
               total_frame$abi[i], 
               ".txt")
  write(percent(total_frame$freq[i]),fn,append=FALSE)
}

###################################################3
# user introduced unsafe
user_only <- res[which(res$user=="true"),]
user_only90 <- res90[which(res90$user=="true"),]

user_aggregate <- count(user_only, c("abi"))
user_aggregate$freq <- user_aggregate$freq / nrow(user_only) 
user_aggregate$type <- "All Dependencies"


user90_aggregate <- count(user_only90, c("abi"))
user90_aggregate$freq <- user90_aggregate$freq / nrow(user_only90) 
user90_aggregate$type <- "Servo Crates"

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
  labs(title="Called Unsafe Functions ABI in Servo (User Introduced Unsafe Only)") +
  labs(x="Unsafe Rust Operations", y="Percentage") +
  scale_fill_grey()

ggsave("~/work/unsafe-analysis-data/paper/rq04_servo_user.eps", plot = last_plot(), device = "eps")

