use clap::CommandFactory;

use crate::{cli::completion::CompletionArgs, error::AppError, ui::Ui};

pub(crate) fn run(ui: &Ui, args: CompletionArgs) -> Result<(), AppError> {
    let mut command = crate::cli::Cli::command();
    let mut out = Vec::new();
    clap_complete::generate(args.shell, &mut command, "podbox", &mut out);
    ui.stdout_raw(&out)
}

pub(crate) fn manpage(ui: &Ui) -> Result<(), AppError> {
    let command = crate::cli::Cli::command();
    let mut out = Vec::new();
    clap_mangen::Man::new(command)
        .render(&mut out)
        .map_err(|err| AppError::unexpected(format!("failed to render man page: {err}")))?;
    ui.stdout_raw(&out)
}
