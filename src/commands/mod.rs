pub(crate) mod config;
pub(crate) mod doctor;
pub(crate) mod manifest;
pub(crate) mod version;

use crate::{cli::Commands, context::AppContext, error::AppError};

pub(crate) fn dispatch(ctx: &AppContext, command: Commands) -> Result<(), AppError> {
    match command {
        Commands::Version(args) => version::run(ctx, args),
        Commands::Doctor(args) => doctor::run(ctx, args),
        Commands::Config(args) => config::run(ctx, args),
        Commands::Manifest(args) => manifest::run(ctx, args),
    }
}
