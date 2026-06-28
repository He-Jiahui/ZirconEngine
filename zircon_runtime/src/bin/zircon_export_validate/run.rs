use std::ffi::OsString;
use std::fs;
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

    let project_manifest = args.project.display().to_string();
    let stage_output = args
        .stage_output
        .as_ref()
        .map(|path| path.display().to_string());
    let report = match ProjectManifest::load(&args.project) {
        Ok(manifest) => match ExportBuildPlan::from_project_manifest(&manifest, &args.profile) {
            Ok(plan) => {
                ExportValidateReport::from_build_plan(project_manifest, stage_output, &plan)
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

    if let Some(report_path) = &args.report {
        if let Some(parent) = report_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|source| {
                    ExportValidateError::CreateReportDirectory {
                        path: parent.to_path_buf(),
                        source,
                    }
                })?;
            }
        }
        fs::write(report_path, &json).map_err(|source| ExportValidateError::WriteReport {
            path: report_path.clone(),
            source,
        })?;
    }

    println!("{json}");
    if report.fatal {
        Ok(ExitCode::from(2))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}
