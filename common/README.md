This crate contains the code that is shared between xtasks, and all the other services.

Since the xtasks need to execute, for example, database related tasks, xtask need to 
know about DatabaseSettings, and so, all the configuration related code is in common.
