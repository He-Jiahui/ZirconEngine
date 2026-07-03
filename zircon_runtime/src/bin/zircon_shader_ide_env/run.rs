use std::ffi::OsString;
use std::path::Path;
use std::process::ExitCode;

use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::core::framework::render::ShaderIdePreviewVariant;
use zircon_runtime::graphics::{write_shader_ide_env_for_project, ShaderIdeEnvReport};

use super::args::{parse, usage};

pub fn run(args: impl IntoIterator<Item = OsString>) -> Result<ExitCode, String> {
    let Some(args) = parse(args)? else {
        println!("{}", usage("zircon shader IDE environment generator"));
        return Ok(ExitCode::SUCCESS);
    };
    let report = generate_shader_ide_env(
        &args.project_root,
        args.output_dir.as_deref(),
        &args.preview_variants,
    )?;
    let json = if args.pretty {
        serde_json::to_string_pretty(&report)
    } else {
        serde_json::to_string(&report)
    }
    .map_err(|error| format!("encode shader IDE env report: {error}"))?;
    println!("{json}");
    Ok(ExitCode::SUCCESS)
}

fn generate_shader_ide_env(
    project_root: &Path,
    output_dir: Option<&Path>,
    preview_variants: &[ShaderIdePreviewVariant],
) -> Result<ShaderIdeEnvReport, String> {
    let mut manager = ProjectManager::open(project_root)
        .map_err(|error| format!("open project {}: {error}", project_root.display()))?;
    manager.scan_and_import().map_err(|error| {
        format!(
            "scan shader IDE project {}: {error}",
            project_root.display()
        )
    })?;
    write_shader_ide_env_for_project(&manager, output_dir, preview_variants)
}
