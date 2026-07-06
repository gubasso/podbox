//! podbox library crate.

#![allow(dead_code)]

pub(crate) mod adapters;
pub(crate) mod cli;
pub(crate) mod commands;
pub(crate) mod config;
pub(crate) mod context;
pub(crate) mod domain;
pub(crate) mod error;
pub(crate) mod exit;
pub(crate) mod logging;
pub(crate) mod services;
pub(crate) mod ui;
pub(crate) mod util;

use std::process::ExitCode;

use clap::Parser;

pub fn run() -> ExitCode {
    match run_inner() {
        Ok(()) => ExitCode::from(exit::SUCCESS),
        Err(err) => {
            if let Ok(ui) = ui::Ui::from_env(false, false) {
                let _ = ui.stderr_line(&format!("{}: {}", err.kind(), err));
            }
            ExitCode::from(err.exit_code())
        }
    }
}

fn run_inner() -> Result<(), error::AppError> {
    let cli = cli::Cli::parse();
    cli.validate()?;
    let ctx = context::AppContext::build(cli.global.clone())?;
    commands::dispatch(&ctx, cli.command)
}
