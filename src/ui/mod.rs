pub(crate) mod human;
pub(crate) mod json;

use anstream::{eprintln, println};
use serde::Serialize;

use crate::error::AppError;

#[derive(Clone, Debug)]
pub(crate) struct Ui {
    color: bool,
    quiet: bool,
}

impl Ui {
    pub(crate) fn from_env(no_color: bool, quiet: bool) -> Result<Self, AppError> {
        let color = !no_color
            && std::env::var_os("NO_COLOR").is_none()
            && supports_color::on(supports_color::Stream::Stdout).is_some();
        let _style = anstyle::Style::new();
        Ok(Self { color, quiet })
    }

    pub(crate) fn stdout_line(&self, line: &str) -> Result<(), AppError> {
        if !self.quiet {
            if self.color {
                use owo_colors::OwoColorize;
                println!("{}", line.green());
            } else {
                println!("{line}");
            }
        }
        Ok(())
    }

    pub(crate) fn stderr_line(&self, line: &str) -> Result<(), AppError> {
        if !self.quiet {
            eprintln!("{line}");
        }
        Ok(())
    }

    pub(crate) fn json<T: Serialize>(&self, value: &T) -> Result<(), AppError> {
        let rendered = serde_json::to_string_pretty(value)
            .map_err(|err| AppError::unexpected(format!("failed to render json: {err}")))?;
        println!("{rendered}");
        Ok(())
    }
}
