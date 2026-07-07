use clap::{Args, Subcommand, ValueEnum};

#[derive(Debug, Args)]
pub(crate) struct WorkspaceArgs {
    #[command(subcommand)]
    pub(crate) command: WorkspaceCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum WorkspaceCommand {
    Up,
    Reconcile {
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        yes: bool,
    },
    Shell {
        #[arg(long, value_enum)]
        reconcile: Option<ReconcileArg>,
        #[arg(last = true)]
        command: Vec<String>,
    },
    Exec {
        #[arg(required = true, last = true)]
        command: Vec<String>,
    },
    Down {
        #[arg(long)]
        dry_run: bool,
        #[arg(long, alias = "force")]
        yes: bool,
    },
    Status,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum ReconcileArg {
    Auto,
    Always,
    Never,
}
