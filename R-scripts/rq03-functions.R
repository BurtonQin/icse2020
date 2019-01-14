#!/usr/bin/env Rscript
library(ggplot2)
library(Hmisc)
library(plyr)
library('scales')

res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq05"
                   , header=FALSE
                   , sep=','
                   , comment.char = "#"
                   , col.names=c("source", "user","crate"))

res90 <- read.table( file="~/unsafe_analysis/analysis-data/research-questions-90-percent/rq05"
                   , header=FALSE
                   , sep=','
                   , comment.char = "#"
                   , col.names=c("source", "user","crate"))

res_aggregate <- count(res, c("source"))
res_aggregate$freq <- res_aggregate$freq / nrow(res) 
res_aggregate$type <- "All"

res90_aggregate <- count(res90, c("source"))
res90_aggregate$freq <- res90_aggregate$freq / nrow(res90) 
res90_aggregate$type <- "Most Downloaded"

# too_small <- (subset(res_aggregate, freq < 0.001))[,"source"]
# too_small90 <- (subset(res90_aggregate, freq < 0.001))[,"source"]
# exclude <- intersect(too_small, too_small90)

exclude <- (subset(res_aggregate, freq < 0.001))[,"source"]
res_aggregate <- subset( res_aggregate, !is.element(source,exclude) )
res90_aggregate <- subset( res90_aggregate, !is.element(source,exclude) )

total_frame <- rbind(res_aggregate,res90_aggregate)
options(digits = 4)

ggplot(total_frame, aes(x = source, y = freq, group = interaction(type,source), fill = type))+
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
  labs(title="Unsafe Rust in Declared Unsafe Functions") +
  labs(x="Unsafe Rust Operations", y="Percentage") +
  scale_fill_grey()

ggsave("~/work/unsafe-analysis-data/paper/rq03_functions_all.eps", plot = last_plot(), device = "eps")

for (i in 1:nrow(total_frame)) {
  fn <- paste0("~/work/unsafe-analysis-data/paper/rq03_functions_all_",
               total_frame$type[i], "_",
               total_frame$source[i], 
               ".txt")
  write(percent(total_frame$freq[i]),fn,append=FALSE)
}

###################################################3
# user introduced unsafe
user_only <- res[which(res$user=="true"),]
user_only90 <- res90[which(res90$user=="true"),]

user_aggregate <- count(user_only, c("source"))
user_aggregate$freq <- user_aggregate$freq / nrow(user_only) 
user_aggregate$type <- "All"

user90_aggregate <- count(user_only90, c("source"))
user90_aggregate$freq <- user90_aggregate$freq / nrow(user_only90) 
user90_aggregate$type <- "Most Downloaded"

exclude <- (subset(user_aggregate, freq < 0.001))[,"source"]
user_aggregate <- subset( user_aggregate, !is.element(source,exclude) )
user90_aggregate <- subset( user90_aggregate, !is.element(source,exclude) )

total_frame <- rbind(user_aggregate,user90_aggregate)
options(digits = 4)

ggplot(total_frame, aes(x = source, y = freq, group = interaction(type,source), fill = type))+
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
  labs(title="Unsafe Rust in Declared Unsafe Functions (User Introduced Unsafe Only)") +
  labs(x="Unsafe Rust Operations", y="Percentage") +
  scale_fill_grey()

ggsave("~/work/unsafe-analysis-data/paper/rq03_functions_user.eps", plot = last_plot(), device = "eps")

for (i in 1:nrow(total_frame)) {
  fn <- paste0("~/work/unsafe-analysis-data/paper/rq03_functions_user_",
               total_frame$type[i], "_",
               total_frame$source[i], 
               ".txt")
  write(percent(total_frame$freq[i]),fn,append=FALSE)
}
