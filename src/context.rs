use std::{env, path::PathBuf};

use tokio::runtime::{Builder, Runtime};
use tracing_appender::non_blocking::WorkerGuard;

use crate::{
    cli::GlobalArgs,
    config::{loader::LoadedConfig, roots::PodboxRoots},
    error::AppError,
    ui::Ui,
};

pub(crate) struct AppContext {
    pub(crate) global: GlobalArgs,
    pub(crate) runtime: Runtime,
    pub(crate) ui: Ui,
    pub(crate) log_guard: WorkerGuard,
    pub(crate) cwd: PathBuf,
    pub(crate) roots: PodboxRoots,
    pub(crate) config: LoadedConfig,
}

impl AppContext {
    pub(crate) fn build(global: GlobalArgs) -> Result<Self, AppError> {
        let roots = PodboxRoots::resolve()?;
        let log_guard = crate::logging::init(&roots)?;
        let runtime = Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|err| AppError::unexpected(format!("failed to build runtime: {err}")))?;
        let ui = Ui::from_env(global.no_color, global.quiet)?;
        let cwd = env::current_dir()
            .map_err(|err| AppError::unexpected(format!("failed to read cwd: {err}")))?;
        let config = LoadedConfig::load(
            &roots,
            global.config.as_deref(),
            global.workspace.as_deref(),
        )?;
        Ok(Self {
            global,
            runtime,
            ui,
            log_guard,
            cwd,
            roots,
            config,
        })
    }
}
