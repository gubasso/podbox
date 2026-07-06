pub(crate) mod config;
pub(crate) mod doctor;
pub(crate) mod image;
pub(crate) mod manifest;
pub(crate) mod version;

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::error::AppError;

#[derive(Debug, Parser)]
#[command(
    name = "podbox",
    version = crate::cli::version::version_text(),
    about = "Secure, reproducible per-workspace dev sandbox"
)]
pub(crate) struct Cli {
    #[command(flatten)]
    pub(crate) global: GlobalArgs,
    #[command(subcommand)]
    pub(crate) command: Commands,
}

impl Cli {
    pub(crate) fn validate(&self) -> Result<(), AppError> {
        if self.global.quiet && self.global.verbose > 0 {
            return Err(AppError::usage("quiet and verbose cannot be used together"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Args)]
pub(crate) struct GlobalArgs {
    #[arg(long, global = true)]
    pub(crate) config: Option<PathBuf>,
    #[arg(long, global = true)]
    pub(crate) manifest: Option<String>,
    #[arg(long, global = true)]
    pub(crate) workspace: Option<PathBuf>,
    #[arg(long, global = true)]
    pub(crate) json: bool,
    #[arg(short, long, global = true)]
    pub(crate) quiet: bool,
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub(crate) verbose: u8,
    #[arg(long, global = true)]
    pub(crate) no_color: bool,
    #[arg(long, global = true, alias = "force")]
    pub(crate) yes: bool,
    #[arg(long, global = true)]
    pub(crate) dry_run: bool,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    // U1: shell is intentionally reserved as the first help entry in the live round.
    Version(version::VersionArgs),
    Doctor(doctor::DoctorArgs),
    Config(config::ConfigArgs),
    Manifest(manifest::ManifestArgs),
    Image(image::ImageArgs),
}
