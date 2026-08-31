use std::cmp::Ordering;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::build::receipt::{canonical::upper_hex_matches, ProductReceiptError};

use super::hex_digest;

const BUILD_SET_SCHEMA_VERSION: u32 = 1;
const BUILD_SET_KIND: &str = "zircon_mvp_product_build_set";
const BUILD_SET_STATUS: &str = "completed";
const BUILD_SET_SOURCE_POLICY: &str = "tracked_head_plus_tracked_dirty_overlay";
const BUILD_SET_SNAPSHOT_RELATIVE_PATH: &str = "source";
const BUILD_SET_ID_PREFIX: &str = "zircon-mvp-build-set-v1";
const BUILD_SET_MANIFEST_LIMIT: usize = 64 * 1024 * 1024;
const BUILD_SET_FILE_LIMIT: usize = 100_000;
const BUILD_SET_DIRECTORY_LIMIT: usize = 100_000;
const BUILD_SET_DIRECTORY_DEPTH_LIMIT: usize = 256;
const BUILD_SET_HASH_BUFFER_BYTES: usize = 64 * 1024;
const GIT_LFS_PREFIX: &[u8] = b"version https://git-lfs.github.com/spec/v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BuildSetManifest {
    schema_version: u32,
    build_set_kind: String,
    status: String,
    build_set_id: String,
    created_utc: String,
    snapshot_relative_path: String,
    source_policy: String,
    git_revision: String,
    dirty_overlay_sha256: String,
    files: Vec<BuildSetFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BuildSetFile {
    relative_path: String,
    sha256: String,
    byte_length: u64,
}

pub(super) struct ValidatedBuildSet {
    pub(super) build_set_id: String,
    pub(super) snapshot_root: PathBuf,
    expected_inventory: HashSet<String>,
    _locked_files: Vec<File>,
    _locked_directories: Vec<File>,
}

struct SnapshotInventory {
    files: Vec<String>,
    locked_directories: Vec<File>,
}

impl ValidatedBuildSet {
    pub(super) fn open(manifest_path: &Path) -> Result<Self, ProductReceiptError> {
        require_immutable_build_set_platform()?;
        let manifest_path = canonical_regular_file(manifest_path, "BuildSet manifest")?;
        let mut manifest_file = open_locked_file(&manifest_path, "BuildSet manifest")?;
        let manifest_bytes = read_bounded_file(
            &mut manifest_file,
            BUILD_SET_MANIFEST_LIMIT,
            "BuildSet manifest",
        )?;
        let manifest: BuildSetManifest =
            serde_json::from_slice(&manifest_bytes).map_err(|error| {
                ProductReceiptError::new(format!("could not parse BuildSet manifest: {error}"))
            })?;
        validate_manifest_authority(&manifest)?;

        let manifest_directory = manifest_path
            .parent()
            .ok_or_else(|| ProductReceiptError::new("BuildSet manifest has no parent directory"))?;
        let snapshot_root = canonical_directory(
            &manifest_directory.join(BUILD_SET_SNAPSHOT_RELATIVE_PATH),
            "BuildSet snapshot root",
        )?;
        if snapshot_root.parent() != Some(manifest_directory) {
            return Err(ProductReceiptError::new(
                "BuildSet snapshot root must be the direct `source` child of its manifest directory",
            ));
        }

        let snapshot_inventory =
            inventory_snapshot_with_directory_leases(&snapshot_root, manifest.files.len())?;
        validate_manifest_files(&manifest.files, &snapshot_inventory.files)?;
        let mut locked_files = Vec::with_capacity(manifest.files.len() + 1);
        locked_files.push(manifest_file);
        let mut hash_buffer = [0_u8; BUILD_SET_HASH_BUFFER_BYTES];
        for expected in &manifest.files {
            let path = snapshot_root.join(relative_path(&expected.relative_path)?);
            locked_files.push(open_verified_snapshot_file(
                &path,
                expected,
                &mut hash_buffer,
            )?);
        }
        let expected_inventory = snapshot_inventory.files.into_iter().collect();
        verify_snapshot_inventory(&snapshot_root, &expected_inventory)?;

        let derived_id = derive_build_set_id(&manifest);
        if manifest.build_set_id != derived_id {
            return Err(ProductReceiptError::new(
                "BuildSet manifest build_set_id does not match its source tree",
            ));
        }
        Ok(Self {
            build_set_id: derived_id,
            snapshot_root,
            expected_inventory,
            _locked_files: locked_files,
            _locked_directories: snapshot_inventory.locked_directories,
        })
    }

    pub(super) fn verify_inventory(&self) -> Result<(), ProductReceiptError> {
        verify_snapshot_inventory(&self.snapshot_root, &self.expected_inventory)
    }
}

fn validate_manifest_authority(manifest: &BuildSetManifest) -> Result<(), ProductReceiptError> {
    if manifest.schema_version != BUILD_SET_SCHEMA_VERSION
        || manifest.build_set_kind != BUILD_SET_KIND
        || manifest.status != BUILD_SET_STATUS
        || manifest.snapshot_relative_path != BUILD_SET_SNAPSHOT_RELATIVE_PATH
        || manifest.source_policy != BUILD_SET_SOURCE_POLICY
    {
        return Err(ProductReceiptError::new(
            "BuildSet manifest has an unexpected schema, status, snapshot, or source policy",
        ));
    }
    require_hex("BuildSet id", &manifest.build_set_id, 64, true)?;
    require_hex("BuildSet Git revision", &manifest.git_revision, 40, false)?;
    require_hex(
        "BuildSet dirty overlay digest",
        &manifest.dirty_overlay_sha256,
        64,
        true,
    )?;
    validate_utc_timestamp(&manifest.created_utc)?;
    if manifest.files.is_empty() || manifest.files.len() > BUILD_SET_FILE_LIMIT {
        return Err(ProductReceiptError::new(format!(
            "BuildSet manifest file count must be between 1 and {BUILD_SET_FILE_LIMIT}"
        )));
    }
    Ok(())
}

fn validate_manifest_files(
    expected: &[BuildSetFile],
    actual_inventory: &[String],
) -> Result<(), ProductReceiptError> {
    let mut previous = None;
    for file in expected {
        validate_relative_path(&file.relative_path)?;
        require_hex("BuildSet file digest", &file.sha256, 64, true)?;
        if previous.is_some_and(|previous: &str| {
            ordinal_compare(previous, &file.relative_path) != Ordering::Less
        }) {
            return Err(ProductReceiptError::new(
                "BuildSet manifest file paths must be unique and ordinally sorted",
            ));
        }
        previous = Some(file.relative_path.as_str());
    }
    if expected.len() != actual_inventory.len()
        || expected
            .iter()
            .zip(actual_inventory)
            .any(|(expected, actual)| expected.relative_path != *actual)
    {
        return Err(ProductReceiptError::new(
            "BuildSet snapshot inventory differs from its manifest",
        ));
    }
    Ok(())
}

fn verify_file_content(
    file: &mut File,
    initial_metadata: &fs::Metadata,
    expected: &BuildSetFile,
    buffer: &mut [u8; BUILD_SET_HASH_BUFFER_BYTES],
) -> Result<(), ProductReceiptError> {
    if !initial_metadata.is_file() || initial_metadata.len() != expected.byte_length {
        return Err(ProductReceiptError::new(format!(
            "BuildSet snapshot file content differs from its manifest: {}",
            expected.relative_path
        )));
    }
    let mut prefix = [0_u8; GIT_LFS_PREFIX.len() + 2];
    let mut prefix_length = 0_usize;
    let mut observed = 0_u64;
    let mut hasher = Sha256::new();
    loop {
        let count = file.read(buffer).map_err(|error| {
            ProductReceiptError::new(format!(
                "could not read BuildSet snapshot file `{}`: {error}",
                expected.relative_path
            ))
        })?;
        if count == 0 {
            break;
        }
        capture_prefix(&mut prefix, &mut prefix_length, &buffer[..count]);
        observed = observed
            .checked_add(count as u64)
            .ok_or_else(|| ProductReceiptError::new("BuildSet snapshot file length overflowed"))?;
        hasher.update(&buffer[..count]);
    }
    if is_unmaterialized_lfs_pointer(&prefix[..prefix_length]) {
        return Err(ProductReceiptError::new(format!(
            "BuildSet rejects an unmaterialized Git LFS pointer: {}",
            expected.relative_path
        )));
    }
    let final_metadata = file.metadata().map_err(|error| {
        ProductReceiptError::new(format!(
            "could not re-inspect BuildSet snapshot file `{}`: {error}",
            expected.relative_path
        ))
    })?;
    let actual_digest = hasher.finalize();
    if observed != expected.byte_length
        || final_metadata.len() != expected.byte_length
        || !upper_hex_matches(&actual_digest, &expected.sha256)
    {
        return Err(ProductReceiptError::new(format!(
            "BuildSet snapshot file content differs from its manifest: {}",
            expected.relative_path
        )));
    }
    Ok(())
}

fn open_verified_snapshot_file(
    path: &Path,
    expected: &BuildSetFile,
    buffer: &mut [u8; BUILD_SET_HASH_BUFFER_BYTES],
) -> Result<File, ProductReceiptError> {
    let (mut file, initial_metadata) =
        open_locked_file_with_metadata(path, &expected.relative_path)?;
    verify_file_content(&mut file, &initial_metadata, expected, buffer)?;
    Ok(file)
}

fn capture_prefix(prefix: &mut [u8], prefix_length: &mut usize, bytes: &[u8]) {
    let count = bytes.len().min(prefix.len() - *prefix_length);
    prefix[*prefix_length..*prefix_length + count].copy_from_slice(&bytes[..count]);
    *prefix_length += count;
}

fn is_unmaterialized_lfs_pointer(prefix: &[u8]) -> bool {
    prefix.starts_with(GIT_LFS_PREFIX)
        && prefix
            .get(GIT_LFS_PREFIX.len())
            .is_some_and(|byte| *byte == b'\n' || *byte == b'\r')
}

fn inventory_snapshot_with_directory_leases(
    snapshot_root: &Path,
    file_capacity: usize,
) -> Result<SnapshotInventory, ProductReceiptError> {
    collect_snapshot_inventory(snapshot_root, true, file_capacity)
}

fn collect_snapshot_inventory(
    snapshot_root: &Path,
    lock_directories: bool,
    file_capacity: usize,
) -> Result<SnapshotInventory, ProductReceiptError> {
    let mut inventory = Vec::with_capacity(file_capacity);
    let (locked_directories, _) =
        visit_snapshot_files(snapshot_root, lock_directories, |relative| {
            inventory.push(relative.to_owned());
            Ok(())
        })?;
    inventory.sort_by(|left, right| ordinal_compare(left, right));
    Ok(SnapshotInventory {
        files: inventory,
        locked_directories,
    })
}

fn verify_snapshot_inventory(
    snapshot_root: &Path,
    expected_inventory: &HashSet<String>,
) -> Result<(), ProductReceiptError> {
    let (_, observed_count) = visit_snapshot_files(snapshot_root, false, |relative| {
        if !expected_inventory.contains(relative) {
            return Err(ProductReceiptError::new(
                "BuildSet snapshot inventory changed during the product build",
            ));
        }
        Ok(())
    })?;
    if observed_count != expected_inventory.len() {
        return Err(ProductReceiptError::new(
            "BuildSet snapshot inventory changed during the product build",
        ));
    }
    Ok(())
}

fn visit_snapshot_files(
    snapshot_root: &Path,
    lock_directories: bool,
    mut visit_file: impl FnMut(&str) -> Result<(), ProductReceiptError>,
) -> Result<(Vec<File>, usize), ProductReceiptError> {
    let mut locked_directories = Vec::new();
    let mut pending = vec![(snapshot_root.to_path_buf(), 0_usize, String::new())];
    let mut directories = 0_usize;
    let mut files = 0_usize;
    while let Some((directory, depth, relative_directory)) = pending.pop() {
        directories = directories.checked_add(1).ok_or_else(|| {
            ProductReceiptError::new("BuildSet snapshot directory count overflowed")
        })?;
        if directories > BUILD_SET_DIRECTORY_LIMIT || depth > BUILD_SET_DIRECTORY_DEPTH_LIMIT {
            return Err(ProductReceiptError::new(
                "BuildSet snapshot exceeded its directory count or depth limit",
            ));
        }
        let metadata = if lock_directories {
            let (directory_lease, metadata) = open_locked_directory_with_metadata(&directory)?;
            locked_directories.push(directory_lease);
            metadata
        } else {
            fs::symlink_metadata(&directory).map_err(|error| {
                ProductReceiptError::new(format!(
                    "could not inspect BuildSet snapshot directory `{}`: {error}",
                    directory.display()
                ))
            })?
        };
        if is_reparse_or_symlink(&metadata) || !metadata.is_dir() {
            return Err(ProductReceiptError::new(format!(
                "BuildSet snapshot contains a reparse-point or non-directory path: {}",
                directory.display()
            )));
        }
        let mut relative = String::new();
        for entry in fs::read_dir(&directory).map_err(|error| {
            ProductReceiptError::new(format!(
                "could not enumerate BuildSet snapshot directory `{}`: {error}",
                directory.display()
            ))
        })? {
            let entry = entry.map_err(|error| {
                ProductReceiptError::new(format!("could not enumerate BuildSet snapshot: {error}"))
            })?;
            let path = entry.path();
            snapshot_relative_path_into(&mut relative, &relative_directory, &entry.file_name())?;
            let metadata = fs::symlink_metadata(&path).map_err(|error| {
                ProductReceiptError::new(format!(
                    "could not inspect BuildSet snapshot entry `{}`: {error}",
                    path.display()
                ))
            })?;
            if is_reparse_or_symlink(&metadata) {
                return Err(ProductReceiptError::new(format!(
                    "BuildSet snapshot contains a reparse point: {}",
                    path.display()
                )));
            }
            if metadata.is_dir() {
                pending.push((path, depth + 1, relative.clone()));
                continue;
            }
            if !metadata.is_file() {
                return Err(ProductReceiptError::new(format!(
                    "BuildSet snapshot contains a non-regular file: {}",
                    path.display()
                )));
            }
            if relative == ".git" {
                continue;
            }
            files = files.checked_add(1).ok_or_else(|| {
                ProductReceiptError::new("BuildSet snapshot file count overflowed")
            })?;
            if files > BUILD_SET_FILE_LIMIT {
                return Err(ProductReceiptError::new(format!(
                    "BuildSet snapshot exceeded the {BUILD_SET_FILE_LIMIT}-file limit"
                )));
            }
            visit_file(&relative)?;
        }
    }
    Ok((locked_directories, files))
}

fn derive_build_set_id(manifest: &BuildSetManifest) -> String {
    let mut hasher = Sha256::new();
    for value in [
        BUILD_SET_ID_PREFIX,
        &manifest.git_revision,
        &manifest.dirty_overlay_sha256,
    ] {
        update_length_framed(&mut hasher, value);
    }
    for file in &manifest.files {
        update_length_framed(&mut hasher, &file.relative_path);
        update_length_framed(&mut hasher, &file.sha256);
        update_length_framed_u64(&mut hasher, file.byte_length);
    }
    hex_digest(&hasher.finalize())
}

fn update_length_framed(hasher: &mut Sha256, value: &str) {
    let bytes = value.as_bytes();
    hasher.update((bytes.len() as i64).to_le_bytes());
    hasher.update(bytes);
}

fn update_length_framed_u64(hasher: &mut Sha256, mut value: u64) {
    let mut digits = [0_u8; 20];
    let mut start = digits.len();
    loop {
        start -= 1;
        digits[start] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    let bytes = &digits[start..];
    hasher.update((bytes.len() as i64).to_le_bytes());
    hasher.update(bytes);
}

fn relative_path(value: &str) -> Result<PathBuf, ProductReceiptError> {
    validate_relative_path(value)?;
    Ok(PathBuf::from(value))
}

fn validate_relative_path(value: &str) -> Result<(), ProductReceiptError> {
    let bytes = value.as_bytes();
    if value.trim().is_empty()
        || bytes.is_empty()
        || bytes[0] == b'/'
        || bytes.last() == Some(&b'/')
        || is_windows_drive_prefix(bytes)
    {
        return Err(unsafe_relative_path(value));
    }

    let mut component_start = 0;
    for (index, byte) in bytes.iter().copied().enumerate() {
        match byte {
            b'\\' => return Err(unsafe_relative_path(value)),
            b'/' => {
                if index == component_start || is_dot_component(&bytes[component_start..index]) {
                    return Err(unsafe_relative_path(value));
                }
                component_start = index + 1;
            }
            _ => {}
        }
    }
    if is_dot_component(&bytes[component_start..]) {
        return Err(unsafe_relative_path(value));
    }
    Ok(())
}

fn is_windows_drive_prefix(bytes: &[u8]) -> bool {
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

fn is_dot_component(component: &[u8]) -> bool {
    component == b"." || component == b".."
}

fn unsafe_relative_path(value: &str) -> ProductReceiptError {
    ProductReceiptError::new(format!(
        "BuildSet contains an unsafe relative path `{value}`"
    ))
}

fn snapshot_relative_path_into(
    relative: &mut String,
    relative_directory: &str,
    file_name: &OsStr,
) -> Result<(), ProductReceiptError> {
    let file_name = file_name
        .to_str()
        .filter(|name| !name.is_empty())
        .ok_or_else(|| ProductReceiptError::new("BuildSet snapshot path is not Unicode"))?;
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

#[cfg(test)]
fn snapshot_relative_path(
    relative_directory: &str,
    file_name: &OsStr,
) -> Result<String, ProductReceiptError> {
    let mut relative = String::with_capacity(
        relative_directory
            .len()
            .saturating_add(1)
            .saturating_add(file_name.len()),
    );
    snapshot_relative_path_into(&mut relative, relative_directory, file_name)?;
    Ok(relative)
}

fn ordinal_compare(left: &str, right: &str) -> Ordering {
    if left.is_ascii() && right.is_ascii() {
        left.as_bytes().cmp(right.as_bytes())
    } else {
        left.encode_utf16().cmp(right.encode_utf16())
    }
}

fn require_hex(
    label: &str,
    value: &str,
    length: usize,
    uppercase: bool,
) -> Result<(), ProductReceiptError> {
    let valid_case = if uppercase {
        value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'A'..=b'F').contains(&byte))
    } else {
        value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    };
    if value.len() != length || !valid_case {
        return Err(ProductReceiptError::new(format!(
            "{label} has an invalid hexadecimal identity"
        )));
    }
    Ok(())
}

fn validate_utc_timestamp(value: &str) -> Result<(), ProductReceiptError> {
    let timestamp = value.strip_suffix('Z').ok_or_else(|| {
        ProductReceiptError::new("BuildSet created_utc must be an ISO-8601 UTC timestamp")
    })?;
    let base = timestamp
        .split_once('.')
        .map_or(timestamp, |(base, fraction)| {
            if fraction.is_empty() || !fraction.bytes().all(|byte| byte.is_ascii_digit()) {
                ""
            } else {
                base
            }
        });
    let bytes = base.as_bytes();
    if bytes.len() != 19
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes
            .iter()
            .copied()
            .enumerate()
            .any(|(index, byte)| !matches!(index, 4 | 7 | 10 | 13 | 16) && !byte.is_ascii_digit())
    {
        return Err(ProductReceiptError::new(
            "BuildSet created_utc must be an ISO-8601 UTC timestamp",
        ));
    }
    let year = decimal_component(bytes, 0, 4);
    let month = decimal_component(bytes, 5, 2);
    let day = decimal_component(bytes, 8, 2);
    let hour = decimal_component(bytes, 11, 2);
    let minute = decimal_component(bytes, 14, 2);
    let second = decimal_component(bytes, 17, 2);
    let leap_year = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let maximum_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap_year => 29,
        2 => 28,
        _ => 0,
    };
    if year == 0 || day == 0 || day > maximum_day || hour > 23 || minute > 59 || second > 59 {
        return Err(ProductReceiptError::new(
            "BuildSet created_utc must be an ISO-8601 UTC timestamp",
        ));
    }
    Ok(())
}

fn decimal_component(bytes: &[u8], start: usize, length: usize) -> u32 {
    bytes[start..start + length]
        .iter()
        .fold(0_u32, |value, byte| value * 10 + u32::from(byte - b'0'))
}

fn canonical_regular_file(path: &Path, label: &str) -> Result<PathBuf, ProductReceiptError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        ProductReceiptError::new(format!(
            "could not inspect {label} `{}`: {error}",
            path.display()
        ))
    })?;
    if is_reparse_or_symlink(&metadata) || !metadata.is_file() {
        return Err(ProductReceiptError::new(format!(
            "{label} must be a non-reparse regular file"
        )));
    }
    fs::canonicalize(path).map_err(|error| {
        ProductReceiptError::new(format!(
            "could not resolve {label} `{}`: {error}",
            path.display()
        ))
    })
}

fn canonical_directory(path: &Path, label: &str) -> Result<PathBuf, ProductReceiptError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        ProductReceiptError::new(format!(
            "could not inspect {label} `{}`: {error}",
            path.display()
        ))
    })?;
    if is_reparse_or_symlink(&metadata) || !metadata.is_dir() {
        return Err(ProductReceiptError::new(format!(
            "{label} must be a non-reparse directory"
        )));
    }
    fs::canonicalize(path).map_err(|error| {
        ProductReceiptError::new(format!(
            "could not resolve {label} `{}`: {error}",
            path.display()
        ))
    })
}

fn open_locked_file(path: &Path, label: &str) -> Result<File, ProductReceiptError> {
    open_locked_file_with_metadata(path, label).map(|(file, _)| file)
}

fn open_locked_file_with_metadata(
    path: &Path,
    label: &str,
) -> Result<(File, fs::Metadata), ProductReceiptError> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;

        const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;

        options
            .share_mode(0x0000_0001)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
    }
    let file = options.open(path).map_err(|error| {
        ProductReceiptError::new(format!(
            "could not lock {label} `{}`: {error}",
            path.display()
        ))
    })?;
    let metadata = file.metadata().map_err(|error| {
        ProductReceiptError::new(format!(
            "could not inspect locked {label} `{}`: {error}",
            path.display()
        ))
    })?;
    if is_reparse_or_symlink(&metadata) || !metadata.is_file() {
        return Err(ProductReceiptError::new(format!(
            "{label} must be a non-reparse regular file"
        )));
    }
    Ok((file, metadata))
}

#[cfg(windows)]
fn open_locked_directory_with_metadata(
    path: &Path,
) -> Result<(File, fs::Metadata), ProductReceiptError> {
    use std::os::windows::fs::OpenOptionsExt;

    const FILE_SHARE_READ: u32 = 0x0000_0001;
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;

    let directory = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
        .map_err(|error| {
            ProductReceiptError::new(format!(
                "could not lock BuildSet snapshot directory `{}`: {error}",
                path.display()
            ))
        })?;
    let metadata = directory.metadata().map_err(|error| {
        ProductReceiptError::new(format!(
            "could not inspect locked BuildSet snapshot directory `{}`: {error}",
            path.display()
        ))
    })?;
    if is_reparse_or_symlink(&metadata) || !metadata.is_dir() {
        return Err(ProductReceiptError::new(
            "BuildSet snapshot directory lease must identify a non-reparse directory",
        ));
    }
    Ok((directory, metadata))
}

#[cfg(not(windows))]
fn open_locked_directory_with_metadata(
    _path: &Path,
) -> Result<(File, fs::Metadata), ProductReceiptError> {
    Err(ProductReceiptError::new(
        "immutable BuildSet directory leases are not implemented on this platform",
    ))
}

#[cfg(windows)]
fn require_immutable_build_set_platform() -> Result<(), ProductReceiptError> {
    Ok(())
}

#[cfg(not(windows))]
fn require_immutable_build_set_platform() -> Result<(), ProductReceiptError> {
    Err(ProductReceiptError::new(
        "product builds require the Windows immutable BuildSet backend",
    ))
}

fn read_bounded_file(
    file: &mut File,
    limit: usize,
    label: &str,
) -> Result<Vec<u8>, ProductReceiptError> {
    let declared_length = file
        .metadata()
        .map_err(|error| ProductReceiptError::new(format!("could not inspect {label}: {error}")))?
        .len();
    if declared_length > limit as u64 {
        return Err(ProductReceiptError::new(format!(
            "{label} exceeds the {limit}-byte limit"
        )));
    }
    let mut bytes = Vec::with_capacity(declared_length as usize);
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| ProductReceiptError::new(format!("could not read {label}: {error}")))?;
    if bytes.len() > limit {
        return Err(ProductReceiptError::new(format!(
            "{label} exceeds the {limit}-byte limit"
        )));
    }
    Ok(bytes)
}

#[cfg(windows)]
fn is_reparse_or_symlink(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    metadata.file_attributes() & 0x0000_0400 != 0
}

#[cfg(not(windows))]
fn is_reparse_or_symlink(metadata: &fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
}

#[cfg(test)]
mod behavior_tests;

#[cfg(test)]
mod performance_tests;

#[cfg(test)]
mod tests;
