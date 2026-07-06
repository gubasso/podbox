use crate::{
    cli::image::{ImageArgs, ImageCommand, PullPolicyArg},
    context::AppContext,
    domain::image::PullPolicy,
    error::AppError,
    services::image_build::{ImageBuildInput, ImageBuildOutcome, ImageBuildService},
};

pub(crate) fn run(ctx: &AppContext, args: ImageArgs) -> Result<u8, AppError> {
    match args.command {
        ImageCommand::Build {
            name,
            full_rebuild,
            pull_policy,
        } => build(ctx, &name, full_rebuild, pull_policy),
        ImageCommand::Status { name } => status(ctx, &name).map(|()| crate::exit::SUCCESS),
        ImageCommand::Prune { yes } => prune(ctx, yes).map(|()| crate::exit::SUCCESS),
    }
}

fn build(
    ctx: &AppContext,
    name: &str,
    full_rebuild: bool,
    pull_policy: Option<PullPolicyArg>,
) -> Result<u8, AppError> {
    let pull_policy = pull_policy
        .map(PullPolicy::from)
        .unwrap_or_else(|| PullPolicy::from_config(&ctx.config.config.images.pull_policy));
    match ImageBuildService::new(ctx).build(ImageBuildInput {
        name: name.to_string(),
        full_rebuild,
        pull_policy,
    })? {
        ImageBuildOutcome::Report(report) => {
            if ctx.global.json {
                ctx.ui.json(&report)?;
            } else {
                ctx.ui.stdout_line(&format!("image: {}", report.image))?;
                ctx.ui.stdout_line(&format!("digest: {}", report.digest))?;
                ctx.ui.stdout_line(if report.reused {
                    "reused: true"
                } else {
                    "reused: false"
                })?;
            }
            Ok(crate::exit::SUCCESS)
        }
        ImageBuildOutcome::ChildExit(code) => Ok(code),
    }
}

fn status(ctx: &AppContext, name: &str) -> Result<(), AppError> {
    match ImageBuildService::new(ctx).status(name)? {
        Some(report) if ctx.global.json => ctx.ui.json(&report),
        Some(report) => {
            ctx.ui.stdout_line(&format!("image: {}", report.image))?;
            ctx.ui.stdout_line(&format!("digest: {}", report.digest))
        }
        None => Err(AppError::Build {
            message: format!("image `{name}` is not built"),
        }),
    }
}

fn prune(ctx: &AppContext, yes: bool) -> Result<(), AppError> {
    if !(yes || ctx.global.yes) {
        return Err(AppError::DestructiveRefused {
            message: "image prune requires --yes".to_string(),
        });
    }
    let count = ImageBuildService::new(ctx).prune()?;
    ctx.ui.stdout_line(&format!("pruned: {count}"))
}

impl From<PullPolicyArg> for PullPolicy {
    fn from(value: PullPolicyArg) -> Self {
        match value {
            PullPolicyArg::Missing => PullPolicy::Missing,
            PullPolicyArg::Newer => PullPolicy::Newer,
            PullPolicyArg::Always => PullPolicy::Always,
            PullPolicyArg::Never => PullPolicy::Never,
        }
    }
}

impl PullPolicy {
    fn from_config(value: &str) -> Self {
        match value {
            "newer" => PullPolicy::Newer,
            "always" => PullPolicy::Always,
            "never" => PullPolicy::Never,
            _ => PullPolicy::Missing,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use camino::Utf8PathBuf;

    use crate::{
        adapters::runtime::stub::StubRuntimeAdapter, cli::image::ImageArgs,
        config::roots::PodboxRoots, context::AppContext,
    };

    // A non-zero runtime child exit is forwarded verbatim as the process exit
    // code, bypassing the AppError 0-7 taxonomy. `6` deliberately overlaps
    // AppError::Build's code to prove it is NOT remapped through exit_code().
    fn child_exit_is_passed_through(injected: u8) {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let roots = PodboxRoots {
            config: root.join("config"),
            cache: root.join("cache"),
            state: root.join("state"),
            data: root.join("data"),
        };
        std::fs::create_dir_all(roots.config.join("images").join("demo").as_std_path()).unwrap();
        std::fs::write(
            roots
                .config
                .join("images")
                .join("demo")
                .join("devcontainer.json")
                .as_std_path(),
            r#"{"image":"alpine:latest"}"#,
        )
        .unwrap();

        let ctx = AppContext::for_test(roots, Arc::new(StubRuntimeAdapter::exiting(injected)));
        let args = ImageArgs {
            command: crate::cli::image::ImageCommand::Build {
                name: "demo".to_string(),
                full_rebuild: false,
                pull_policy: None,
            },
        };
        let code = super::run(&ctx, args).expect("build should succeed with a child-exit outcome");
        assert_eq!(code, injected, "child exit code must pass through verbatim");
    }

    #[test]
    fn image_build_forwards_child_exit_codes_verbatim() {
        child_exit_is_passed_through(42);
        // 6 overlaps the AppError::Build taxonomy slot; must still pass through.
        child_exit_is_passed_through(6);
    }
}
