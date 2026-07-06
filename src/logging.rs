use std::fs;

use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{config::roots::PodboxRoots, error::AppError};

pub(crate) fn init(
    roots: &PodboxRoots,
) -> Result<tracing_appender::non_blocking::WorkerGuard, AppError> {
    fs::create_dir_all(&roots.state).map_err(|err| AppError::io(roots.state.clone(), err))?;
    let file_appender = tracing_appender::rolling::never(&roots.state, "podbox.log");
    let (writer, guard) = tracing_appender::non_blocking(file_appender);
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));
    let layer = fmt::layer().json().with_ansi(false).with_writer(writer);
    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(layer)
        .try_init();
    tracing::debug!("logging initialized");
    Ok(guard)
}
