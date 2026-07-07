use clap::{Args, ValueEnum};

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum StarterArg {
    Minimal,
    None,
}

#[derive(Debug, Args)]
pub(crate) struct InitArgs {
    #[arg(long, value_enum, default_value_t = StarterArg::Minimal)]
    pub(crate) starter: StarterArg,
    #[arg(long)]
    pub(crate) dry_run: bool,
    #[arg(long)]
    pub(crate) json: bool,
}
