use crate::{
    cli::status::StatusArgs,
    context::AppContext,
    error::AppError,
    services::workspace_lifecycle::{WorkspaceLifecycleService, WorkspaceStatusReport},
};

pub(crate) fn run(ctx: &AppContext, args: StatusArgs) -> Result<(), AppError> {
    let report = WorkspaceLifecycleService::new(ctx).status()?;
    render(ctx, args.json || ctx.global.json, &report)
}

pub(crate) fn render(
    ctx: &AppContext,
    json: bool,
    report: &WorkspaceStatusReport,
) -> Result<(), AppError> {
    if json {
        ctx.ui.json(report)
    } else {
        ctx.ui
            .stdout_line(&format!("workspace: {}", report.identity.label.0))?;
        ctx.ui
            .stdout_line(&format!("state: {:?}", report.state).to_lowercase())?;
        ctx.ui
            .stdout_line(&format!("manifest: {}", report.manifest))?;
        ctx.ui
            .stdout_line(&format!("config: {}", report.config_sources.join(", ")))?;
        ctx.ui
            .stdout_line(&format!("runtime: {}", report.runtime_profile))?;
        ctx.ui
            .stdout_line(&format!("hardening: {}", report.hardening_posture))?;
        ctx.ui
            .stdout_line(&format!("network: {}", report.network_policy))?;
        ctx.ui
            .stdout_line(&format!("credentials: {}", report.credential_policy))?;
        if let Some(image) = &report.image {
            ctx.ui.stdout_line(&format!("image: {image}"))?;
        }
        ctx.ui.stdout_line(&format!(
            "image_fresh: {}",
            report.image_freshness.as_deref().unwrap_or("unknown")
        ))?;
        if report.drift.drifted {
            let causes = report
                .drift
                .causes
                .iter()
                .map(|cause| cause.0.as_str())
                .collect::<Vec<_>>()
                .join("; ");
            ctx.ui.stdout_line(&format!("drift: {causes}"))?;
        } else {
            ctx.ui.stdout_line("drift: clean")?;
        }
        if report.relaxations.is_empty() {
            ctx.ui.stdout_line("relaxations: []")
        } else {
            ctx.ui
                .stdout_line(&format!("relaxations: {}", report.relaxations.join(", ")))
        }
    }
}
