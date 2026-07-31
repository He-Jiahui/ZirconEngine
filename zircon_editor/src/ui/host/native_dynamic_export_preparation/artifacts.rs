use std::fs;
use std::path::Path;

use crate::core::export::ExportGenerationInventory;

use super::staging::{is_native_dynamic_artifact, NativeStagingStats};

pub(super) fn sync_built_native_artifact(
    artifact: &Path,
    destination: &Path,
    inventory: &mut ExportGenerationInventory,
) -> std::io::Result<NativeStagingStats> {
    let Some(file_name) = artifact.file_name() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "built native artifact path has no file name",
        ));
    };
    fs::create_dir_all(destination)?;
    let mut stats = NativeStagingStats::default();
    for entry in fs::read_dir(destination)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_file()
            && path.file_name() != Some(file_name)
            && is_native_dynamic_artifact(&path)
        {
            fs::remove_file(&path)?;
            inventory.invalidate_subtree(&path);
            stats.removed_files = stats.removed_files.saturating_add(1);
        }
    }

    let destination_path = destination.join(file_name);
    let source_digest = inventory.digest_path(artifact)?;
    let destination_matches = destination_path.is_file()
        && inventory
            .digest_path(&destination_path)
            .is_ok_and(|digest| digest == source_digest);
    if destination_matches {
        return Ok(stats);
    }
    let byte_count = fs::metadata(artifact)?.len();
    fs::copy(artifact, &destination_path)?;
    inventory.invalidate_subtree(&destination_path);
    if inventory.digest_path(&destination_path)? != source_digest {
        return Err(std::io::Error::other(format!(
            "built native artifact staging digest mismatch: {} -> {}",
            artifact.display(),
            destination_path.display()
        )));
    }
    stats.copied_files = 1;
    stats.copied_bytes = byte_count;
    Ok(stats)
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
