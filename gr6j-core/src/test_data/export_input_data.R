library(airGR)

data(L0123001, package = "airGR")

setwd(".")

write.csv(data, "airGR_L0123001_dataset.csv", row.names = FALSE)