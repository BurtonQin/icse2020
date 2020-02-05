#!/usr/bin/env Rscript
library(ggplot2)
#library(Hmisc)
library(plyr)
library(scales)
#library(xtable)

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

res <- res[which(res$user=="true"),]
res90 <- res90[which(res90$user=="true"),]

res_aggregate <- count(res, c("abi"))
res_aggregate$freq <- res_aggregate$freq / nrow(res) 
res_aggregate$type <- "All"

res90_aggregate <- count(res90, c("abi"))
res90_aggregate$freq <- res90_aggregate$freq / nrow(res90) 
res90_aggregate$type <- "Most Downloaded"

exclude <- (subset(res_aggregate, freq < 0.03))[,"abi"]
res_aggregate <- subset( res_aggregate, !is.element(abi,exclude) )
res90_aggregate <- subset( res90_aggregate, !is.element(abi,exclude) )

total_frame <- rbind(res_aggregate,res90_aggregate)
options(digits = 4)

ggplot(total_frame, aes(x = abi, y = freq, group = interaction(type,abi), fill = type))+
  geom_bar(position = "dodge",stat="identity") +
  geom_text(aes(label=scales::percent(freq, scale = 100)), position=position_dodge(width=0.9), vjust=-0.25) + 
  theme (
    legend.title = element_blank(),
    legend.position="top",
    text = element_text(size=25),
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
  labs(x="Abstract Binary Interface", y="Percentage") +
  scale_fill_grey()

ggsave("~/unsafe_analysis/camera-ready/rq04_all.eps", plot = last_plot(), device = "eps")


core <- res[which(res$crate=="core"),]
f_core <- nrow(core)/nrow(res) * 100
print('all core')
print(f_core)

res_aggregate <- count(res, c("crate"))
res_aggregate$freq <- res_aggregate$freq / nrow(res) 


res90_aggregate <- count(res90, c("abi"))
res90_aggregate$freq <- res90_aggregate$freq / nrow(res90) 
res90_aggregate$type <- "Most Downloaded"

core90 <- res90[which(res90$crate=="core"),]
f_core <- nrow(core)/nrow(res) * 100
print('md core')
print(f_core)
