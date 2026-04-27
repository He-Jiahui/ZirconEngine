use std::fs;
use std::path::{Path, PathBuf};

use crate::PluginPackageManifest;

use super::{ExportBuildPlan, ExportMaterializeReport};

impl ExportBuildPlan {
    pub fn write_generated_files(
        &self,
        root: impl AsRef<Path>,
    ) -> Result<Vec<PathBuf>, std::io::Error> {
        let root = root.as_ref();
        let mut written = Vec::new();
        for file in &self.generated_files {
            let path = root.join(&file.path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&path, &file.contents)?;
            written.push(path);
        }
        Ok(written)
    }

    pub fn materialize(
        &self,
        output_root: impl AsRef<Path>,
    ) -> Result<ExportMaterializeReport, std::io::Error> {
        let generated_files = self.write_generated_files(output_root)?;
        Ok(ExportMaterializeReport {
            generated_files,
            copied_packages: Vec::new(),
            diagnostics: Vec::new(),
        })
    }

    pub fn materialize_with_native_packages(
        &self,
        plugin_root: impl AsRef<Path>,
        output_root: impl AsRef<Path>,
    ) -> Result<ExportMaterializeReport, std::io::Error> {
        let plugin_root = plugin_root.as_ref();
        let output_root = output_root.as_ref();
        let mut report = self.materialize(output_root)?;

        for package_id in &self.native_dynamic_packages {
            let Some(source) = find_native_package_dir(plugin_root, package_id)? else {
                report.diagnostics.push(format!(
                    "native dynamic package {package_id} was selected but no plugin.toml was found under {}",
                    plugin_root.display()
                ));
                continue;
            };
            let destination = output_root.join("plugins").join(package_id);
            report.diagnostics.extend(copy_native_dynamic_package(
                &source,
                &destination,
                package_id,
            )?);
            report.copied_packages.push(destination);
        }

        Ok(report)
    }
}

fn find_native_package_dir(
    root: &Path,
    package_id: &str,
) -> Result<Option<PathBuf>, std::io::Error> {
    if !root.exists() {
        return Ok(None);
    }

    let direct = root.join(package_id);
    if package_manifest_matches(&direct.join("plugin.toml"), package_id)? {
        return Ok(Some(direct));
    }

    let mut stack = vec![root.to_path_buf()];
    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            if package_manifest_matches(&path.join("plugin.toml"), package_id)? {
                return Ok(Some(path));
            }
            stack.push(path);
        }
    }

    Ok(None)
}

fn package_manifest_matches(path: &Path, package_id: &str) -> Result<bool, std::io::Error> {
    if !path.exists() {
        return Ok(false);
    }
    let source = fs::read_to_string(path)?;
    Ok(toml::from_str::<PluginPackageManifest>(&source)
        .map(|manifest| manifest.id == package_id)
        .unwrap_or(false))
}

fn copy_native_dynamic_package(
    source: &Path,
    destination: &Path,
    package_id: &str,
) -> Result<Vec<String>, std::io::Error> {
    let mut diagnostics = Vec::new();
    let mut saw_native_dir = false;
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        if source_path.is_dir() {
            if file_name == "native" {
                saw_native_dir = true;
                let copied_artifacts = copy_native_artifacts(&source_path, &destination_path)?;
                if copied_artifacts == 0 {
                    diagnostics.push(format!(
                        "native dynamic package {package_id} has no dynamic library artifacts under {}",
                        source_path.display()
                    ));
                }
            } else if should_copy_native_resource_dir(file_name) {
                copy_dir_all(&source_path, &destination_path)?;
            }
        } else if should_copy_native_dynamic_file(file_name) {
            fs::copy(&source_path, &destination_path)?;
        }
    }
    if !saw_native_dir {
        diagnostics.push(format!(
            "native dynamic package {package_id} has no native artifact directory under {}",
            source.display()
        ));
    }
    Ok(diagnostics)
}

fn should_copy_native_resource_dir(name: &str) -> bool {
    matches!(name, "assets" | "asset" | "resources" | "resource")
}

fn should_copy_native_dynamic_file(name: &str) -> bool {
    name == "plugin.toml"
}

fn copy_dir_all(source: &Path, destination: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir_all(&source_path, &destination_path)?;
        } else {
            if let Some(parent) = destination_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&source_path, &destination_path)?;
        }
    }
    Ok(())
}

fn copy_native_artifacts(source: &Path, destination: &Path) -> Result<usize, std::io::Error> {
    let mut copied = 0;
    if !source.exists() {
        return Ok(copied);
    }
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        if source_path.is_dir() {
            continue;
        }
        let Some(file_name) = source_path.file_name() else {
            continue;
        };
        if !is_native_dynamic_artifact(&source_path) {
            continue;
        }
        fs::copy(&source_path, destination.join(file_name))?;
        copied += 1;
    }
    Ok(copied)
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
