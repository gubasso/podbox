use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub(crate) struct ConfigArgs {
    #[command(subcommand)]
    pub(crate) command: ConfigCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ConfigCommand {
    Paths,
    Show,
    Validate,
    Get { key: String },
    Set { key: String, value: String },
    Unset { key: String },
}
