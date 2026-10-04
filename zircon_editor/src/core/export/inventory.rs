use std::collections::{BTreeMap, HashSet};
use std::ffi::OsStr;
use std::fmt::Write as _;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::export::{ExportArtifactRef, ExportDigest};

const PERSISTENT_INVENTORY_FORMAT_VERSION: u32 = 1;

#[derive(Default)]
pub(crate) struct ExportGenerationInventory {
    digests_by_canonical_path: BTreeMap<PathBuf, ExportDigest>,
    visiting_directories: HashSet<PathBuf>,
    seen_file_paths: HashSet<PathBuf>,
    persistent_cache_path: Option<PathBuf>,
    persistent_file_digests: BTreeMap<PathBuf, PersistedFileDigest>,
    persistent_tool_identities: BTreeMap<String, PersistedToolIdentity>,
    generation_tool_identities: BTreeMap<String, PersistedToolIdentity>,
    persistent_cache_dirty: bool,
    #[cfg(test)]
    file_reads: usize,
    #[cfg(test)]
    file_bytes_read: u64,
    #[cfg(test)]
    file_hashes: usize,
    #[cfg(test)]
    tool_probes: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistentInventoryCache {
    format_version: u32,
    files: Vec<PersistedFileDigest>,
    tools: Vec<PersistedToolIdentity>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PersistedFileDigest {
    canonical_path: PathBuf,
    identity: FileMetadataIdentity,
    digest: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FileMetadataIdentity {
    size: u64,
    modified_marker: String,
    change_marker: String,
    file_identity: String,
    cacheable: bool,
}

impl FileMetadataIdentity {
    pub(crate) const fn is_cacheable(&self) -> bool {
        self.cacheable
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct PersistedToolIdentity {
    cache_key: String,
    artifact_key: String,
    locator: String,
    digest: [u8; 32],
}

impl ExportGenerationInventory {
    pub(crate) fn with_persistent_cache(path: PathBuf) -> Self {
        let mut inventory = Self::default();
        inventory.persistent_cache_path = Some(path.clone());
        let Ok(bytes) = fs::read(path) else {
            return inventory;
        };
        let Ok(cache) = serde_json::from_slice::<PersistentInventoryCache>(&bytes) else {
            return inventory;
        };
        if cache.format_version != PERSISTENT_INVENTORY_FORMAT_VERSION {
            return inventory;
        }
        inventory.persistent_file_digests = cache
            .files
            .into_iter()
            .map(|record| (record.canonical_path.clone(), record))
            .collect();
        inventory.persistent_tool_identities = cache
            .tools
            .into_iter()
            .map(|record| (record.cache_key.clone(), record))
            .collect();
        inventory
    }

    pub(crate) fn artifact_with_current_digest(
        &mut self,
        key: impl Into<String>,
        path: &Path,
    ) -> std::io::Result<ExportArtifactRef> {
        Ok(ExportArtifactRef::new(key, path.display().to_string())
            .with_digest(self.digest_path(path)?))
    }

    pub(crate) fn artifact_with_optional_digest(
        &mut self,
        key: impl Into<String>,
        path: &Path,
    ) -> std::io::Result<ExportArtifactRef> {
        let digest = if path.exists() {
            self.digest_path(path)?
        } else {
            ExportDigest::from_bytes(*blake3::hash(b"<missing>").as_bytes())
        };
        Ok(ExportArtifactRef::new(key, path.display().to_string()).with_digest(digest))
    }

    pub(crate) fn artifact_matches_disk(&mut self, artifact: &ExportArtifactRef) -> bool {
        artifact
            .digest
            .and_then(|expected| {
                self.digest_path(Path::new(&artifact.locator))
                    .ok()
                    .map(|actual| (expected, actual))
            })
            .is_some_and(|(expected, actual)| expected == actual)
    }

    pub(crate) fn tool_identity(
        &mut self,
        key: &str,
        program: &OsStr,
        version_args: &[&str],
    ) -> std::io::Result<ExportArtifactRef> {
        self.tool_identity_with_probe(key, program, version_args, || {
            probe_tool_version(program, version_args)
        })
    }

    fn tool_identity_with_probe(
        &mut self,
        key: &str,
        program: &OsStr,
        version_args: &[&str],
        probe: impl FnOnce() -> std::io::Result<Vec<u8>>,
    ) -> std::io::Result<ExportArtifactRef> {
        let cache_key = tool_cache_key(key, program, version_args);
        if let Some(identity) = self.generation_tool_identities.get(&cache_key) {
            return Ok(tool_artifact(identity));
        }

        let bytes = probe()?;
        #[cfg(test)]
        {
            self.tool_probes += 1;
        }
        let identity = PersistedToolIdentity {
            cache_key: cache_key.clone(),
            artifact_key: format!("{key}_toolchain"),
            locator: program.to_string_lossy().into_owned(),
            digest: *blake3::hash(&bytes).as_bytes(),
        };
        self.generation_tool_identities
            .insert(cache_key.clone(), identity.clone());
        if self.persistent_tool_identities.get(&cache_key) != Some(&identity) {
            self.persistent_tool_identities
                .insert(cache_key, identity.clone());
            self.persistent_cache_dirty = true;
        }
        Ok(tool_artifact(&identity))
    }

    pub(crate) fn digest_path(&mut self, path: &Path) -> std::io::Result<ExportDigest> {
        let canonical_path = fs::canonicalize(path)?;
        self.digest_canonical_path(&canonical_path)
    }

    pub(crate) fn invalidate_subtree(&mut self, path: &Path) {
        let canonical_path = canonical_identity(path);
        self.digests_by_canonical_path.retain(|cached_path, _| {
            !cached_path.starts_with(&canonical_path) && !canonical_path.starts_with(cached_path)
        });
        self.seen_file_paths.retain(|cached_path| {
            !cached_path.starts_with(&canonical_path) && !canonical_path.starts_with(cached_path)
        });
        let previous_len = self.persistent_file_digests.len();
        self.persistent_file_digests.retain(|cached_path, _| {
            !cached_path.starts_with(&canonical_path) && !canonical_path.starts_with(cached_path)
        });
        self.persistent_cache_dirty |= previous_len != self.persistent_file_digests.len();
    }

    fn digest_canonical_path(&mut self, path: &Path) -> std::io::Result<ExportDigest> {
        if let Some(digest) = self.digests_by_canonical_path.get(path) {
            return Ok(*digest);
        }

        let metadata = fs::metadata(path)?;
        let digest = if metadata.is_file() {
            self.digest_file(path)?
        } else if metadata.is_dir() {
            self.digest_directory(path)?
        } else {
            return Err(std::io::Error::other(format!(
                "export artifact is neither a file nor directory: {}",
                path.display()
            )));
        };
        self.digests_by_canonical_path
            .insert(path.to_path_buf(), digest);
        Ok(digest)
    }

    fn digest_file(&mut self, path: &Path) -> std::io::Result<ExportDigest> {
        let identity = file_metadata_identity(path)?;
        self.seen_file_paths.insert(path.to_path_buf());
        if let Some(cached) = self
            .persistent_file_digests
            .get(path)
            .filter(|cached| identity.cacheable && cached.identity == identity)
        {
            return Ok(ExportDigest::from_bytes(cached.digest));
        }

        for _ in 0..2 {
            let identity_before = file_metadata_identity(path)?;
            let mut file = File::open(path)?;
            let mut hasher = blake3::Hasher::new();
            hasher.update(&[1]);
            hasher.update(&identity_before.size.to_le_bytes());
            let mut bytes_read = 0_u64;
            let mut chunk = [0_u8; 64 * 1024];
            loop {
                let read = file.read(&mut chunk)?;
                if read == 0 {
                    break;
                }
                hasher.update(&chunk[..read]);
                bytes_read = bytes_read.saturating_add(read as u64);
            }
            #[cfg(test)]
            {
                self.file_reads += 1;
                self.file_bytes_read = self.file_bytes_read.saturating_add(bytes_read);
            }
            let identity_after = file_metadata_identity(path)?;
            if identity_before != identity_after || bytes_read != identity_after.size {
                continue;
            }
            #[cfg(test)]
            {
                self.file_hashes += 1;
            }
            let digest = ExportDigest::from_bytes(*hasher.finalize().as_bytes());
            if identity_after.cacheable {
                self.persistent_file_digests.insert(
                    path.to_path_buf(),
                    PersistedFileDigest {
                        canonical_path: path.to_path_buf(),
                        identity: identity_after,
                        digest: *digest.as_bytes(),
                    },
                );
                self.persistent_cache_dirty = true;
            } else if self.persistent_file_digests.remove(path).is_some() {
                self.persistent_cache_dirty = true;
            }
            return Ok(digest);
        }
        Err(std::io::Error::other(format!(
            "export artifact changed while it was being fingerprinted: {}",
            path.display()
        )))
    }

    fn digest_directory(&mut self, path: &Path) -> std::io::Result<ExportDigest> {
        let is_generation_root = self.visiting_directories.is_empty();
        if !self.visiting_directories.insert(path.to_path_buf()) {
            return Err(std::io::Error::other(format!(
                "export artifact directory cycle: {}",
                path.display()
            )));
        }

        let digest = self.digest_directory_children(path);
        if digest.is_ok() && is_generation_root {
            self.prune_unseen_persistent_files(path);
        }
        self.visiting_directories.remove(path);
        digest
    }

    fn prune_unseen_persistent_files(&mut self, root: &Path) {
        let previous_len = self.persistent_file_digests.len();
        self.persistent_file_digests.retain(|cached_path, _| {
            !cached_path.starts_with(root) || self.seen_file_paths.contains(cached_path)
        });
        self.persistent_cache_dirty |= previous_len != self.persistent_file_digests.len();
    }

    fn digest_directory_children(&mut self, path: &Path) -> std::io::Result<ExportDigest> {
        let mut children = fs::read_dir(path)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, _>>()?;
        children.sort();

        let mut hasher = blake3::Hasher::new();
        hasher.update(&[2]);
        hash_usize(&mut hasher, children.len());
        for child in children {
            let name = child.file_name().ok_or_else(|| {
                std::io::Error::other(format!(
                    "export artifact child has no file name: {}",
                    child.display()
                ))
            })?;
            hash_os_string(&mut hasher, name);
            let canonical_child = fs::canonicalize(&child)?;
            let child_digest = self.digest_canonical_path(&canonical_child)?;
            hasher.update(child_digest.as_bytes());
        }
        Ok(ExportDigest::from_bytes(*hasher.finalize().as_bytes()))
    }

    fn persist_cache(&self) -> std::io::Result<()> {
        if !self.persistent_cache_dirty {
            return Ok(());
        }
        let Some(path) = self.persistent_cache_path.as_ref() else {
            return Ok(());
        };
        let cache = PersistentInventoryCache {
            format_version: PERSISTENT_INVENTORY_FORMAT_VERSION,
            files: self.persistent_file_digests.values().cloned().collect(),
            tools: self.persistent_tool_identities.values().cloned().collect(),
        };
        let encoded = serde_json::to_vec_pretty(&cache).map_err(std::io::Error::other)?;
        persist_bytes_atomically(path, &encoded)
    }
}

impl Drop for ExportGenerationInventory {
    fn drop(&mut self) {
        let _ = self.persist_cache();
    }
}

pub(crate) fn persist_bytes_atomically(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let staging = path.with_extension(format!(
        "tmp-{}-{:x}",
        std::process::id(),
        cache_write_nonce()
    ));
    let mut file = File::create(&staging)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    replace_cache_file(&staging, path)
}

fn tool_cache_key(key: &str, program: &OsStr, version_args: &[&str]) -> String {
    let capacity = version_args.iter().fold(
        key.len()
            .saturating_add(program.len())
            .saturating_add(4)
            .saturating_add(version_args.len().saturating_sub(1)),
        |capacity, argument| capacity.saturating_add(argument.len()),
    );
    let mut cache_key = String::with_capacity(capacity);
    write!(&mut cache_key, "{key}\0{program:?}\0").expect("writing to a String cannot fail");
    for (index, argument) in version_args.iter().enumerate() {
        if index > 0 {
            cache_key.push('\0');
        }
        cache_key.push_str(argument);
    }
    cache_key
}

fn probe_tool_version(program: &OsStr, version_args: &[&str]) -> std::io::Result<Vec<u8>> {
    let output = Command::new(program).args(version_args).output()?;
    if !output.status.success() {
        return Err(std::io::Error::other(format!(
            "tool {:?} {:?} exited with {:?}",
            program,
            version_args,
            output.status.code()
        )));
    }
    let mut bytes = output.stdout;
    bytes.extend(output.stderr);
    Ok(bytes)
}

fn tool_artifact(identity: &PersistedToolIdentity) -> ExportArtifactRef {
    ExportArtifactRef::new(&identity.artifact_key, &identity.locator)
        .with_digest(ExportDigest::from_bytes(identity.digest))
}

#[cfg(windows)]
pub(crate) fn file_metadata_identity(path: &Path) -> std::io::Result<FileMetadataIdentity> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        FileBasicInfo, FileIdInfo, GetFileInformationByHandleEx, FILE_BASIC_INFO, FILE_ID_INFO,
    };

    let file = File::open(path)?;
    let metadata = file.metadata()?;
    let handle = file.as_raw_handle() as _;
    let mut basic = FILE_BASIC_INFO::default();
    // Both records come from the same open handle, so replacement races cannot mix identities.
    let basic_result = unsafe {
        GetFileInformationByHandleEx(
            handle,
            FileBasicInfo,
            (&mut basic as *mut FILE_BASIC_INFO).cast(),
            std::mem::size_of::<FILE_BASIC_INFO>() as u32,
        )
    };
    if basic_result == 0 {
        return Ok(untrusted_file_metadata_identity(&metadata));
    }
    let mut file_id = FILE_ID_INFO::default();
    let id_result = unsafe {
        GetFileInformationByHandleEx(
            handle,
            FileIdInfo,
            (&mut file_id as *mut FILE_ID_INFO).cast(),
            std::mem::size_of::<FILE_ID_INFO>() as u32,
        )
    };
    if id_result == 0 {
        return Ok(FileMetadataIdentity {
            size: metadata.len(),
            modified_marker: basic.LastWriteTime.to_string(),
            change_marker: format!("{}:{}", basic.CreationTime, basic.ChangeTime),
            file_identity: "unavailable".to_string(),
            cacheable: false,
        });
    }
    Ok(FileMetadataIdentity {
        size: metadata.len(),
        modified_marker: basic.LastWriteTime.to_string(),
        change_marker: format!("{}:{}", basic.CreationTime, basic.ChangeTime),
        file_identity: format!(
            "{:016x}:{}",
            file_id.VolumeSerialNumber,
            hex_bytes(&file_id.FileId.Identifier)
        ),
        cacheable: true,
    })
}

#[cfg(windows)]
fn untrusted_file_metadata_identity(metadata: &std::fs::Metadata) -> FileMetadataIdentity {
    FileMetadataIdentity {
        size: metadata.len(),
        modified_marker: format!("{:?}", metadata.modified().ok()),
        change_marker: "unavailable".to_string(),
        file_identity: "unavailable".to_string(),
        cacheable: false,
    }
}

#[cfg(not(windows))]
pub(crate) fn file_metadata_identity(path: &Path) -> std::io::Result<FileMetadataIdentity> {
    use std::os::unix::fs::MetadataExt;

    let metadata = File::open(path)?.metadata()?;
    Ok(FileMetadataIdentity {
        size: metadata.len(),
        modified_marker: format!("{}:{}", metadata.mtime(), metadata.mtime_nsec()),
        change_marker: format!("{}:{}", metadata.ctime(), metadata.ctime_nsec()),
        file_identity: format!("{}:{}", metadata.dev(), metadata.ino()),
        cacheable: true,
    })
}

#[cfg(windows)]
fn hex_bytes(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut encoded, "{byte:02x}");
    }
    encoded
}

#[cfg(windows)]
fn replace_cache_file(staging: &Path, destination: &Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };

    let staging = staging
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let destination = destination
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let result = unsafe {
        MoveFileExW(
            staging.as_ptr(),
            destination.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if result == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn replace_cache_file(staging: &Path, destination: &Path) -> std::io::Result<()> {
    fs::rename(staging, destination)
}

fn cache_write_nonce() -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::thread::current().id().hash(&mut hasher);
    std::time::SystemTime::now().hash(&mut hasher);
    hasher.finish()
}

fn canonical_identity(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map(|root| root.join(path))
                .unwrap_or_else(|_| path.to_path_buf())
        }
    })
}

fn hash_usize(hasher: &mut blake3::Hasher, value: usize) {
    hasher.update(&(value as u64).to_le_bytes());
}

fn hash_bytes(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hash_usize(hasher, bytes.len());
    hasher.update(bytes);
}

#[cfg(windows)]
fn hash_os_string(hasher: &mut blake3::Hasher, value: &std::ffi::OsStr) {
    use std::os::windows::ffi::OsStrExt;
    for unit in value.encode_wide() {
        hasher.update(&unit.to_le_bytes());
    }
    hasher.update(&[0, 0]);
}

#[cfg(not(windows))]
fn hash_os_string(hasher: &mut blake3::Hasher, value: &std::ffi::OsStr) {
    use std::os::unix::ffi::OsStrExt;
    hash_bytes(hasher, value.as_bytes());
}

#[cfg(test)]
#[path = "inventory/tests/hash_membership_tests.rs"]
mod hash_membership_tests;

#[cfg(test)]
#[path = "inventory/tests/tool_cache_key_tests.rs"]
mod tool_cache_key_tests;

#[cfg(test)]
#[path = "tests/inventory.rs"]
mod tests;
