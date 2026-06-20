use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub(super) struct NativeDynamicPackageCopyPreview {
    pub(super) diagnostics: Vec<String>,
}

pub(super) struct NativeDynamicPackageFileEntry {
    pub(super) source_path: PathBuf,
    pub(super) relative_path: String,
}

pub(super) fn copy_native_dynamic_package(
    source: &Path,
    destination: &Path,
    package_id: &str,
) -> Result<Vec<String>, std::io::Error> {
    let mut diagnostics = Vec::new();
    let mut saw_native_dir = false;
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        if file_type.is_symlink() {
            diagnostics.push(format!(
                "native dynamic package {package_id} skipped symlinked payload {}",
                source_path.display()
            ));
            continue;
        }
        if file_type.is_dir() {
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

pub(super) fn preview_native_dynamic_package_copy(
    source: &Path,
    package_id: &str,
) -> Result<NativeDynamicPackageCopyPreview, std::io::Error> {
    let mut diagnostics = Vec::new();
    let mut saw_native_dir = false;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let source_path = entry.path();
        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        if file_type.is_symlink() {
            diagnostics.push(format!(
                "native dynamic package {package_id} skipped symlinked payload {}",
                source_path.display()
            ));
            continue;
        }
        if file_type.is_dir() && file_name == "native" {
            saw_native_dir = true;
            let artifact_count = count_native_artifacts(&source_path)?;
            if artifact_count == 0 {
                diagnostics.push(format!(
                    "native dynamic package {package_id} has no dynamic library artifacts under {}",
                    source_path.display()
                ));
            }
        }
    }
    if !saw_native_dir {
        diagnostics.push(format!(
            "native dynamic package {package_id} has no native artifact directory under {}",
            source.display()
        ));
    }
    Ok(NativeDynamicPackageCopyPreview { diagnostics })
}

pub(super) fn native_dynamic_package_file_entries(
    source: &Path,
) -> Result<Vec<NativeDynamicPackageFileEntry>, std::io::Error> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }

        let source_path = entry.path();
        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        if file_type.is_dir() {
            if file_name == "native" {
                collect_native_artifact_entries(&source_path, file_name, &mut entries)?;
            } else if should_copy_native_resource_dir(file_name) {
                collect_resource_entries(&source_path, file_name, &mut entries)?;
            }
        } else if should_copy_native_dynamic_file(file_name) {
            entries.push(NativeDynamicPackageFileEntry {
                source_path,
                relative_path: file_name.to_string(),
            });
        }
    }
    entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(entries)
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
        let file_type = entry.file_type()?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
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

fn collect_resource_entries(
    source: &Path,
    relative_prefix: &str,
    entries: &mut Vec<NativeDynamicPackageFileEntry>,
) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        let source_path = entry.path();
        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        let relative_path = format!("{relative_prefix}/{file_name}");
        if file_type.is_dir() {
            collect_resource_entries(&source_path, &relative_path, entries)?;
        } else {
            entries.push(NativeDynamicPackageFileEntry {
                source_path,
                relative_path,
            });
        }
    }
    Ok(())
}

fn collect_native_artifact_entries(
    source: &Path,
    relative_prefix: &str,
    entries: &mut Vec<NativeDynamicPackageFileEntry>,
) -> Result<(), std::io::Error> {
    if !is_real_directory(source)? {
        return Ok(());
    }
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() || file_type.is_dir() {
            continue;
        }
        let source_path = entry.path();
        if !is_native_dynamic_artifact(&source_path) {
            continue;
        }
        let Some(file_name) = source_path
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .map(str::to_owned)
        else {
            continue;
        };
        entries.push(NativeDynamicPackageFileEntry {
            source_path,
            relative_path: format!("{relative_prefix}/{file_name}"),
        });
    }
    Ok(())
}

fn copy_native_artifacts(source: &Path, destination: &Path) -> Result<usize, std::io::Error> {
    let mut copied = 0;
    if !is_real_directory(source)? {
        return Ok(copied);
    }
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() || file_type.is_dir() {
            continue;
        }
        let source_path = entry.path();
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

fn count_native_artifacts(source: &Path) -> Result<usize, std::io::Error> {
    let mut count = 0;
    if !is_real_directory(source)? {
        return Ok(count);
    }
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() || file_type.is_dir() {
            continue;
        }
        let source_path = entry.path();
        if is_native_dynamic_artifact(&source_path) {
            count += 1;
        }
    }
    Ok(count)
}

fn is_real_directory(path: &Path) -> Result<bool, std::io::Error> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(metadata.is_dir() && !metadata.file_type().is_symlink()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
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
