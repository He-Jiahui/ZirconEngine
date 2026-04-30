use std::fs;
use std::path::Path;

pub(super) fn stage_native_package_static_files(
    source: &Path,
    destination: &Path,
) -> std::io::Result<()> {
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
