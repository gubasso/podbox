use clap::Args;

pub(crate) const BUILD_SHA: &str = match option_env!("PODBOX_BUILD_SHA") {
    Some(value) => value,
    None => "unknown",
};
pub(crate) const BUILD_DATE: &str = match option_env!("PODBOX_BUILD_DATE") {
    Some(value) => value,
    None => "unknown",
};
/// Runtime-built version string so the top-level `--version` flag reflects the
/// injected `PODBOX_BUILD_SHA` / `PODBOX_BUILD_DATE` metadata, identically to the
/// `version` subcommand. `concat!` only accepts string literals, so the SHA/date
/// consts cannot be interpolated at compile time; build it lazily instead.
pub(crate) fn version_text() -> &'static str {
    static TEXT: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
        format!(
            "{} (sha {}, built {})",
            env!("CARGO_PKG_VERSION"),
            BUILD_SHA,
            BUILD_DATE
        )
    });
    TEXT.as_str()
}

#[derive(Debug, Args)]
pub(crate) struct VersionArgs {
    #[arg(long)]
    pub(crate) json: bool,
}
