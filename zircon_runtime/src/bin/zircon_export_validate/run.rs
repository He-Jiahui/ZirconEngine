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
        Ok(manifest) => match ExportBuildPlan::from_project_manifest_with_plugin_root(
            &manifest,
            &args.profile,
            args.project
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join("zircon_plugins"),
        ) {
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
#[path = "tests/run.rs"]
mod tests;
