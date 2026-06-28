use std::ffi::OsString;
use std::path::PathBuf;

use super::error::{ExportValidateError, ExportValidateResult};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidateArgs {
    pub project: PathBuf,
    pub profile: String,
    pub report: Option<PathBuf>,
    pub stage_output: Option<PathBuf>,
    pub pretty: bool,
}

pub fn parse(
    args: impl IntoIterator<Item = OsString>,
) -> ExportValidateResult<Option<ValidateArgs>> {
    let mut project = PathBuf::from("zircon-project.toml");
    let mut profile = None;
    let mut report = None;
    let mut stage_output = None;
    let mut pretty = false;

    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let arg_text = arg.to_str().ok_or_else(|| {
            ExportValidateError::Usage(usage(
                "zircon_export_validate expects UTF-8 command arguments",
            ))
        })?;
        match arg_text {
            "-h" | "--help" => return Ok(None),
            "--project" => project = next_path(&mut args, "--project")?,
            "--profile" => profile = Some(next_string(&mut args, "--profile")?),
            "--report" => report = Some(next_path(&mut args, "--report")?),
            "--stage-output" => stage_output = Some(next_path(&mut args, "--stage-output")?),
            "--pretty" => pretty = true,
            unknown => {
                return Err(ExportValidateError::Usage(usage(&format!(
                    "unknown argument {unknown}"
                ))))
            }
        }
    }

    let profile = profile.ok_or_else(|| ExportValidateError::Usage(usage("missing --profile")))?;
    Ok(Some(ValidateArgs {
        project,
        profile,
        report,
        stage_output,
        pretty,
    }))
}

pub fn usage(message: &str) -> String {
    format!(
        "{message}\nusage: zircon_export_validate --profile <name> [--project <zircon-project.toml>] [--report <path>] [--stage-output <dir>] [--pretty]"
    )
}

fn next_string(
    args: &mut impl Iterator<Item = OsString>,
    flag: &'static str,
) -> ExportValidateResult<String> {
    args.next()
        .ok_or_else(|| ExportValidateError::Usage(usage(&format!("missing value for {flag}"))))?
        .into_string()
        .map_err(|_| ExportValidateError::Usage(usage(&format!("{flag} value must be UTF-8"))))
}

fn next_path(
    args: &mut impl Iterator<Item = OsString>,
    flag: &'static str,
) -> ExportValidateResult<PathBuf> {
    Ok(PathBuf::from(next_string(args, flag)?))
}
