use clap::Args;

use crate::cli::workspace::ReconcileArg;

#[derive(Debug, Args)]
pub(crate) struct ShellArgs {
    #[arg(long, value_enum)]
    pub(crate) reconcile: Option<ReconcileArg>,
    #[arg(last = true)]
    pub(crate) command: Vec<String>,
}
