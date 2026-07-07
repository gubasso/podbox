use clap::Args;
use clap_complete::Shell;

#[derive(Debug, Args)]
pub(crate) struct CompletionArgs {
    #[arg(value_enum)]
    pub(crate) shell: Shell,
}
