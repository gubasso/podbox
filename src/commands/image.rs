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
            names,
            all,
            full_rebuild,
            pull_policy,
        } => build(ctx, &names, all, full_rebuild, pull_policy),
        ImageCommand::List => list(ctx).map(|()| crate::exit::SUCCESS),
        ImageCommand::Inspect { name } => status(ctx, &name).map(|()| crate::exit::SUCCESS),
        ImageCommand::Status { name } => status(ctx, &name).map(|()| crate::exit::SUCCESS),
        ImageCommand::Prune { yes } => prune(ctx, yes).map(|()| crate::exit::SUCCESS),
    }
}

fn build(
    ctx: &AppContext,
    names: &[String],
    all: bool,
    full_rebuild: bool,
    pull_policy: Option<PullPolicyArg>,
) -> Result<u8, AppError> {
    let targets = if all {
        if !names.is_empty() {
            return Err(AppError::usage(
                "image build accepts either <name>... or --all, not both",
            ));
        }
        image_names(ctx)?
    } else {
        if names.is_empty() {
            return Err(AppError::usage("image build requires <name>... or --all"));
        }
        names.to_vec()
    };

    // Collect every target's outcome first, then render a single document. Under
    // `--json` this keeps stdout a single well-formed JSON document (10 §2) rather
    // than one object per target. A non-zero runtime child exit fails fast and is
    // forwarded verbatim without emitting a partial aggregate.
    let mut reports = Vec::new();
    for name in &targets {
        match build_one(ctx, name, full_rebuild, pull_policy)? {
            ImageBuildOutcome::Report(report) => reports.push(report),
            ImageBuildOutcome::ChildExit(code) => return Ok(code),
        }
    }

    if ctx.global.json {
        #[derive(serde::Serialize)]
        struct BuildReport {
            schema_version: u32,
            images: Vec<crate::services::image_build::ImageBuildReport>,
        }
        ctx.ui.json(&BuildReport {
            schema_version: crate::util::schema_version(),
            images: reports,
        })?;
    } else {
        for report in &reports {
            ctx.ui.stdout_line(&format!("image: {}", report.image))?;
            ctx.ui.stdout_line(&format!("digest: {}", report.digest))?;
            ctx.ui.stdout_line(if report.reused {
                "reused: true"
            } else {
                "reused: false"
            })?;
            // Dry-run conveys the same rebuild/reuse rationale as --json (06 §5).
            if let Some(reason) = &report.reason {
                ctx.ui.stdout_line(&format!("reason: {reason}"))?;
            }
        }
    }
    Ok(crate::exit::SUCCESS)
}

fn build_one(
    ctx: &AppContext,
    name: &str,
    full_rebuild: bool,
    pull_policy: Option<PullPolicyArg>,
) -> Result<ImageBuildOutcome, AppError> {
    let pull_policy = pull_policy
        .map(PullPolicy::from)
        .unwrap_or_else(|| PullPolicy::from_config(&ctx.config.config.images.pull_policy));
    ImageBuildService::new(ctx).build(ImageBuildInput {
        name: name.to_string(),
        full_rebuild,
        pull_policy,
        dry_run: ctx.global.dry_run,
    })
}

fn list(ctx: &AppContext) -> Result<(), AppError> {
    let names = image_names(ctx)?;
    if ctx.global.json {
        #[derive(serde::Serialize)]
        struct Report {
            schema_version: u32,
            images: Vec<String>,
        }
        ctx.ui.json(&Report {
            schema_version: crate::util::schema_version(),
            images: names,
        })
    } else {
        for name in names {
            ctx.ui.stdout_line(&name)?;
        }
        Ok(())
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

fn image_names(ctx: &AppContext) -> Result<Vec<String>, AppError> {
    let dir = ctx.roots.config.join("images");
    let mut names = Vec::new();
    if !dir.exists() {
        return Ok(names);
    }
    for entry in std::fs::read_dir(&dir).map_err(|err| AppError::io(dir.clone(), err))? {
        let entry = entry
            .map_err(|err| AppError::unexpected(format!("failed to read images dir: {err}")))?;
        if !entry.file_type().map(|ty| ty.is_dir()).unwrap_or(false) {
            continue;
        }
        if let Some(name) = entry.file_name().to_str() {
            names.push(name.to_string());
        }
    }
    names.sort();
    Ok(names)
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
                names: vec!["demo".to_string()],
                all: false,
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
