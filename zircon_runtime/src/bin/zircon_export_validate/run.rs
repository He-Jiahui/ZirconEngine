use std::ffi::OsString;
use std::fs::{self, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use zircon_runtime::asset::project::ProjectManifest;
use zircon_runtime::plugin::{ExportBuildPlan, ExportValidateReport};

use super::args::{parse, usage};
use super::error::{ExportValidateError, ExportValidateResult};

pub fn run(args: impl IntoIterator<Item = OsString>) -> ExportValidateResult<ExitCode> {
    let Some(args) = parse(args)? else {
        println!("{}", usage("zircon export validate report generator"));
        return Ok(ExitCode::SUCCESS);
    };
    let write_stdout = should_write_stdout(&args);

    let project_manifest = args.project.display().to_string();
    let stage_output = args
        .stage_output
        .as_ref()
        .map(|path| path.display().to_string());
    let mut contents_artifact = None;
    let report = match ProjectManifest::load(&args.project) {
        Ok(manifest) => match ExportBuildPlan::from_project_manifest(&manifest, &args.profile) {
            Ok(plan) => {
                let mut report =
                    ExportValidateReport::from_build_plan(project_manifest, stage_output, &plan);
                if let Some(artifact_path) = &args.contents_artifact {
                    let artifact =
                        ExportValidateReport::generated_contents_artifact_json(&plan, args.pretty)
                            .map_err(|source| ExportValidateError::EncodeReport { source })?;
                    report.record_generated_contents_artifact(
                        recorded_contents_artifact_path(artifact_path)?,
                        artifact.len() as u64,
                        ExportValidateReport::sha256_digest(artifact.as_bytes()),
                    );
                    contents_artifact = Some((artifact_path.clone(), artifact));
                }
                report
            }
            Err(error) => ExportValidateReport::fatal_error(
                project_manifest,
                args.profile,
                stage_output,
                false,
                format!("failed to validate export profile: {error}"),
            ),
        },
        Err(error) => ExportValidateReport::fatal_error(
            project_manifest,
            args.profile,
            stage_output,
            false,
            format!("failed to load project manifest: {error}"),
        ),
    };

    let json = if args.pretty {
        serde_json::to_string_pretty(&report)
    } else {
        serde_json::to_string(&report)
    }
    .map_err(|source| ExportValidateError::EncodeReport { source })?;

    write_outputs(
        args.report.as_deref().map(|path| (path, json.as_str())),
        contents_artifact
            .as_ref()
            .map(|(path, contents)| (path.as_path(), contents.as_str())),
    )?;

    if write_stdout {
        println!("{json}");
    }
    if report.fatal {
        Ok(ExitCode::from(2))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

fn should_write_stdout(args: &super::args::ValidateArgs) -> bool {
    args.report.is_none() || args.stdout
}

fn recorded_contents_artifact_path(path: &Path) -> ExportValidateResult<String> {
    std::path::absolute(path)
        .map(|absolute| absolute.display().to_string())
        .map_err(|source| ExportValidateError::ResolveContentsArtifactPath {
            path: path.to_path_buf(),
            source,
        })
}

fn write_outputs(
    report: Option<(&Path, &str)>,
    artifact: Option<(&Path, &str)>,
) -> ExportValidateResult<()> {
    let mut report_output = report
        .map(|(path, contents)| open_output(path, contents, OutputKind::Report))
        .transpose()?;
    let mut artifact_output = artifact
        .map(|(path, contents)| open_output(path, contents, OutputKind::ContentsArtifact))
        .transpose()?;

    if let (Some(report), Some(artifact)) = (&report_output, &artifact_output) {
        if report.handle == artifact.handle {
            return Err(ExportValidateError::OutputPathsAlias {
                report: report.path.clone(),
                artifact: artifact.path.clone(),
            });
        }
    }

    // Both identities are frozen before either file is truncated. A path replaced after this
    // point cannot redirect the report write onto the already-open artifact (or vice versa).
    if let Some(output) = artifact_output.as_mut() {
        output.write()?;
    }
    if let Some(output) = report_output.as_mut() {
        output.write()?;
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum OutputKind {
    Report,
    ContentsArtifact,
}

struct OpenOutput<'a> {
    path: PathBuf,
    contents: &'a str,
    kind: OutputKind,
    handle: same_file::Handle,
}

impl OpenOutput<'_> {
    fn write(&mut self) -> ExportValidateResult<()> {
        let file = self.handle.as_file_mut();
        file.set_len(0)
            .and_then(|()| file.seek(SeekFrom::Start(0)).map(|_| ()))
            .and_then(|()| file.write_all(self.contents.as_bytes()))
            .and_then(|()| file.flush())
            .map_err(|source| self.kind.write_error(self.path.clone(), source))
    }
}

fn open_output<'a>(
    path: &Path,
    contents: &'a str,
    kind: OutputKind,
) -> ExportValidateResult<OpenOutput<'a>> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .map_err(|source| kind.create_directory_error(parent.to_path_buf(), source))?;
    }
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(path)
        .map_err(|source| kind.write_error(path.to_path_buf(), source))?;
    let handle = same_file::Handle::from_file(file)
        .map_err(|source| kind.write_error(path.to_path_buf(), source))?;
    Ok(OpenOutput {
        path: path.to_path_buf(),
        contents,
        kind,
        handle,
    })
}

impl OutputKind {
    fn create_directory_error(self, path: PathBuf, source: std::io::Error) -> ExportValidateError {
        match self {
            Self::Report => ExportValidateError::CreateReportDirectory { path, source },
            Self::ContentsArtifact => {
                ExportValidateError::CreateContentsArtifactDirectory { path, source }
            }
        }
    }

    fn write_error(self, path: PathBuf, source: std::io::Error) -> ExportValidateError {
        match self {
            Self::Report => ExportValidateError::WriteReport { path, source },
            Self::ContentsArtifact => ExportValidateError::WriteContentsArtifact { path, source },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use zircon_runtime::asset::{AssetUri, ProjectManifest};
    use zircon_runtime::core::framework::platform::RuntimeTargetMode;
    use zircon_runtime::core::framework::project::{
        ExportPackagingStrategy, ExportProfile, ExportTargetPlatform, RuntimeProfileId,
    };

    use super::super::args::ValidateArgs;
    use super::{
        recorded_contents_artifact_path, run, should_write_stdout, write_outputs,
        ExportValidateReport,
    };

    #[test]
    fn export_validate_closeout_stdout_policy_covers_report_and_explicit_stdout() {
        assert!(should_write_stdout(&validate_args(None, false)));
        assert!(!should_write_stdout(&validate_args(
            Some(PathBuf::from("out/report.json")),
            false,
        )));
        assert!(should_write_stdout(&validate_args(
            Some(PathBuf::from("out/report.json")),
            true,
        )));
    }

    #[test]
    fn export_validate_closeout_records_relative_contents_artifact_as_absolute() {
        let relative = PathBuf::from("out/generated-contents.json");
        let recorded = PathBuf::from(
            recorded_contents_artifact_path(&relative)
                .expect("relative contents artifact path should resolve"),
        );

        assert!(recorded.is_absolute());
        assert_eq!(
            recorded,
            std::path::absolute(relative).expect("expected path should resolve")
        );
    }

    #[test]
    fn export_validate_closeout_project_load_failure_retains_profile_and_exit_code() {
        let root = unique_temp_dir("fatal-report");
        let missing_project = root.join("missing-zircon-project.toml");
        let report_path = root.join("out").join("report.json");

        let exit_code = run([
            OsString::from("--project"),
            missing_project.into_os_string(),
            OsString::from("--profile"),
            OsString::from("client"),
            OsString::from("--report"),
            report_path.clone().into_os_string(),
        ])
        .expect("manifest load failures should be encoded into the report");

        assert_eq!(exit_code, std::process::ExitCode::from(2));
        let report = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(&report_path).expect("fatal report should be written"),
        )
        .expect("fatal report should be JSON");
        assert_eq!(report["profile"], "client");
        assert_eq!(report["fatal"], true);
        assert!(report["fatal_diagnostics"][0]
            .as_str()
            .expect("fatal diagnostic should be text")
            .contains("failed to load project manifest"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn export_validate_closeout_writes_contents_artifact_and_matching_report_metadata() {
        let root = unique_temp_dir("contents-artifact");
        let project_path = root.join("zircon-project.toml");
        let report_path = root.join("out").join("report.json");
        let artifact_path = root.join("out").join("generated-contents.json");
        source_template_manifest()
            .save(&project_path)
            .expect("project manifest should be written");

        let exit_code = run([
            OsString::from("--project"),
            project_path.into_os_string(),
            OsString::from("--profile"),
            OsString::from("server"),
            OsString::from("--report"),
            report_path.clone().into_os_string(),
            OsString::from("--contents-artifact"),
            artifact_path.clone().into_os_string(),
        ])
        .expect("valid project export should produce both outputs");

        assert_eq!(exit_code, std::process::ExitCode::SUCCESS);
        let report = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(&report_path).expect("report should be written"),
        )
        .expect("report should be JSON");
        let artifact = fs::read_to_string(&artifact_path).expect("artifact should be written");
        let artifact_json =
            serde_json::from_str::<serde_json::Value>(&artifact).expect("artifact should be JSON");

        assert_eq!(report["schema_version"], 2);
        assert_eq!(artifact_json["schema_version"], 1);
        assert_eq!(
            report["generated_contents_artifact_path"],
            artifact_path.display().to_string()
        );
        assert_eq!(
            report["generated_contents_artifact_byte_length"],
            artifact.len() as u64
        );
        assert_eq!(
            report["generated_contents_artifact_digest"],
            ExportValidateReport::sha256_digest(artifact.as_bytes())
        );
        assert!(artifact_json["generated_files"]
            .as_array()
            .expect("artifact should contain generated files")
            .iter()
            .any(|file| file["path"] == "src/main.rs" && file["contents"].is_string()));
        assert!(report["plan_summary"]["generated_files"]
            .as_array()
            .expect("report should summarize generated files")
            .iter()
            .all(|file| file.get("contents").is_none()));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn export_validate_closeout_rejects_check_use_hardlink_replacement_without_truncation() {
        let root = unique_temp_dir("output-identity-replacement");
        let report_path = root.join("report.json");
        let artifact_path = root.join("artifact.json");
        fs::write(&report_path, "preserve-existing-output").expect("report fixture");
        fs::write(&artifact_path, "distinct-at-argument-check").expect("artifact fixture");
        fs::remove_file(&artifact_path).expect("replace artifact fixture");
        fs::hard_link(&report_path, &artifact_path).expect("replacement hard link");

        let error = write_outputs(
            Some((&report_path, "new-report")),
            Some((&artifact_path, "new-artifact")),
        )
        .expect_err("opened output identities must reject the replacement alias");

        assert!(matches!(
            error,
            super::ExportValidateError::OutputPathsAlias { .. }
        ));
        assert_eq!(
            fs::read_to_string(&report_path).expect("report remains readable"),
            "preserve-existing-output"
        );
        assert_eq!(
            fs::read_to_string(&artifact_path).expect("artifact alias remains readable"),
            "preserve-existing-output"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn export_validate_closeout_rejects_broken_symlink_and_parent_component_aliases_at_open() {
        use std::os::unix::fs::symlink;

        let root = unique_temp_dir("symlink-output-identity");
        let real = root.join("real");
        let nested = real.join("nested");
        fs::create_dir_all(&nested).expect("real output directories");
        let artifact_path = real.join("artifact.json");
        let broken_link = root.join("broken-report.json");
        symlink(&artifact_path, &broken_link).expect("broken output symlink");
        let error = write_outputs(
            Some((&broken_link, "report")),
            Some((&artifact_path, "artifact")),
        )
        .expect_err("broken symlink target must be compared after opening");
        assert!(matches!(
            error,
            super::ExportValidateError::OutputPathsAlias { .. }
        ));

        fs::remove_file(&broken_link).expect("remove broken symlink");
        let directory_link = root.join("linked");
        symlink(&nested, &directory_link).expect("directory symlink");
        let parent_component_path = directory_link.join("..").join("artifact.json");
        let error = write_outputs(
            Some((&parent_component_path, "report")),
            Some((&artifact_path, "artifact")),
        )
        .expect_err("symlink parent traversal must use opened file identity");
        assert!(matches!(
            error,
            super::ExportValidateError::OutputPathsAlias { .. }
        ));
        let _ = fs::remove_dir_all(root);
    }

    fn source_template_manifest() -> ProjectManifest {
        let mut manifest = ProjectManifest::new(
            "Export Validate CLI Artifact",
            AssetUri::parse("res://scenes/main.zscene").expect("scene URI should parse"),
            1,
        );
        manifest.export_profiles = vec![ExportProfile::new(
            "server",
            RuntimeTargetMode::ServerRuntime,
            ExportTargetPlatform::Windows,
            RuntimeProfileId::Server,
        )
        .with_strategy(ExportPackagingStrategy::SourceTemplate)];
        manifest
    }

    fn validate_args(report: Option<PathBuf>, stdout: bool) -> ValidateArgs {
        ValidateArgs {
            project: PathBuf::from("zircon-project.toml"),
            profile: "client".to_string(),
            report,
            contents_artifact: None,
            stage_output: None,
            pretty: false,
            stdout,
        }
    }

    fn unique_temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should follow the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zircon-export-validate-{label}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("temporary test directory should be created");
        root
    }
}
