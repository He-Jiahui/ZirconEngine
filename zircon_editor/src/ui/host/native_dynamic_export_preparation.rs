use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use zircon_runtime::{
    ExportBuildPlan, NativePluginLoadReport, PluginModuleKind, PluginPackageManifest,
};

use super::editor_manager_plugins_export::EditorExportCargoInvocation;

#[derive(Debug)]
pub(super) struct NativeDynamicPreparation {
    pub(super) plugin_root: PathBuf,
    pub(super) build_root: PathBuf,
    pub(super) cargo_invocations: Vec<EditorExportCargoInvocation>,
    pub(super) diagnostics: Vec<String>,
}

pub(super) fn prepare_native_dynamic_packages(
    output_root: &Path,
    plan: &ExportBuildPlan,
    native_report: &NativePluginLoadReport,
) -> Result<NativeDynamicPreparation, String> {
    let staging_root = output_root.join(".native-dynamic-staging");
    let build_root = output_root.join(".native-dynamic-build");
    if staging_root.exists() {
        fs::remove_dir_all(&staging_root).map_err(|error| {
            format!(
                "failed to clear native dynamic staging root {}: {error}",
                staging_root.display()
            )
        })?;
    }
    if build_root.exists() {
        fs::remove_dir_all(&build_root).map_err(|error| {
            format!(
                "failed to clear native dynamic build root {}: {error}",
                build_root.display()
            )
        })?;
    }
    fs::create_dir_all(&staging_root).map_err(|error| {
        format!(
            "failed to create native dynamic staging root {}: {error}",
            staging_root.display()
        )
    })?;

    let mut cargo_invocations = Vec::new();
    let mut diagnostics = Vec::new();
    for package_id in &plan.native_dynamic_packages {
        let Some(candidate) = native_report
            .discovered
            .iter()
            .find(|candidate| &candidate.plugin_id == package_id)
        else {
            diagnostics.push(format!(
                "native dynamic package {package_id} has no discovered package manifest for artifact staging"
            ));
            continue;
        };
        let Some(package_root) = candidate.manifest_path.parent() else {
            diagnostics.push(format!(
                "native dynamic package {package_id} manifest has no parent directory"
            ));
            continue;
        };
        let staged_package = staging_root.join(package_id);
        stage_native_package_static_files(package_root, &staged_package).map_err(|error| {
            format!(
                "failed to stage native dynamic package {package_id} into {}: {error}",
                staged_package.display()
            )
        })?;
        let artifact_count =
            copy_native_artifacts(&package_root.join("native"), &staged_package.join("native"))
                .map_err(|error| {
                    format!("failed to stage native dynamic artifacts for {package_id}: {error}")
                })?;
        if artifact_count > 0 {
            diagnostics.push(format!(
                "native dynamic package {package_id} staged {artifact_count} existing native artifact(s)"
            ));
            continue;
        }

        let native_manifest_path = package_root.join("native/Cargo.toml");
        if !native_manifest_path.exists() {
            continue;
        }
        let Some(crate_name) = module_crate(&candidate.package_manifest, PluginModuleKind::Runtime)
            .or_else(|| module_crate(&candidate.package_manifest, PluginModuleKind::Editor))
        else {
            diagnostics.push(format!(
                "native dynamic package {package_id} has native Cargo.toml but no runtime or editor crate name"
            ));
            continue;
        };
        let build_target = build_root.join(sanitize_path_component(package_id));
        let invocation = invoke_native_cargo_build(&native_manifest_path, &build_target)?;
        if invocation.success {
            let artifact = build_target
                .join("debug")
                .join(dynamic_library_file_name(&crate_name));
            if artifact.exists() {
                let staged_native = staged_package.join("native");
                fs::create_dir_all(&staged_native).map_err(|error| {
                    format!(
                        "failed to create staged native artifact directory {}: {error}",
                        staged_native.display()
                    )
                })?;
                fs::copy(&artifact, staged_native.join(artifact.file_name().unwrap())).map_err(
                    |error| {
                        format!(
                            "failed to stage built native artifact {}: {error}",
                            artifact.display()
                        )
                    },
                )?;
            } else {
                diagnostics.push(format!(
                    "native dynamic package {package_id} cargo build succeeded but artifact was missing: {}",
                    artifact.display()
                ));
            }
        }
        cargo_invocations.push(invocation);
    }

    Ok(NativeDynamicPreparation {
        plugin_root: staging_root,
        build_root,
        cargo_invocations,
        diagnostics,
    })
}

pub(super) fn cleanup_native_dynamic_preparation(
    preparation: &NativeDynamicPreparation,
) -> Vec<String> {
    let mut diagnostics = Vec::new();
    for root in [&preparation.plugin_root, &preparation.build_root] {
        if !root.exists() {
            continue;
        }
        if let Err(error) = fs::remove_dir_all(root) {
            diagnostics.push(format!(
                "failed to remove native dynamic temporary directory {}: {error}",
                root.display()
            ));
        }
    }
    diagnostics
}

fn invoke_native_cargo_build(
    manifest_path: &Path,
    target_dir: &Path,
) -> Result<EditorExportCargoInvocation, String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let args = vec![
        "build".to_string(),
        "--manifest-path".to_string(),
        manifest_path.display().to_string(),
        "--locked".to_string(),
        "--target-dir".to_string(),
        target_dir.display().to_string(),
    ];
    let output = Command::new(&cargo)
        .args(&args)
        .output()
        .map_err(|error| format!("failed to invoke cargo for native dynamic plugin: {error}"))?;

    let mut command = Vec::with_capacity(args.len() + 1);
    command.push(cargo);
    command.extend(args);

    Ok(EditorExportCargoInvocation {
        command,
        status_code: output.status.code(),
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

fn stage_native_package_static_files(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let Some(file_name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        if source_path.is_dir() {
            if matches!(
                file_name.as_str(),
                "assets" | "asset" | "resources" | "resource"
            ) {
                copy_dir_all(&source_path, &destination_path)?;
            }
        } else if file_name == "plugin.toml" {
            fs::copy(&source_path, &destination_path)?;
        }
    }
    Ok(())
}

fn copy_native_artifacts(source: &Path, destination: &Path) -> std::io::Result<usize> {
    let mut copied = 0;
    if !source.exists() {
        return Ok(copied);
    }
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        if source_path.is_dir() || !is_native_dynamic_artifact(&source_path) {
            continue;
        }
        let Some(file_name) = source_path.file_name() else {
            continue;
        };
        fs::copy(&source_path, destination.join(file_name))?;
        copied += 1;
    }
    Ok(copied)
}

fn copy_dir_all(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir_all(&source_path, &destination_path)?;
        } else {
            fs::copy(&source_path, &destination_path)?;
        }
    }
    Ok(())
}

fn module_crate(package: &PluginPackageManifest, kind: PluginModuleKind) -> Option<String> {
    package
        .modules
        .iter()
        .find(|module| module.kind == kind)
        .map(|module| module.crate_name.clone())
}

fn is_native_dynamic_artifact(path: &Path) -> bool {
    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return false;
    };
    matches!(
        extension.to_ascii_lowercase().as_str(),
        "dll" | "so" | "dylib" | "pdb" | "dbg" | "dsym"
    )
}

fn dynamic_library_file_name(crate_name: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{crate_name}.dll")
    } else if cfg!(target_os = "macos") {
        format!("lib{crate_name}.dylib")
    } else {
        format!("lib{crate_name}.so")
    }
}

fn sanitize_path_component(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}
