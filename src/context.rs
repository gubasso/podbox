use std::{env, path::PathBuf, sync::Arc};

use tokio::runtime::{Builder, Runtime};
use tracing_appender::non_blocking::WorkerGuard;

use crate::{
    adapters::{
        runtime::{self, RuntimeAdapter},
        state_store::{FsStateStore, StateStore},
    },
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
    pub(crate) state_store: Arc<dyn StateStore>,
    pub(crate) runtime_adapter: Arc<dyn RuntimeAdapter>,
}

impl AppContext {
    pub(crate) fn build(global: GlobalArgs) -> Result<Self, AppError> {
        let roots = PodboxRoots::resolve()?;
        let log_guard = crate::logging::init(&roots)?;
        let runtime = Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|err| AppError::unexpected(format!("failed to build runtime: {err}")))?;
        let ui = Ui::from_env(global.no_color || global.json, global.quiet)?;
        let cwd = env::current_dir()
            .map_err(|err| AppError::unexpected(format!("failed to read cwd: {err}")))?;
        let config = LoadedConfig::load(
            &roots,
            global.config.as_deref(),
            global.workspace.as_deref(),
        )?;
        let state_store = Arc::new(FsStateStore::new(roots.state.clone()));
        let runtime_adapter = runtime::default_adapter();
        Ok(Self {
            global,
            runtime,
            ui,
            log_guard,
            cwd,
            roots,
            config,
            state_store,
            runtime_adapter,
        })
    }

    /// Assemble a context with an injected runtime adapter for tests, backing all
    /// roots on a caller-owned tempdir. Networking/logging are wired to inert
    /// sinks so no host I/O escapes the test.
    #[cfg(test)]
    pub(crate) fn for_test(roots: PodboxRoots, runtime_adapter: Arc<dyn RuntimeAdapter>) -> Self {
        use crate::adapters::state_store::FsStateStore;

        let runtime = Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime");
        let (_writer, log_guard) = tracing_appender::non_blocking(std::io::sink());
        let state_store = Arc::new(FsStateStore::new(roots.state.clone()));
        Self {
            global: GlobalArgs {
                config: None,
                manifest: None,
                workspace: None,
                json: false,
                quiet: true,
                verbose: 0,
                no_color: true,
                yes: false,
                dry_run: false,
            },
            runtime,
            ui: Ui::from_env(true, true).expect("test ui"),
            log_guard,
            cwd: PathBuf::from("/"),
            roots,
            config: LoadedConfig::empty(),
            state_store,
            runtime_adapter,
        }
    }
}
