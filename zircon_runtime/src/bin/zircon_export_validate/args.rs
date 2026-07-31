use std::ffi::OsString;
use std::path::{Path, PathBuf};

use super::error::{ExportValidateError, ExportValidateResult};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidateArgs {
    pub project: PathBuf,
    pub profile: String,
    pub report: Option<PathBuf>,
    pub contents_artifact: Option<PathBuf>,
    pub stage_output: Option<PathBuf>,
    pub pretty: bool,
    pub stdout: bool,
}

pub fn parse(
    args: impl IntoIterator<Item = OsString>,
) -> ExportValidateResult<Option<ValidateArgs>> {
    let mut project = PathBuf::from("zircon-project.toml");
    let mut profile = None;
    let mut report = None;
    let mut contents_artifact = None;
    let mut stage_output = None;
    let mut pretty = false;
    let mut stdout = false;

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
            "--contents-artifact" => {
                contents_artifact = Some(next_path(&mut args, "--contents-artifact")?)
            }
            "--stage-output" => stage_output = Some(next_path(&mut args, "--stage-output")?),
            "--pretty" => pretty = true,
            "--stdout" => stdout = true,
            unknown => {
                return Err(ExportValidateError::Usage(usage(&format!(
                    "unknown argument {unknown}"
                ))));
            }
        }
    }

    let profile = profile.ok_or_else(|| ExportValidateError::Usage(usage("missing --profile")))?;
    if report
        .as_deref()
        .zip(contents_artifact.as_deref())
        .is_some_and(|(report, artifact)| output_paths_alias(report, artifact))
    {
        return Err(ExportValidateError::Usage(usage(
            "--report and --contents-artifact must use different paths",
        )));
    }
    Ok(Some(ValidateArgs {
        project,
        profile,
        report,
        contents_artifact,
        stage_output,
        pretty,
        stdout,
    }))
}

pub fn usage(message: &str) -> String {
    format!(
        "{message}\nusage: zircon_export_validate --profile <name> [--project <zircon-project.toml>] [--report <path>] [--contents-artifact <path>] [--stage-output <dir>] [--pretty] [--stdout]"
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

fn output_paths_alias(report: &Path, artifact: &Path) -> bool {
    same_file::is_same_file(report, artifact).unwrap_or(false)
        || output_paths_equal(report, artifact)
}

#[cfg(windows)]
fn output_paths_equal(left: &Path, right: &Path) -> bool {
    left.to_string_lossy().to_lowercase() == right.to_string_lossy().to_lowercase()
}

#[cfg(not(windows))]
fn output_paths_equal(left: &Path, right: &Path) -> bool {
    left == right
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::parse;

    #[test]
    fn export_validate_closeout_accepts_explicit_stdout_and_contents_artifact() {
        let args = parse([
            OsString::from("--profile"),
            OsString::from("client"),
            OsString::from("--report"),
            OsString::from("out/report.json"),
            OsString::from("--contents-artifact"),
            OsString::from("out/contents.json"),
            OsString::from("--stdout"),
        ])
        .expect("arguments should parse")
        .expect("help should not be requested");

        assert_eq!(args.report, Some(PathBuf::from("out/report.json")));
        assert_eq!(
            args.contents_artifact,
            Some(PathBuf::from("out/contents.json"))
        );
        assert!(args.stdout);
    }

    #[test]
    fn export_validate_closeout_rejects_identical_report_and_contents_artifact_paths() {
        let error = parse([
            OsString::from("--profile"),
            OsString::from("client"),
            OsString::from("--report"),
            OsString::from("out/report.json"),
            OsString::from("--contents-artifact"),
            OsString::from("out/report.json"),
        ])
        .expect_err("the report must not overwrite the contents artifact");

        assert!(error
            .to_string()
            .contains("--report and --contents-artifact must use different paths"));
    }

    #[test]
    fn export_validate_closeout_defers_parent_component_identity_to_output_open() {
        let args = parse([
            OsString::from("--profile"),
            OsString::from("client"),
            OsString::from("--report"),
            OsString::from("out/report.json"),
            OsString::from("--contents-artifact"),
            OsString::from("out/nested/../report.json"),
        ])
        .expect("argument parsing must not guess across symlinked parent components")
        .expect("help should not be requested");

        assert_eq!(args.report, Some(PathBuf::from("out/report.json")));
        assert_eq!(
            args.contents_artifact,
            Some(PathBuf::from("out/nested/../report.json"))
        );
    }

    #[cfg(windows)]
    #[test]
    fn export_validate_closeout_rejects_windows_case_aliased_output_paths() {
        let error = parse([
            OsString::from("--profile"),
            OsString::from("client"),
            OsString::from("--report"),
            OsString::from("out/report.json"),
            OsString::from("--contents-artifact"),
            OsString::from("out/REPORT.json"),
        ])
        .expect_err("Windows case aliases must not address both outputs");

        assert!(error
            .to_string()
            .contains("--report and --contents-artifact must use different paths"));
    }

    #[test]
    fn export_validate_closeout_rejects_hard_linked_output_paths() {
        let root = unique_temp_dir("hard-link-alias");
        let report = root.join("report.json");
        let artifact = root.join("contents.json");
        fs::write(&report, "existing output").expect("report fixture should be written");
        fs::hard_link(&report, &artifact).expect("hard-link fixture should be created");

        let error = parse([
            OsString::from("--profile"),
            OsString::from("client"),
            OsString::from("--report"),
            report.into_os_string(),
            OsString::from("--contents-artifact"),
            artifact.into_os_string(),
        ])
        .expect_err("hard links must not address both outputs");

        assert!(error
            .to_string()
            .contains("--report and --contents-artifact must use different paths"));
        let _ = fs::remove_dir_all(root);
    }

    fn unique_temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should follow the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zircon-export-validate-args-{label}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("temporary test directory should be created");
        root
    }
}
