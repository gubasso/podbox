use clap::{Args, Subcommand, ValueEnum};

#[derive(Debug, Args)]
pub(crate) struct ImageArgs {
    #[command(subcommand)]
    pub(crate) command: ImageCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ImageCommand {
    Build {
        names: Vec<String>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        full_rebuild: bool,
        #[arg(long, value_enum)]
        pull_policy: Option<PullPolicyArg>,
    },
    List,
    Inspect {
        name: String,
    },
    #[command(hide = true)]
    Status {
        name: String,
    },
    Prune {
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum PullPolicyArg {
    Missing,
    Newer,
    Always,
    Never,
}
