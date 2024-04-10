library(airGR)

data(L0123001, package = "airGR")

setwd(getSrcDirectory(function(){})[1])

# remove warm-up
Ind_Run <- seq(which(format(BasinObs$DatesR, format = "%Y-%m-%d") == "1994-01-01"), 
               which(format(BasinObs$DatesR, format = "%Y-%m-%d") == "1998-12-31"))

# create inputs
InputsModel <- CreateInputsModel(FUN_MOD = RunModel_GR6J, DatesR = BasinObs$DatesR,
                                 Precip = BasinObs$P, PotEvap = BasinObs$E)

# run the model
RunOptions <- CreateRunOptions(FUN_MOD = RunModel_GR6J,
                               InputsModel = InputsModel, IndPeriod_Run = Ind_Run,
                               IniStates = NULL, IniResLevels = NULL,
                               IndPeriod_WarmUp = NULL)

Param <- c(1000, 0.0, 200, 1, 0, 20);
OutputsModel <- RunModel_GR6J(InputsModel = InputsModel, RunOptions = RunOptions, 
                              Param = Param)

# export for Rust test
df <- data.frame(OutputsModel$DatesR,OutputsModel$Prec, OutputsModel$PotEvap,
                 OutputsModel$Pn, OutputsModel$Ps, OutputsModel$PR, OutputsModel$Perc,
                 OutputsModel$Exch, OutputsModel$AExch1, OutputsModel$AExch2, 
                 OutputsModel$QR, OutputsModel$Prod, OutputsModel$Rout,
                 OutputsModel$Exp, OutputsModel$QD, OutputsModel$AExch,
                 OutputsModel$AE, OutputsModel$QRExp, OutputsModel$Qsim
                 )
colnames(df) <- c("Date", "precipitation", "evapotranspiration", "net_rainfall", 
                 "storage_p", "pr", "percolation", "exchange",
                 "exchange_from_routing_store", "exchange_from_direct_branch", 
                 "routing_store_outflow", "store_levels.production_store",
                 "store_levels.routing_store", "store_levels.exponential_store",
                 "outflow_from_uh2_branch", "actual_exchange", 
                 "actual_evapotranspiration", "exponential_store_outflow", "run_off")


write.csv(df, "airGR_results_L0123001_sc2.csv", row.names = FALSE)