use std::{
    fs::{self, File, OpenOptions},
    io::Read,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};
use zircon_runtime_interface::project::{ProjectGuid, ProjectManifestSummary};

use super::manifest::{valid_path, FileEntry, Manifest, MAX_BLOB_BYTES, MAX_PROJECT_BYTES};
use super::package_lock::{
    package_lock_state, PackageLockProviderError, ProjectPackageLockContext,
    ProjectPackageLockProvider,
};

const SNAPSHOT_SCHEMA_VERSION: u32 = 1;
const ENGINE_ID: &str = "Zircon";
const IGNORE_POLICY: &str = "zircon-project-v1";
const MAX_FILES: usize = 10_000;
const MAX_VISITED_ENTRIES: usize = 50_000;
const MAX_VISITED_DIRECTORIES: usize = 10_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LocalSnapshot {
    pub(crate) manifest: Manifest,
}

/// Scans only the native selected-project root. Project-controlled generated,
/// secret, cache, and Hub-private paths are omitted; links and special files fail closed.
pub(crate) async fn capture(
    root: &Path,
    expected_guid: ProjectGuid,
) -> Result<LocalSnapshot, &'static str> {
    let root = root.to_owned();
    let input_root = root.clone();
    let bytes =
        tokio::task::spawn_blocking(move || read_local_file(&input_root, "zircon-project.toml"))
            .await
            .map_err(|_| "hub_cloud_sync_project_changed")??;
    let summary = ProjectManifestSummary::parse_toml_bytes(&bytes)
        .map_err(|_| "hub_cloud_sync_project_unavailable")?;
    if summary.value.project_guid != Some(expected_guid) {
        return Err("hub_cloud_sync_project_changed");
    }
    let manifest_digest =
        zircon_runtime_interface::project::ProjectManifestDigest::from_bytes(&bytes);
    // Cloud source synchronization captures the authoring host's dependency target.
    // Client-runtime requests remain explicit on the shared helper consumer API.
    let lock = crate::account::package::capture_project_lock(
        &root,
        expected_guid,
        manifest_digest,
        crate::account::package::PackageRuntimeMode::EditorHost,
    )
    .await
    .map_err(|_| "hub_cloud_sync_package_lock_unavailable")?;
    let context = ProjectPackageLockContext {
        project_guid: expected_guid,
        manifest_digest,
        target: lock.target,
    };
    let provider = super::package_lock::CapturedPackageLock(lock);
    tokio::task::spawn_blocking(move || {
        capture_with_package_lock(&root, expected_guid, &context, &provider)
    })
    .await
    .map_err(|_| "hub_cloud_sync_snapshot_invalid")?
}

pub(crate) fn capture_with_package_lock(
    root: &Path,
    expected_guid: ProjectGuid,
    context: &ProjectPackageLockContext,
    provider: &dyn ProjectPackageLockProvider,
) -> Result<LocalSnapshot, &'static str> {
    if context.project_guid != expected_guid {
        return Err("hub_cloud_sync_project_changed");
    }
    verify_manifest_generation(root, context.manifest_digest)?;
    let state = package_lock_state(provider, root, context).map_err(package_lock_error)?;
    let digest = match &state {
        zircon_runtime_interface::project::ProjectPackageLockState::Present { digest, .. } => {
            digest.clone()
        }
        zircon_runtime_interface::project::ProjectPackageLockState::Unavailable { .. } => {
            return Err("hub_cloud_sync_package_lock_unavailable")
        }
    };
    let snapshot = capture_scanned(root, expected_guid, digest, Some(state))?;
    verify_manifest_generation(root, context.manifest_digest)?;
    Ok(snapshot)
}

fn verify_manifest_generation(
    root: &Path,
    expected: zircon_runtime_interface::project::ProjectManifestDigest,
) -> Result<(), &'static str> {
    let bytes = read_local_file(root, "zircon-project.toml")?;
    if zircon_runtime_interface::project::ProjectManifestDigest::from_bytes(bytes) != expected {
        return Err("hub_cloud_sync_project_changed");
    }
    Ok(())
}

fn package_lock_error(error: PackageLockProviderError) -> &'static str {
    match error {
        PackageLockProviderError::Unavailable => "hub_cloud_sync_package_lock_unavailable",
        PackageLockProviderError::ProjectChanged => "hub_cloud_sync_project_changed",
        PackageLockProviderError::TargetChanged => "hub_cloud_sync_package_lock_target_changed",
        PackageLockProviderError::Invalid => "hub_cloud_sync_package_lock_invalid",
    }
}

fn capture_scanned(
    root: &Path,
    expected_guid: ProjectGuid,
    package_lock_digest: String,
    package_lock: Option<zircon_runtime_interface::project::ProjectPackageLockState>,
) -> Result<LocalSnapshot, &'static str> {
    if package_lock_digest.len() != 64
        || !package_lock_digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err("hub_cloud_sync_package_lock_invalid");
    }
    let root = root
        .canonicalize()
        .map_err(|_| "hub_cloud_sync_project_unavailable")?;
    if !root.is_dir() {
        return Err("hub_cloud_sync_project_unavailable");
    }
    verify_project_guid(&root, expected_guid)?;

    let mut entries = Vec::new();
    let mut total_bytes = 0_u64;
    let mut budget = ScanBudget::new(MAX_VISITED_ENTRIES, MAX_VISITED_DIRECTORIES);
    scan_directory(&root, &root, &mut entries, &mut total_bytes, &mut budget)?;
    entries.sort_by(|left: &FileEntry, right: &FileEntry| left.path.cmp(&right.path));

    let manifest = Manifest {
        schema_version: SNAPSHOT_SCHEMA_VERSION,
        engine: ENGINE_ID.into(),
        package_lock_digest,
        ignore_policy: IGNORE_POLICY.into(),
        source_revision: None,
        files: entries,
        package_lock,
    }
    .normalized()
    .map_err(|_| "hub_cloud_sync_snapshot_invalid")?;
    Ok(LocalSnapshot { manifest })
}

pub(crate) fn read_blob(root: &Path, entry: &FileEntry) -> Result<Vec<u8>, &'static str> {
    let bytes = read_local_file(root, &entry.path)?;
    if bytes.len() as u64 != entry.bytes || sha256(&bytes) != entry.digest {
        return Err("hub_cloud_sync_project_changed");
    }
    Ok(bytes)
}

pub(crate) fn verify_project_guid(
    root: &Path,
    expected_guid: ProjectGuid,
) -> Result<(), &'static str> {
    let bytes = read_local_file(root, "zircon-project.toml")?;
    let summary = ProjectManifestSummary::parse_toml_bytes(&bytes)
        .map_err(|_| "hub_cloud_sync_project_unavailable")?;
    if summary.value.project_guid != Some(expected_guid) {
        return Err("hub_cloud_sync_project_changed");
    }
    Ok(())
}

fn scan_directory(
    root: &Path,
    directory: &Path,
    entries: &mut Vec<FileEntry>,
    total_bytes: &mut u64,
    budget: &mut ScanBudget,
) -> Result<(), &'static str> {
    let metadata = fs::symlink_metadata(directory).map_err(|_| "hub_cloud_sync_project_changed")?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("hub_cloud_sync_snapshot_invalid");
    }
    for child in fs::read_dir(directory).map_err(|_| "hub_cloud_sync_project_unavailable")? {
        budget.visit_entry()?;
        let child = child.map_err(|_| "hub_cloud_sync_snapshot_invalid")?;
        let name = child
            .file_name()
            .into_string()
            .map_err(|_| "hub_cloud_sync_snapshot_invalid")?;
        let path = child.path();
        let relative = path
            .strip_prefix(root)
            .map_err(|_| "hub_cloud_sync_snapshot_invalid")?
            .to_str()
            .ok_or("hub_cloud_sync_snapshot_invalid")?
            .replace('\\', "/");
        let metadata = fs::symlink_metadata(&path).map_err(|_| "hub_cloud_sync_project_changed")?;
        if metadata.is_dir() {
            budget.visit_directory()?;
        }
        // The native project GUID binds this local checkout to Hub state. It is
        // validated above, but must not become shared cloud content: another
        // teammate has a different local GUID and cannot safely apply theirs.
        if directory == root && name.eq_ignore_ascii_case("zircon-project.toml") {
            continue;
        }
        if ignored_component(&name) {
            continue;
        }
        if metadata.file_type().is_symlink() {
            return Err("hub_cloud_sync_snapshot_invalid");
        }
        if metadata.is_dir() {
            if !valid_path(&relative) {
                return Err("hub_cloud_sync_snapshot_invalid");
            }
            scan_directory(root, &path, entries, total_bytes, budget)?;
            continue;
        }
        if !metadata.is_file() || !valid_path(&relative) {
            return Err("hub_cloud_sync_snapshot_invalid");
        }
        if entries.len() >= MAX_FILES {
            return Err("hub_cloud_sync_snapshot_invalid");
        }
        let bytes = read_local_file(root, &relative)?;
        let length = bytes.len() as u64;
        *total_bytes = total_bytes
            .checked_add(length)
            .ok_or("hub_cloud_sync_snapshot_invalid")?;
        if length > MAX_BLOB_BYTES as u64 || *total_bytes > MAX_PROJECT_BYTES {
            return Err("hub_cloud_sync_snapshot_invalid");
        }
        entries.push(FileEntry {
            path: relative,
            digest: sha256(&bytes),
            bytes: length,
        });
    }
    Ok(())
}

struct ScanBudget {
    maximum_entries: usize,
    maximum_directories: usize,
    visited_entries: usize,
    visited_directories: usize,
}

impl ScanBudget {
    fn new(maximum_entries: usize, maximum_directories: usize) -> Self {
        Self {
            maximum_entries,
            maximum_directories,
            visited_entries: 0,
            visited_directories: 1,
        }
    }

    fn visit_entry(&mut self) -> Result<(), &'static str> {
        self.visited_entries = self
            .visited_entries
            .checked_add(1)
            .ok_or("hub_cloud_sync_snapshot_invalid")?;
        (self.visited_entries <= self.maximum_entries)
            .then_some(())
            .ok_or("hub_cloud_sync_snapshot_invalid")
    }

    fn visit_directory(&mut self) -> Result<(), &'static str> {
        self.visited_directories = self
            .visited_directories
            .checked_add(1)
            .ok_or("hub_cloud_sync_snapshot_invalid")?;
        (self.visited_directories <= self.maximum_directories)
            .then_some(())
            .ok_or("hub_cloud_sync_snapshot_invalid")
    }
}

fn ignored_component(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower == ".zircon"
        || matches!(
            lower.as_str(),
            ".git"
                | ".hg"
                | ".svn"
                | ".ssh"
                | ".aws"
                | ".azure"
                | ".gnupg"
                | ".codex"
                | ".env"
                | "target"
                | "build"
                | "binaries"
                | "generated"
                | "node_modules"
                | "cache"
                | "deriveddatacache"
                | "intermediate"
                | "saved"
                | "crashdumps"
                | "credentials"
                | "secrets"
        )
        || lower.starts_with(".env.")
        || [".pem", ".key", ".pfx", ".p12", ".keystore"]
            .iter()
            .any(|suffix| lower.ends_with(suffix))
        || !valid_path(name)
}

fn read_local_file(root: &Path, relative: &str) -> Result<Vec<u8>, &'static str> {
    if !valid_path(relative)
        || relative
            .split('/')
            .any(|part| part.eq_ignore_ascii_case(".zircon"))
    {
        return Err("hub_cloud_sync_snapshot_invalid");
    }
    let mut path = root.to_path_buf();
    let components = relative.split('/').collect::<Vec<_>>();
    for component in components.iter().take(components.len().saturating_sub(1)) {
        path.push(component);
        let metadata = fs::symlink_metadata(&path).map_err(|_| "hub_cloud_sync_project_changed")?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err("hub_cloud_sync_snapshot_invalid");
        }
    }
    path.push(components.last().ok_or("hub_cloud_sync_snapshot_invalid")?);
    let mut file = open_without_following_links(&path)?;
    let before = file
        .metadata()
        .map_err(|_| "hub_cloud_sync_project_changed")?;
    if !before.is_file() || before.len() > MAX_BLOB_BYTES as u64 {
        return Err("hub_cloud_sync_snapshot_invalid");
    }
    let mut bytes = Vec::with_capacity(before.len() as usize);
    (&mut file)
        .take(MAX_BLOB_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "hub_cloud_sync_project_changed")?;
    let after = file
        .metadata()
        .map_err(|_| "hub_cloud_sync_project_changed")?;
    if bytes.len() as u64 != before.len() || after.len() != before.len() {
        return Err("hub_cloud_sync_project_changed");
    }
    Ok(bytes)
}

fn open_without_following_links(path: &Path) -> Result<File, &'static str> {
    #[cfg(windows)]
    {
        use std::os::windows::{fs::MetadataExt, fs::OpenOptionsExt};
        const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)
            .map_err(|_| "hub_cloud_sync_project_changed")?;
        if file
            .metadata()
            .map_err(|_| "hub_cloud_sync_project_changed")?
            .file_attributes()
            & FILE_ATTRIBUTE_REPARSE_POINT
            != 0
        {
            return Err("hub_cloud_sync_snapshot_invalid");
        }
        Ok(file)
    }
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::OpenOptionsExt;
        const O_NOFOLLOW: i32 = 0x0002_0000;
        OpenOptions::new()
            .read(true)
            .custom_flags(O_NOFOLLOW)
            .open(path)
            .map_err(|_| "hub_cloud_sync_project_changed")
    }
    #[cfg(all(unix, not(target_os = "linux")))]
    {
        let _ = path;
        Err("hub_cloud_sync_snapshot_invalid")
    }
    #[cfg(not(any(windows, unix)))]
    {
        OpenOptions::new()
            .read(true)
            .open(path)
            .map_err(|_| "hub_cloud_sync_project_changed")
    }
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
#[path = "tests/snapshot.rs"]
mod tests;
