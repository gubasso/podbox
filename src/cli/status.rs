use clap::Args;

#[derive(Debug, Args)]
pub(crate) struct StatusArgs {
    #[arg(long)]
    pub(crate) json: bool,
}
