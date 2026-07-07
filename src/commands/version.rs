use serde::Serialize;

use crate::{cli::version::VersionArgs, error::AppError, ui, ui::Ui};

#[derive(Serialize)]
struct VersionReport {
    version: &'static str,
    build_sha: &'static str,
    build_date: &'static str,
}

pub(crate) fn run(ui: &Ui, global_json: bool, args: VersionArgs) -> Result<(), AppError> {
    let report = VersionReport {
        version: env!("CARGO_PKG_VERSION"),
        build_sha: crate::cli::version::BUILD_SHA,
        build_date: crate::cli::version::BUILD_DATE,
    };
    if args.json || global_json {
        ui.json(&crate::ui::json::versioned(report))
    } else {
        for line in ui::human::lines(&[
            ("version", report.version.to_string()),
            ("build_sha", report.build_sha.to_string()),
            ("build_date", report.build_date.to_string()),
        ]) {
            ui.stdout_line(&line)?;
        }
        Ok(())
    }
}
