use std::fs;
use std::path::Path;

pub(super) fn copy_native_artifacts(source: &Path, destination: &Path) -> std::io::Result<usize> {
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

pub(super) fn copy_built_native_artifact(
    artifact: &Path,
    destination: &Path,
) -> std::io::Result<()> {
    let Some(file_name) = artifact.file_name() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "built native artifact path has no file name",
        ));
    };
    fs::create_dir_all(destination)?;
    fs::copy(artifact, destination.join(file_name))?;
    Ok(())
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

pub(super) fn dynamic_library_file_name(crate_name: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{crate_name}.dll")
    } else if cfg!(target_os = "macos") {
        format!("lib{crate_name}.dylib")
    } else {
        format!("lib{crate_name}.so")
    }
}
