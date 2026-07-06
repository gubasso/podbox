use crate::{
    cli::doctor::DoctorArgs,
    context::AppContext,
    error::AppError,
    services::doctor::{self, StdDoctorProbe},
};

pub(crate) fn run(ctx: &AppContext, args: DoctorArgs) -> Result<(), AppError> {
    let report = doctor::catalog(args.scope, &StdDoctorProbe);
    if args.json || ctx.global.json {
        ctx.ui.json(&report)?;
    } else if args.quiet || ctx.global.quiet {
        // quiet suppresses report output but preserves exit classification.
    } else {
        ctx.ui
            .stdout_line(&format!("status: {:?}", report.status).to_lowercase())?;
        for check in &report.checks {
            ctx.ui
                .stdout_line(&format!("{}: {:?}", check.id.0, check.status).to_lowercase())?;
        }
    }
    let code = doctor::exit_code(&report, ctx.config.config.defaults.doctor_strict);
    if code == crate::exit::SUCCESS {
        Ok(())
    } else {
        Err(match code {
            crate::exit::USAGE_OR_CONFIG => {
                AppError::config_syntax("doctor found configuration failures")
            }
            crate::exit::HOST_RUNTIME => AppError::HostRuntime {
                message: "doctor found host/runtime failures".to_string(),
            },
            crate::exit::VALIDATION => AppError::Validation {
                message: "doctor found strict warnings".to_string(),
            },
            _ => AppError::unexpected("doctor produced unsupported exit code"),
        })
    }
}
