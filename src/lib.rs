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
    let cli = cli::Cli::parse();
    let json = cli.global.json;
    let no_color = cli.global.no_color || json;
    let quiet = cli.global.quiet;
    match run_inner(cli) {
        Ok(code) => ExitCode::from(code),
        Err(err) => {
            if let Ok(ui) = ui::Ui::from_env(no_color, json && quiet) {
                if json {
                    let _ = ui.json(&JsonError::from(&err));
                } else {
                    let _ = ui.stderr_line(&format!("{}: {}", err.kind(), err));
                }
            }
            ExitCode::from(err.exit_code())
        }
    }
}

#[derive(serde::Serialize)]
struct JsonError {
    schema_version: u32,
    error: JsonErrorBody,
}

#[derive(serde::Serialize)]
struct JsonErrorBody {
    code: &'static str,
    message: String,
}

impl From<&error::AppError> for JsonError {
    fn from(err: &error::AppError) -> Self {
        Self {
            schema_version: util::schema_version(),
            error: JsonErrorBody {
                code: err.kind(),
                message: err.to_string(),
            },
        }
    }
}

fn run_inner(cli: cli::Cli) -> Result<u8, error::AppError> {
    cli.validate()?;
    // Pure verbs (`version`, `completion`, hidden manpage emitter) must short-circuit
    // before any runtime/state/logging init: they build no runtime adapter and mutate
    // no state (02 §4.9, U1). They render through a standalone `Ui` only.
    let json = cli.global.json;
    let no_color = cli.global.no_color || json;
    let quiet = cli.global.quiet;
    match cli.command {
        cli::Commands::Version(args) => {
            let ui = ui::Ui::from_env(no_color, quiet)?;
            commands::version::run(&ui, json, args).map(|()| exit::SUCCESS)
        }
        cli::Commands::Completion(args) => {
            let ui = ui::Ui::from_env(no_color, quiet)?;
            commands::completion::run(&ui, args).map(|()| exit::SUCCESS)
        }
        cli::Commands::Manpage => {
            let ui = ui::Ui::from_env(no_color, quiet)?;
            commands::completion::manpage(&ui).map(|()| exit::SUCCESS)
        }
        command => {
            let ctx = context::AppContext::build(cli.global.clone())?;
            commands::dispatch(&ctx, command)
        }
    }
}
