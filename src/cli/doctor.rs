use clap::{Args, ValueEnum};
use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DoctorScope {
    All,
    Host,
    Runtime,
    Config,
    Workspace,
    Images,
    Network,
}

#[derive(Debug, Args)]
pub(crate) struct DoctorArgs {
    #[arg(long, value_enum, default_value_t = DoctorScope::All)]
    pub(crate) scope: DoctorScope,
    #[arg(long)]
    pub(crate) json: bool,
    #[arg(long)]
    pub(crate) quiet: bool,
}
