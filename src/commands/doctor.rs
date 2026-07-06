use serde::Serialize;

use crate::{
    cli::doctor::DoctorArgs, context::AppContext, domain::doctor::Severity, error::AppError,
};

#[derive(Serialize)]
struct DoctorSkeleton {
    schema_version: u32,
    scope: crate::cli::doctor::DoctorScope,
    status: Severity,
    catalog_status: &'static str,
    checks: Vec<String>,
}

pub(crate) fn run(ctx: &AppContext, args: DoctorArgs) -> Result<(), AppError> {
    let report = DoctorSkeleton {
        schema_version: crate::util::schema_version(),
        scope: args.scope,
        status: Severity::Unknown,
        catalog_status: "not_implemented",
        checks: Vec::new(),
    };
    if args.json || ctx.global.json {
        ctx.ui.json(&report)
    } else if args.quiet || ctx.global.quiet {
        Ok(())
    } else {
        ctx.ui.stdout_line("doctor: not_implemented")?;
        ctx.ui.stdout_line("status: unknown")
    }
}
