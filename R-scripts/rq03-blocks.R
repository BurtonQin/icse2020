#!/usr/bin/env Rscript
library(ggplot2)
library(Hmisc)
library(plyr)
library(scales)

# Both compiler generates and user introduced unsafe
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

res_aggregate <- count(res, c("source"))
res_aggregate$freq <- res_aggregate$freq / nrow(res) 
res_aggregate$type <- "All"


res90_aggregate <- count(res90, c("source"))
res90_aggregate$freq <- res90_aggregate$freq / nrow(res90) 
res90_aggregate$type <- "Most Downloaded"

total_frame <- rbind(res_aggregate,res90_aggregate)
options(digits = 4)

ggplot(total_frame, aes(x = source, y = freq, group = interaction(type,source), fill = type))+
  geom_bar(position = "dodge",stat="identity") +
  geom_text(aes(label=scales::percent(freq, scale = 100)), position=position_dodge(width=0.9), vjust=-0.25) + 
  theme (
    legend.title = element_blank(),
    axis.text.x=element_text(angle=45, hjust=1),
    axis.text.y = element_blank()
  ) +
  labs(title="Unsafe Rust in Blocks") +
  labs(x="Unsafe Rust Operations", y="Percentage") +
  scale_fill_grey()
  
ggsave("~/work/unsafe-analysis-data/paper/rq03_blocks_all.eps", plot = last_plot(), device = "eps")

for (i in 1:nrow(total_frame)) {
  fn <- paste0("~/work/unsafe-analysis-data/paper/rq03_blocks_all_",
               total_frame$type[i], "_",
               total_frame$source[i], 
               ".txt")
  write(percent(total_frame$freq[i]),fn,append=FALSE)
}

###################################################3
# user introduced unsafe
user_only <- res[which(res$user=="true"),]
user_only90 <- res[which(res90$user=="true"),]

user_aggregate <- count(user_only, c("source"))
user_aggregate$freq <- user_aggregate$freq / nrow(user_only) 
user_aggregate$type <- "All"


user90_aggregate <- count(user_only90, c("source"))
user90_aggregate$freq <- user90_aggregate$freq / nrow(user_only90) 
user90_aggregate$type <- "Most Downloaded"

total_frame <- rbind(user_aggregate,user90_aggregate)
options(digits = 4)

ggplot(total_frame, aes(x = source, y = freq, group = interaction(type,source), fill = type))+
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

for (i in 1:nrow(total_frame)) {
  fn <- paste0("~/work/unsafe-analysis-data/paper/rq03_blocks_user_",
               total_frame$type[i], "_",
               total_frame$source[i], 
               ".txt")
  write(percent(total_frame$freq[i]),fn,append=FALSE)
}

###############

# unsafe function calls classification
res <- read.table( file="~/unsafe_analysis/analysis-data/research-questions/rq04-calls"
                   , header=FALSE
                   , sep='\t'
                   , comment.char = "#"
                   , col.names=c("type", "block_id", "user"))
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
