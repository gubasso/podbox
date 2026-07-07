use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub(crate) struct NetworkArgs {
    #[command(subcommand)]
    pub(crate) command: NetworkCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum NetworkCommand {
    Show,
    Allow {
        entry: String,
        #[arg(id = "manifest_target", long = "manifest-target")]
        manifest: bool,
    },
}
