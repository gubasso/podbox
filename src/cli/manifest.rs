use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub(crate) struct ManifestArgs {
    #[command(subcommand)]
    pub(crate) command: ManifestCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ManifestCommand {
    List,
    Show { name: String },
    Validate { name: String },
    Compose { name: String },
}
