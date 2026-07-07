use crate::{
    cli::shell::ShellArgs, context::AppContext, error::AppError,
    services::workspace_lifecycle::WorkspaceLifecycleService,
};

pub(crate) fn run(ctx: &AppContext, args: ShellArgs) -> Result<u8, AppError> {
    WorkspaceLifecycleService::new(ctx)
        .shell(args.command, super::workspace::mode(ctx, args.reconcile))
}
