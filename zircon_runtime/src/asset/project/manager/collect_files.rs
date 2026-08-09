use std::fs;
use std::path::{Path, PathBuf};

use crate::asset::safe_project_path::is_link_or_reparse;
use crate::asset::AssetImportError;

use super::is_meta_sidecar::is_meta_sidecar;

pub(super) fn collect_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<(), AssetImportError> {
    collect_matching_files(root, files, |path| {
        !is_meta_sidecar(path)
            && !crate::core::resource::io::atomic_file::is_atomic_write_transaction_path(path)
            && !is_auxiliary_source_file(path)
    })
}

pub(super) fn collect_matching_files<F>(
    root: &Path,
    files: &mut Vec<PathBuf>,
    mut include: F,
) -> Result<(), AssetImportError>
where
    F: FnMut(&Path) -> bool,
{
    if !root.exists() {
        return Ok(());
    }
    collect_matching_files_recursive(root, files, &mut include)
}

fn collect_matching_files_recursive<F>(
    directory: &Path,
    files: &mut Vec<PathBuf>,
    include: &mut F,
) -> Result<(), AssetImportError>
where
    F: FnMut(&Path) -> bool,
{
    let metadata = fs::symlink_metadata(directory)?;
    reject_link_or_reparse(directory, &metadata)?;
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;
        reject_link_or_reparse(&path, &metadata)?;
        if metadata.is_dir() {
            collect_matching_files_recursive(&path, files, include)?;
        } else if metadata.is_file() && include(&path) {
            files.push(path);
        }
    }
    Ok(())
}

fn reject_link_or_reparse(path: &Path, metadata: &fs::Metadata) -> Result<(), AssetImportError> {
    if is_link_or_reparse(metadata) {
        return Err(AssetImportError::UnsafeProjectAssetLink {
            path: path.to_path_buf(),
        });
    }
    Ok(())
}

fn is_auxiliary_source_file(path: &Path) -> bool {
    // External glTF buffers and raw font binaries are source auxiliaries, not standalone assets.
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("bin")
                || extension.eq_ignore_ascii_case("ttf")
                || extension.eq_ignore_ascii_case("otf")
                || extension.eq_ignore_ascii_case("woff")
                || extension.eq_ignore_ascii_case("woff2")
        })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::collect_files;

    static NEXT_TEST_ROOT: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn source_collection_ignores_atomic_write_transaction_siblings() {
        let root = std::env::temp_dir().join(format!(
            "zircon_collect_files_atomic_siblings_{}_{}",
            std::process::id(),
            NEXT_TEST_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let source = root.join("material.zmaterial");
        let staging = root.join(".material.zmaterial.zr-staging-123-4");
        let backup = root.join(".material.zmaterial.zr-backup-123-5");
        fs::write(&source, "source").unwrap();
        fs::write(&staging, "staging").unwrap();
        fs::write(&backup, "backup").unwrap();

        let mut files = Vec::new();
        collect_files(&root, &mut files).unwrap();

        assert_eq!(files, vec![source]);
        let _ = fs::remove_dir_all(root);
    }
}
