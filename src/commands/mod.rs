pub(crate) mod config;
pub(crate) mod doctor;
pub(crate) mod image;
pub(crate) mod manifest;
pub(crate) mod network;
pub(crate) mod shell;
pub(crate) mod version;
pub(crate) mod workspace;

use crate::{cli::Commands, context::AppContext, error::AppError};

pub(crate) fn dispatch(ctx: &AppContext, command: Commands) -> Result<u8, AppError> {
    match command {
        Commands::Shell(args) => shell::run(ctx, args),
        Commands::Version(args) => version::run(ctx, args).map(|()| crate::exit::SUCCESS),
        Commands::Doctor(args) => doctor::run(ctx, args).map(|()| crate::exit::SUCCESS),
        Commands::Workspace(args) => workspace::run(ctx, args),
        Commands::Network(args) => network::run(ctx, args).map(|()| crate::exit::SUCCESS),
        Commands::Config(args) => config::run(ctx, args).map(|()| crate::exit::SUCCESS),
        Commands::Manifest(args) => manifest::run(ctx, args).map(|()| crate::exit::SUCCESS),
        Commands::Image(args) => image::run(ctx, args),
    }
}
