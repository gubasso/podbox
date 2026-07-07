use crate::{
    cli::workspace::{ReconcileArg, WorkspaceArgs, WorkspaceCommand},
    context::AppContext,
    error::AppError,
    services::workspace_lifecycle::{ReconcileMode, WorkspaceLifecycleService},
};

pub(crate) fn run(ctx: &AppContext, args: WorkspaceArgs) -> Result<u8, AppError> {
    let service = WorkspaceLifecycleService::new(ctx);
    match args.command {
        WorkspaceCommand::Up => render_reconcile(ctx, service.up()?).map(|()| crate::exit::SUCCESS),
        WorkspaceCommand::Reconcile { dry_run, yes } => render_reconcile(
            ctx,
            service.reconcile(dry_run || ctx.global.dry_run, yes || ctx.global.yes)?,
        )
        .map(|()| crate::exit::SUCCESS),
        WorkspaceCommand::Shell { reconcile, command } => {
            service.shell(command, mode(ctx, reconcile))
        }
        WorkspaceCommand::Exec { command } => service.exec(command),
        WorkspaceCommand::Down { dry_run, yes } => render_reconcile(
            ctx,
            service.down(dry_run || ctx.global.dry_run, yes || ctx.global.yes)?,
        )
        .map(|()| crate::exit::SUCCESS),
        WorkspaceCommand::Status => {
            let report = service.status()?;
            crate::commands::status::render(ctx, ctx.global.json, &report)?;
            Ok(crate::exit::SUCCESS)
        }
    }
}

pub(crate) fn mode(ctx: &AppContext, arg: Option<ReconcileArg>) -> ReconcileMode {
    match arg {
        Some(ReconcileArg::Always) => ReconcileMode::Always,
        Some(ReconcileArg::Never) => ReconcileMode::Never,
        Some(ReconcileArg::Auto) => ReconcileMode::Auto,
        None => match ctx.config.config.defaults.reconcile.as_str() {
            "always" => ReconcileMode::Always,
            "never" => ReconcileMode::Never,
            _ => ReconcileMode::Auto,
        },
    }
}

fn render_reconcile(
    ctx: &AppContext,
    report: crate::services::workspace_lifecycle::ReconcileReport,
) -> Result<(), AppError> {
    if ctx.global.json {
        ctx.ui.json(&report)
    } else {
        ctx.ui
            .stdout_line(&format!("workspace: {}", report.identity.label.0))?;
        ctx.ui
            .stdout_line(&format!("state: {:?}", report.state).to_lowercase())?;
        ctx.ui
            .stdout_line(&format!("steps: {}", report.steps.join(", ")))
    }
}
