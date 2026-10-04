use std::collections::HashSet;
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use sha2::{Digest, Sha256};

use crate::core::resource::io::replace_staged_file;
use crate::plugin::native::NativePluginArtifactDigest;

const NATIVE_COPY_BUFFER_BYTES: usize = 64 * 1024;
const MATERIALIZED_FILE_STAGING_ATTEMPTS: usize = 1_024;
static MATERIALIZED_FILE_STAGING_SEQUENCE: AtomicU64 = AtomicU64::new(1);

pub(super) struct NativeDynamicPackageFileEntry {
    pub(super) source_path: PathBuf,
    pub(super) relative_path: String,
    pub(super) source_file: File,
    pub(super) source_digest: NativePluginArtifactDigest,
}

pub(super) struct NativeDynamicPackageFileInventory {
    pub(super) entries: Vec<NativeDynamicPackageFileEntry>,
    pub(super) diagnostics: Vec<String>,
}

pub(super) struct MaterializedFileStaging {
    path: PathBuf,
    destination: PathBuf,
    published: bool,
}

impl MaterializedFileStaging {
    pub(super) fn create(destination: &Path) -> Result<(File, Self), std::io::Error> {
        let parent = destination
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)?;
        let destination_name = destination
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("zircon-export"));
        for _ in 0..MATERIALIZED_FILE_STAGING_ATTEMPTS {
            let sequence = MATERIALIZED_FILE_STAGING_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            if sequence == 0 {
                continue;
            }
            let mut staging_name = OsString::from(".");
            staging_name.push(destination_name);
            staging_name.push(format!(".zr-staging-{}-{sequence}", std::process::id()));
            let path = parent.join(staging_name);
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => {
                    return Ok((
                        file,
                        Self {
                            path,
                            destination: destination.to_path_buf(),
                            published: false,
                        },
                    ));
                }
                Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
        Err(std::io::Error::new(
            ErrorKind::AlreadyExists,
            format!(
                "could not allocate a staging file for {}",
                destination.display()
            ),
        ))
    }

    pub(super) fn publish(mut self) -> Result<(), std::io::Error> {
        replace_staged_file(&self.path, &self.destination)?;
        self.published = true;
        Ok(())
    }
}

impl Drop for MaterializedFileStaging {
    fn drop(&mut self) {
        if !self.published {
            let _ = fs::remove_file(&self.path);
        }
    }
}

pub(super) fn copy_native_dynamic_package_files(
    entries: &[NativeDynamicPackageFileEntry],
    destination: &Path,
) -> Result<usize, std::io::Error> {
    let mut created_parents = HashSet::new();
    let mut staged_files = Vec::with_capacity(entries.len());
    fs::create_dir_all(destination)?;
    for entry in entries {
        let destination_path = destination.join(&entry.relative_path);
        if let Some(parent) = destination_path.parent() {
            if created_parents.insert(parent.to_path_buf()) {
                fs::create_dir_all(parent)?;
            }
        }
        if let Some(staging) = stage_file_if_changed(
            &entry.source_file,
            &entry.source_digest,
            &entry.source_path,
            &destination_path,
        )? {
            staged_files.push(staging);
        }
    }
    let copied = staged_files.len();
    for staging in staged_files {
        staging.publish()?;
    }
    Ok(copied)
}

pub(super) fn validate_native_dynamic_package_file_entries(
    entries: &[NativeDynamicPackageFileEntry],
) -> Result<(), std::io::Error> {
    for entry in entries {
        let current_digest =
            NativePluginArtifactDigest::capture_file(&entry.source_file, &entry.source_path)
                .map_err(std::io::Error::other)?;
        if current_digest != entry.source_digest {
            return Err(std::io::Error::new(
                ErrorKind::PermissionDenied,
                format!(
                    "native package source {} changed after inventory capture",
                    entry.source_path.display()
                ),
            ));
        }
    }
    Ok(())
}

// Native payloads can be large, so existing outputs and replacement streams are hashed with
// bounded buffers. Timestamps are ignored because export roots can be restored or copied.
fn copy_file_if_changed(
    source: &File,
    source_digest: &NativePluginArtifactDigest,
    source_path: &Path,
    destination: &Path,
) -> Result<bool, std::io::Error> {
    let Some(staging) = stage_file_if_changed(source, source_digest, source_path, destination)?
    else {
        return Ok(false);
    };
    staging.publish()?;
    Ok(true)
}

fn stage_file_if_changed(
    source: &File,
    source_digest: &NativePluginArtifactDigest,
    source_path: &Path,
    destination: &Path,
) -> Result<Option<MaterializedFileStaging>, std::io::Error> {
    if file_matches_digest(destination, source_digest)? {
        return Ok(None);
    }
    let (mut staging_file, staging) = MaterializedFileStaging::create(destination)?;
    let write_result =
        write_verified_native_source(source, source_digest, source_path, &mut staging_file)
            .and_then(|_| staging_file.sync_all());
    drop(staging_file);
    write_result?;
    Ok(Some(staging))
}

fn file_matches_digest(
    path: &Path,
    expected: &NativePluginArtifactDigest,
) -> Result<bool, std::io::Error> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Ok(false);
    }
    if metadata.len() != expected.byte_length {
        return Ok(false);
    }

    let file = File::open(path)?;
    let actual =
        NativePluginArtifactDigest::capture_file(&file, path).map_err(std::io::Error::other)?;
    Ok(actual == *expected)
}

pub(super) fn write_verified_native_file<W: Write>(
    entry: &NativeDynamicPackageFileEntry,
    destination: &mut W,
) -> Result<u64, std::io::Error> {
    write_verified_native_source(
        &entry.source_file,
        &entry.source_digest,
        &entry.source_path,
        destination,
    )
}

fn write_verified_native_source<W: Write>(
    source: &File,
    expected: &NativePluginArtifactDigest,
    source_path: &Path,
    destination: &mut W,
) -> Result<u64, std::io::Error> {
    let mut source = source.try_clone()?;
    source.seek(SeekFrom::Start(0))?;
    let mut sha256 = Sha256::new();
    let mut buffer = [0_u8; NATIVE_COPY_BUFFER_BYTES];
    let mut byte_length = 0_u64;
    loop {
        let read = source.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        destination.write_all(&buffer[..read])?;
        sha256.update(&buffer[..read]);
        byte_length = byte_length.checked_add(read as u64).ok_or_else(|| {
            std::io::Error::new(
                ErrorKind::InvalidData,
                format!(
                    "native package source {} exceeds supported byte length",
                    source_path.display()
                ),
            )
        })?;
    }
    let actual = NativePluginArtifactDigest {
        sha256: format!("{:x}", sha256.finalize()),
        byte_length,
    };
    if actual != *expected {
        return Err(std::io::Error::new(
            ErrorKind::PermissionDenied,
            format!(
                "native package source {} changed while materialization was streaming it",
                source_path.display()
            ),
        ));
    }
    Ok(byte_length)
}

pub(super) fn native_dynamic_package_file_inventory(
    source: &Path,
    package_id: &str,
) -> Result<NativeDynamicPackageFileInventory, std::io::Error> {
    let mut entries = Vec::new();
    let mut diagnostics = Vec::new();
    let mut saw_native_dir = false;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            diagnostics.push(format!(
                "native dynamic package {package_id} skipped symlinked payload {}",
                entry.path().display()
            ));
            continue;
        }

        let source_path = entry.path();
        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        if file_type.is_dir() {
            if file_name == "native" {
                saw_native_dir = true;
                let previous_entry_count = entries.len();
                collect_native_artifact_entries(&source_path, file_name, source, &mut entries)?;
                if entries.len() == previous_entry_count {
                    diagnostics.push(format!(
                        "native dynamic package {package_id} has no dynamic library artifacts under {}",
                        source_path.display()
                    ));
                }
            } else if should_copy_native_resource_dir(file_name) {
                collect_resource_entries(&source_path, file_name, source, &mut entries)?;
            }
        } else if should_copy_native_dynamic_file(file_name) {
            entries.push(native_file_entry(
                source_path,
                file_name.to_string(),
                source,
            )?);
        }
    }
    if !saw_native_dir {
        diagnostics.push(format!(
            "native dynamic package {package_id} has no native artifact directory under {}",
            source.display()
        ));
    }
    entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(NativeDynamicPackageFileInventory {
        entries,
        diagnostics,
    })
}

fn should_copy_native_resource_dir(name: &str) -> bool {
    matches!(name, "assets" | "asset" | "resources" | "resource")
}

fn should_copy_native_dynamic_file(name: &str) -> bool {
    name == "plugin.toml"
}

fn collect_resource_entries(
    source: &Path,
    relative_prefix: &str,
    package_root: &Path,
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
            collect_resource_entries(&source_path, &relative_path, package_root, entries)?;
        } else {
            entries.push(native_file_entry(source_path, relative_path, package_root)?);
        }
    }
    Ok(())
}

fn collect_native_artifact_entries(
    source: &Path,
    relative_prefix: &str,
    package_root: &Path,
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
        entries.push(native_file_entry(
            source_path,
            format!("{relative_prefix}/{file_name}"),
            package_root,
        )?);
    }
    Ok(())
}

fn native_file_entry(
    source_path: PathBuf,
    relative_path: String,
    package_root: &Path,
) -> Result<NativeDynamicPackageFileEntry, std::io::Error> {
    let source_file = open_native_source_file(&source_path, package_root)?;
    let source_digest = NativePluginArtifactDigest::capture_file(&source_file, &source_path)
        .map_err(std::io::Error::other)?;
    Ok(NativeDynamicPackageFileEntry {
        source_path,
        relative_path,
        source_file,
        source_digest,
    })
}

fn open_native_source_file(path: &Path, package_root: &Path) -> Result<File, std::io::Error> {
    open_native_source_file_with_hook(path, package_root, || {})
}

fn open_native_source_file_with_hook(
    path: &Path,
    package_root: &Path,
    before_open: impl FnOnce(),
) -> Result<File, std::io::Error> {
    let relative = path
        .strip_prefix(package_root)
        .map_err(std::io::Error::other)?;
    let admitted_root = fs::canonicalize(package_root)?;
    let admitted_path = admitted_root.join(relative);
    before_open();
    crate::asset::importer::open_admitted_file(&admitted_path, &admitted_root)
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
    match extension.len() {
        2 => extension.eq_ignore_ascii_case("so"),
        3 => ["dll", "pdb", "dbg"]
            .iter()
            .any(|supported| extension.eq_ignore_ascii_case(supported)),
        4 => extension.eq_ignore_ascii_case("dsym"),
        5 => extension.eq_ignore_ascii_case("dylib"),
        _ => false,
    }
}

#[cfg(test)]
#[path = "copy/tests/native_extension_tests.rs"]
mod native_extension_tests;

#[cfg(test)]
#[path = "tests/copy.rs"]
mod tests;
