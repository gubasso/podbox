use std::fs;

use serde::Serialize;

use crate::{
    cli::init::{InitArgs, StarterArg},
    context::AppContext,
    domain::doctor::DoctorReport,
    error::AppError,
    services::doctor::{self, StdDoctorProbe},
    util::seed::{self, Starter},
};

#[derive(Clone, Debug, Serialize)]
struct InitReport {
    schema_version: u32,
    starter: &'static str,
    manifest: String,
    dry_run: bool,
    actions: Vec<InitAction>,
    doctor: Option<DoctorReport>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct InitAction {
    path: String,
    action: &'static str,
}

pub(crate) fn run(ctx: &AppContext, args: InitArgs) -> Result<(), AppError> {
    let starter = match args.starter {
        StarterArg::Minimal => Starter::Minimal,
        StarterArg::None => Starter::None,
    };
    let manifest = ctx
        .global
        .manifest
        .clone()
        .unwrap_or_else(|| match starter {
            Starter::Minimal => "minimal".to_string(),
            Starter::None => ctx.config.config.defaults.manifest.clone(),
        });
    let dry_run = args.dry_run || ctx.global.dry_run;
    let force = ctx.global.yes;
    let actions = plan_and_apply(ctx, starter, &manifest, dry_run, force)?;
    // Post-step doctor runs after a real init (skipped for --dry-run). Its
    // diagnostics are *reported*, not fatal, per the report-vs-fail policy.
    let doctor = if dry_run {
        None
    } else {
        Some(doctor::catalog(
            crate::cli::doctor::DoctorScope::All,
            &StdDoctorProbe,
        ))
    };
    let report = InitReport {
        schema_version: crate::util::schema_version(),
        starter: match starter {
            Starter::Minimal => "minimal",
            Starter::None => "none",
        },
        manifest,
        dry_run,
        actions,
        doctor,
    };
    if args.json || ctx.global.json {
        ctx.ui.json(&report)
    } else {
        ctx.ui
            .stdout_line(&format!("starter: {}", report.starter))?;
        for action in &report.actions {
            ctx.ui
                .stdout_line(&format!("{}: {}", action.action, action.path))?;
        }
        if let Some(doctor) = &report.doctor {
            ctx.ui
                .stdout_line(&format!("doctor: {:?}", doctor.status).to_lowercase())?;
            for check in &doctor.checks {
                ctx.ui
                    .stdout_line(&format!("  {}: {:?}", check.id.0, check.status).to_lowercase())?;
                if !check.message.is_empty() {
                    ctx.ui
                        .stdout_line(&format!("    message: {}", check.message))?;
                }
                if let Some(remediation) = &check.remediation {
                    ctx.ui
                        .stdout_line(&format!("    remediation: {remediation}"))?;
                }
                if let Some(docs) = &check.docs {
                    ctx.ui.stdout_line(&format!("    docs: {docs}"))?;
                }
            }
        }
        Ok(())
    }
}

fn plan_and_apply(
    ctx: &AppContext,
    starter: Starter,
    manifest: &str,
    dry_run: bool,
    force: bool,
) -> Result<Vec<InitAction>, AppError> {
    let mut actions = Vec::new();
    for dir in seed::dirs() {
        let path = ctx.roots.config.join(dir);
        actions.push(InitAction {
            path: path.to_string(),
            action: if path.exists() {
                "exists"
            } else {
                "create-dir"
            },
        });
        if !dry_run {
            fs::create_dir_all(&path).map_err(|err| AppError::io(path.clone(), err))?;
        }
    }
    for file in seed::files(starter, manifest) {
        let path = seed::target(&ctx.roots.config, file.relative_path);
        let exists = path.exists();
        let should_write = !exists || (force && !file.leaf_protected);
        let action = match (exists, should_write, file.leaf_protected) {
            (false, true, _) => "create",
            (true, true, _) => "replace",
            (true, false, true) => "preserve-leaf",
            (true, false, false) => "preserve",
            _ => "preserve",
        };
        actions.push(InitAction {
            path: path.to_string(),
            action,
        });
        if should_write && !dry_run {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|err| AppError::io(parent.to_path_buf(), err))?;
            }
            fs::write(&path, file.content.as_bytes()).map_err(|err| AppError::io(path, err))?;
        }
    }
    Ok(actions)
}
