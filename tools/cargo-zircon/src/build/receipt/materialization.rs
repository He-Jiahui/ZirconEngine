use std::path::Path;

#[cfg(windows)]
use std::{
    collections::HashSet,
    ffi::OsStr,
    fmt,
    fs::{self, File, OpenOptions},
    path::{Component, PathBuf},
};

#[cfg(windows)]
use super::{
    canonical::upper_hex_matches,
    file_digest::{digest_open_file_handle_bytes_with_buffer, FileDigestBuffer},
    ReceiptArtifact,
};
use super::{ProductReceipt, ProductReceiptError};

pub(crate) fn verify(
    receipt: &ProductReceipt,
    artifact_root: &Path,
) -> Result<(), ProductReceiptError> {
    verify_receipts(std::slice::from_ref(receipt), artifact_root)
}

pub(crate) fn verify_receipts(
    receipts: &[ProductReceipt],
    artifact_root: &Path,
) -> Result<(), ProductReceiptError> {
    require_immutable_materialization_platform()?;
    verify_windows_materialization(receipts, artifact_root)
}

#[cfg(windows)]
fn verify_windows_materialization(
    receipts: &[ProductReceipt],
    artifact_root: &Path,
) -> Result<(), ProductReceiptError> {
    let mut locked_directories =
        open_locked_absolute_directory_chain(artifact_root, "artifact root")?;
    let artifact_root = fs::canonicalize(artifact_root).map_err(|error| {
        ProductReceiptError::new(format!(
            "could not canonicalize product receipt artifact root `{}`: {error}",
            artifact_root.display()
        ))
    })?;
    locked_directories.extend(open_locked_absolute_directory_chain(
        &artifact_root,
        "canonical artifact root",
    )?);
    let root_metadata = fs::metadata(&artifact_root).map_err(|error| {
        ProductReceiptError::new(format!(
            "could not inspect product receipt artifact root `{}`: {error}",
            artifact_root.display()
        ))
    })?;
    if !root_metadata.is_dir() {
        return Err(ProductReceiptError::new(
            "product receipt artifact root must be a directory",
        ));
    }

    let terminal_root = fs::canonicalize(&artifact_root).map_err(|error| {
        ProductReceiptError::new(format!(
            "could not re-canonicalize product receipt artifact root: {error}"
        ))
    })?;
    if terminal_root != artifact_root {
        return Err(ProductReceiptError::new(
            "product receipt artifact root changed while it was being opened",
        ));
    }

    let artifact_count = receipts.iter().fold(0_usize, |count, receipt| {
        count
            .saturating_add(receipt.build_products.len())
            .saturating_add(receipt.runtime_dependencies.len())
            .saturating_add(receipt.symbols.len())
            .saturating_add(usize::from(receipt.sbom.is_some()))
    });
    let mut expected_paths = HashSet::with_capacity(artifact_count);
    let mut expected_directories = HashSet::with_capacity(artifact_count);
    for artifact in receipts.iter().flat_map(receipt_artifacts) {
        expected_paths.insert(artifact.relative_path.as_str());
        insert_expected_artifact_directories(artifact, &mut expected_directories);
    }
    inventory_materialization(
        &artifact_root,
        &mut locked_directories,
        &mut expected_paths,
        &mut expected_directories,
    )?;
    if let Some(path) = expected_paths.iter().min() {
        return Err(ProductReceiptError::new(format!(
            "product receipt artifact root is missing declared artifact `{path}`"
        )));
    }
    if let Some(path) = expected_directories.iter().min() {
        return Err(ProductReceiptError::new(format!(
            "product receipt artifact root is missing declared directory `{path}`"
        )));
    }

    let mut locked_files = Vec::with_capacity(artifact_count);
    let mut artifact_path = PathBuf::new();
    let mut digest_buffer = FileDigestBuffer::new();
    for artifact in receipts.iter().flat_map(receipt_artifacts) {
        materialization_path_into(&mut artifact_path, &artifact_root, &artifact.relative_path);
        let canonical = fs::canonicalize(&artifact_path).map_err(|error| {
            ProductReceiptError::new(format!(
                "could not canonicalize product receipt artifact `{}`: {error}",
                artifact.logical_name
            ))
        })?;
        if !canonical.starts_with(&artifact_root) {
            return Err(ProductReceiptError::new(format!(
                "product receipt artifact `{}` resolved outside its artifact root",
                artifact.logical_name
            )));
        }
        let mut file = open_locked_file(&artifact_path, &artifact.logical_name)?;
        let metadata = file.metadata().map_err(|error| {
            ProductReceiptError::new(format!(
                "could not inspect product receipt artifact `{}`: {error}",
                artifact.logical_name
            ))
        })?;
        reject_reparse_metadata(&metadata, &artifact.logical_name)?;
        let digest = digest_open_file_handle_bytes_with_buffer(&mut file, &mut digest_buffer)?;
        if digest.byte_length != artifact.byte_length
            || !upper_hex_matches(&digest.sha256, &artifact.sha256)
        {
            return Err(ProductReceiptError::new(format!(
                "materialized product receipt artifact `{}` does not match its declared length and SHA-256",
                artifact.logical_name
            )));
        }
        locked_files.push(file);
    }

    drop(locked_files);
    drop(locked_directories);
    Ok(())
}

#[cfg(windows)]
fn materialization_path_into(path: &mut PathBuf, artifact_root: &Path, relative_path: &str) {
    path.clear();
    path.push(artifact_root);
    path.push(relative_path);
}

#[cfg(windows)]
fn open_locked_absolute_directory_chain(
    path: &Path,
    label: &str,
) -> Result<Vec<File>, ProductReceiptError> {
    if !path.is_absolute() {
        return Err(ProductReceiptError::new(
            "product receipt artifact root must be absolute",
        ));
    }
    let mut current = PathBuf::new();
    let mut locked = Vec::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => current.push(component.as_os_str()),
            Component::Normal(_) => {
                current.push(component.as_os_str());
                locked.push(open_locked_directory(&current).map_err(|error| {
                    ProductReceiptError::new(format!(
                        "could not lock {label} component `{}`: {error}",
                        current.display()
                    ))
                })?);
            }
            Component::CurDir | Component::ParentDir => {
                return Err(ProductReceiptError::new(
                    "product receipt artifact root must be a normalized absolute path",
                ));
            }
        }
    }
    if locked.is_empty() {
        return Err(ProductReceiptError::new(
            "product receipt artifact root must name a directory below its volume root",
        ));
    }
    Ok(locked)
}

#[cfg(not(windows))]
fn verify_windows_materialization(
    _receipts: &[ProductReceipt],
    _artifact_root: &Path,
) -> Result<(), ProductReceiptError> {
    unreachable!("platform gate rejects unsupported materialization verification")
}

#[cfg(windows)]
fn inventory_materialization<'a>(
    artifact_root: &Path,
    locked_directories: &mut Vec<File>,
    expected_paths: &mut HashSet<&'a str>,
    expected_directories: &mut HashSet<&'a str>,
) -> Result<(), ProductReceiptError> {
    let mut pending = vec![(artifact_root.to_path_buf(), String::new())];
    while let Some((directory, relative_directory)) = pending.pop() {
        let entries = fs::read_dir(&directory).map_err(|error| {
            ProductReceiptError::new(format!(
                "could not enumerate product receipt artifact directory `{}`: {error}",
                directory.display()
            ))
        })?;
        let mut relative = String::new();
        for entry in entries {
            let entry = entry.map_err(|error| {
                ProductReceiptError::new(format!(
                    "could not enumerate product receipt artifact directory entry: {error}"
                ))
            })?;
            let path = entry.path();
            inventory_relative_path_into(&mut relative, &relative_directory, &entry.file_name())?;
            let metadata = fs::symlink_metadata(&path).map_err(|error| {
                ProductReceiptError::new(format!(
                    "could not inspect product receipt materialization `{}`: {error}",
                    path.display()
                ))
            })?;
            reject_reparse_metadata(&metadata, path.display())?;
            if metadata.is_dir() {
                if !expected_directories.remove(relative.as_str()) {
                    return Err(ProductReceiptError::new(format!(
                        "product receipt artifact root contains undeclared directory `{relative}`"
                    )));
                }
                locked_directories.push(open_locked_directory(&path)?);
                pending.push((path, relative.clone()));
            } else if metadata.is_file() {
                if !expected_paths.remove(relative.as_str()) {
                    return Err(ProductReceiptError::new(format!(
                        "product receipt artifact root contains undeclared artifact `{relative}`"
                    )));
                }
            } else {
                return Err(ProductReceiptError::new(format!(
                    "product receipt artifact root contains unsupported entry `{}`",
                    path.display()
                )));
            }
        }
    }
    Ok(())
}

#[cfg(windows)]
fn inventory_relative_path_into(
    relative: &mut String,
    relative_directory: &str,
    file_name: &OsStr,
) -> Result<(), ProductReceiptError> {
    let file_name = file_name
        .to_str()
        .filter(|name| !name.is_empty())
        .ok_or_else(|| ProductReceiptError::new("product receipt artifact path is not Unicode"))?;
    let separator_length = usize::from(!relative_directory.is_empty());
    let required_capacity = relative_directory
        .len()
        .saturating_add(separator_length)
        .saturating_add(file_name.len());
    relative.clear();
    relative.reserve(required_capacity);
    relative.push_str(relative_directory);
    if separator_length != 0 {
        relative.push('/');
    }
    relative.push_str(file_name);
    Ok(())
}

#[cfg(all(test, windows))]
fn inventory_relative_path(
    relative_directory: &str,
    file_name: &OsStr,
) -> Result<String, ProductReceiptError> {
    let mut relative = String::with_capacity(
        relative_directory
            .len()
            .saturating_add(1)
            .saturating_add(file_name.len()),
    );
    inventory_relative_path_into(&mut relative, relative_directory, file_name)?;
    Ok(relative)
}

#[cfg(windows)]
fn open_locked_directory(path: &Path) -> Result<File, ProductReceiptError> {
    use std::os::windows::fs::OpenOptionsExt;

    let mut options = OpenOptions::new();
    options
        .read(true)
        .share_mode(0x0000_0001)
        .custom_flags(0x0200_0000 | 0x0020_0000);
    let file = options.open(path).map_err(|error| {
        ProductReceiptError::new(format!(
            "could not lock product receipt artifact directory `{}`: {error}",
            path.display()
        ))
    })?;
    let metadata = file.metadata().map_err(|error| {
        ProductReceiptError::new(format!(
            "could not inspect locked product receipt artifact directory `{}`: {error}",
            path.display()
        ))
    })?;
    reject_reparse_metadata(&metadata, path.display())?;
    if !metadata.is_dir() {
        return Err(ProductReceiptError::new(format!(
            "product receipt artifact directory `{}` is not a directory",
            path.display()
        )));
    }
    Ok(file)
}

#[cfg(windows)]
fn open_locked_file(path: &Path, label: &str) -> Result<File, ProductReceiptError> {
    use std::os::windows::fs::OpenOptionsExt;

    let mut options = OpenOptions::new();
    options
        .read(true)
        .share_mode(0x0000_0001)
        .custom_flags(0x0020_0000);
    options.open(path).map_err(|error| {
        ProductReceiptError::new(format!(
            "could not lock product receipt artifact `{label}` at `{}`: {error}",
            path.display()
        ))
    })
}

#[cfg(windows)]
fn reject_reparse_metadata(
    metadata: &fs::Metadata,
    label: impl fmt::Display,
) -> Result<(), ProductReceiptError> {
    use std::os::windows::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
    if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(ProductReceiptError::new(format!(
            "product receipt materialization `{label}` must not be a reparse point"
        )));
    }
    Ok(())
}

#[cfg(windows)]
fn receipt_artifacts(receipt: &ProductReceipt) -> impl Iterator<Item = &ReceiptArtifact> {
    receipt
        .build_products
        .iter()
        .chain(&receipt.runtime_dependencies)
        .chain(&receipt.symbols)
        .chain(receipt.sbom.as_ref())
}

#[cfg(windows)]
fn insert_expected_artifact_directories<'a>(
    artifact: &'a ReceiptArtifact,
    directories: &mut HashSet<&'a str>,
) {
    for (separator, _) in artifact.relative_path.match_indices('/') {
        directories.insert(&artifact.relative_path[..separator]);
    }
}

#[cfg(windows)]
fn require_immutable_materialization_platform() -> Result<(), ProductReceiptError> {
    Ok(())
}

#[cfg(all(test, windows))]
mod performance_tests;

#[cfg(all(test, windows))]
mod tests;

#[cfg(not(windows))]
fn require_immutable_materialization_platform() -> Result<(), ProductReceiptError> {
    Err(ProductReceiptError::new(
        "the immutable ProductReceipt materialization verifier is not implemented on this platform",
    ))
}
