use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::export::ExportDigest;

use crate::core::export::{persist_bytes_atomically, ExportGenerationInventory};

const NATIVE_STAGING_MANIFEST_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::ui::host) struct NativeStagingStats {
    pub copied_files: u64,
    pub copied_bytes: u64,
    pub removed_files: u64,
    pub existing_artifact_count: usize,
}

impl NativeStagingStats {
    pub(super) fn merge(&mut self, other: Self) {
        self.copied_files = self.copied_files.saturating_add(other.copied_files);
        self.copied_bytes = self.copied_bytes.saturating_add(other.copied_bytes);
        self.removed_files = self.removed_files.saturating_add(other.removed_files);
        self.existing_artifact_count = self
            .existing_artifact_count
            .saturating_add(other.existing_artifact_count);
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct NativeStagingManifest {
    format_version: u32,
    files: Vec<NativeStagingManifestFile>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct NativeStagingManifestFile {
    relative_path: PathBuf,
    digest: ExportDigest,
    byte_count: u64,
}

struct NativeStagingSourceFile {
    source_path: PathBuf,
    relative_path: PathBuf,
    digest: ExportDigest,
    byte_count: u64,
    is_native_artifact: bool,
}

pub(super) fn sync_native_package(
    source: &Path,
    destination: &Path,
    manifest_path: &Path,
    inventory: &mut ExportGenerationInventory,
) -> std::io::Result<NativeStagingStats> {
    fs::create_dir_all(destination)?;
    let source_files = collect_native_package_files(source, inventory)?;
    let previous = load_manifest(manifest_path);
    let mut stats = NativeStagingStats {
        existing_artifact_count: source_files
            .values()
            .filter(|source| source.is_native_artifact)
            .count(),
        ..NativeStagingStats::default()
    };

    for previous_file in previous.files {
        if source_files.contains_key(&previous_file.relative_path) {
            continue;
        }
        let stale_path = destination.join(&previous_file.relative_path);
        if remove_staged_file(&stale_path)? {
            inventory.invalidate_subtree(&stale_path);
            stats.removed_files = stats.removed_files.saturating_add(1);
        }
    }

    let mut current_files = Vec::with_capacity(source_files.len());
    for source_file in source_files.into_values() {
        let destination_path = destination.join(&source_file.relative_path);
        let destination_matches = destination_path.is_file()
            && inventory
                .digest_path(&destination_path)
                .is_ok_and(|digest| digest == source_file.digest);
        if !destination_matches {
            if let Some(parent) = destination_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&source_file.source_path, &destination_path)?;
            inventory.invalidate_subtree(&destination_path);
            let copied_digest = inventory.digest_path(&destination_path)?;
            if copied_digest != source_file.digest {
                return Err(std::io::Error::other(format!(
                    "native staging copy digest mismatch: {} -> {}",
                    source_file.source_path.display(),
                    destination_path.display()
                )));
            }
            stats.copied_files = stats.copied_files.saturating_add(1);
            stats.copied_bytes = stats.copied_bytes.saturating_add(source_file.byte_count);
        }
        current_files.push(NativeStagingManifestFile {
            relative_path: source_file.relative_path,
            digest: source_file.digest,
            byte_count: source_file.byte_count,
        });
    }
    persist_manifest(
        manifest_path,
        &NativeStagingManifest {
            format_version: NATIVE_STAGING_MANIFEST_VERSION,
            files: current_files,
        },
    )?;
    prune_empty_child_directories(destination)?;
    Ok(stats)
}

pub(super) fn prune_stale_packages(
    packages_root: &Path,
    manifests_root: &Path,
    active_packages: &BTreeSet<String>,
) -> std::io::Result<NativeStagingStats> {
    let mut stats = NativeStagingStats::default();
    for root in [packages_root, manifests_root] {
        if !root.is_dir() {
            continue;
        }
        for entry in fs::read_dir(root)? {
            let entry = entry?;
            let path = entry.path();
            let package_name = if root == manifests_root {
                path.file_stem()
            } else {
                path.file_name()
            }
            .and_then(|name| name.to_str());
            if package_name.is_some_and(|name| active_packages.contains(name)) {
                continue;
            }
            stats.removed_files = stats
                .removed_files
                .saturating_add(remove_path_and_count_files(&path)?);
        }
    }
    Ok(stats)
}

fn collect_native_package_files(
    package_root: &Path,
    inventory: &mut ExportGenerationInventory,
) -> std::io::Result<BTreeMap<PathBuf, NativeStagingSourceFile>> {
    let mut files = BTreeMap::new();
    let plugin_manifest = package_root.join("plugin.toml");
    if plugin_manifest.is_file() {
        insert_source_file(package_root, &plugin_manifest, false, inventory, &mut files)?;
    }
    for directory in ["assets", "asset", "resources", "resource"] {
        let path = package_root.join(directory);
        if path.is_dir() {
            collect_directory_files(package_root, &path, inventory, &mut files)?;
        }
    }
    let native_root = package_root.join("native");
    if native_root.is_dir() {
        for entry in fs::read_dir(&native_root)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let path = entry.path();
            if file_type.is_file() && is_native_dynamic_artifact(&path) {
                insert_source_file(package_root, &path, true, inventory, &mut files)?;
            }
        }
    }
    Ok(files)
}

fn collect_directory_files(
    package_root: &Path,
    directory: &Path,
    inventory: &mut ExportGenerationInventory,
    files: &mut BTreeMap<PathBuf, NativeStagingSourceFile>,
) -> std::io::Result<()> {
    let mut entries = fs::read_dir(directory)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let file_type = entry.file_type()?;
        let path = entry.path();
        if file_type.is_dir() {
            collect_directory_files(package_root, &path, inventory, files)?;
        } else if file_type.is_file() {
            insert_source_file(package_root, &path, false, inventory, files)?;
        }
    }
    Ok(())
}

fn insert_source_file(
    package_root: &Path,
    source_path: &Path,
    is_native_artifact: bool,
    inventory: &mut ExportGenerationInventory,
    files: &mut BTreeMap<PathBuf, NativeStagingSourceFile>,
) -> std::io::Result<()> {
    let relative_path = source_path
        .strip_prefix(package_root)
        .map_err(std::io::Error::other)?
        .to_path_buf();
    files.insert(
        relative_path.clone(),
        NativeStagingSourceFile {
            source_path: source_path.to_path_buf(),
            relative_path,
            digest: inventory.digest_path(source_path)?,
            byte_count: fs::metadata(source_path)?.len(),
            is_native_artifact,
        },
    );
    Ok(())
}

fn load_manifest(path: &Path) -> NativeStagingManifest {
    let Ok(bytes) = fs::read(path) else {
        return NativeStagingManifest::default();
    };
    let Ok(manifest) = serde_json::from_slice::<NativeStagingManifest>(&bytes) else {
        return NativeStagingManifest::default();
    };
    if manifest.format_version == NATIVE_STAGING_MANIFEST_VERSION {
        manifest
    } else {
        NativeStagingManifest::default()
    }
}

fn persist_manifest(path: &Path, manifest: &NativeStagingManifest) -> std::io::Result<()> {
    let encoded = serde_json::to_vec_pretty(manifest).map_err(std::io::Error::other)?;
    persist_bytes_atomically(path, &encoded)
}

fn remove_staged_file(path: &Path) -> std::io::Result<bool> {
    match fs::remove_file(path) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn prune_empty_child_directories(root: &Path) -> std::io::Result<()> {
    if !root.is_dir() {
        return Ok(());
    }
    let mut entries = fs::read_dir(root)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        if entry.file_type()?.is_dir() {
            prune_empty_directory_tree(&entry.path())?;
        }
    }
    Ok(())
}

fn prune_empty_directory_tree(root: &Path) -> std::io::Result<bool> {
    if !root.is_dir() {
        return Ok(false);
    }
    let mut entries = fs::read_dir(root)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        if entry.file_type()?.is_dir() {
            prune_empty_directory_tree(&entry.path())?;
        }
    }
    if fs::read_dir(root)?.next().is_none() {
        fs::remove_dir(root)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn remove_path_and_count_files(path: &Path) -> std::io::Result<u64> {
    if path.is_dir() {
        let count = count_files(path)?;
        fs::remove_dir_all(path)?;
        Ok(count)
    } else if path.is_file() {
        fs::remove_file(path)?;
        Ok(1)
    } else {
        Ok(0)
    }
}

fn count_files(root: &Path) -> std::io::Result<u64> {
    let mut count = 0_u64;
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            count = count.saturating_add(count_files(&entry.path())?);
        } else {
            count = count.saturating_add(1);
        }
    }
    Ok(count)
}

pub(super) fn is_native_dynamic_artifact(path: &Path) -> bool {
    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return false;
    };
    matches!(
        extension.to_ascii_lowercase().as_str(),
        "dll" | "so" | "dylib" | "pdb" | "dbg" | "dsym"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unchanged_warm_staging_copies_zero_files_and_bytes() {
        let fixture = StagingFixture::new();
        fixture.write_source("plugin.toml", b"id = 'fixture'");
        fixture.write_source("assets/scene.bin", b"scene");

        let first = fixture.sync();
        let second = fixture.sync();

        assert_eq!(first.copied_files, 2);
        assert!(first.copied_bytes > 0);
        assert_eq!(second.copied_files, 0);
        assert_eq!(second.copied_bytes, 0);
        assert_eq!(second.removed_files, 0);
    }

    #[test]
    fn changed_deleted_and_renamed_sources_update_the_staging_tree() {
        let fixture = StagingFixture::new();
        fixture.write_source("plugin.toml", b"id = 'fixture'");
        fixture.write_source("assets/changed.bin", b"before!!");
        fixture.write_source("assets/deleted.bin", b"deleted");
        fixture.write_source("assets/old-name.bin", b"renamed");
        fixture.sync();

        std::thread::sleep(std::time::Duration::from_millis(2));
        fixture.write_source("assets/changed.bin", b"after!!!");
        fs::remove_file(fixture.source.join("assets/deleted.bin")).unwrap();
        fs::rename(
            fixture.source.join("assets/old-name.bin"),
            fixture.source.join("assets/new-name.bin"),
        )
        .unwrap();
        let stats = fixture.sync();

        assert_eq!(stats.copied_files, 2);
        assert_eq!(stats.removed_files, 2);
        assert_eq!(
            fs::read(fixture.destination.join("assets/changed.bin")).unwrap(),
            b"after!!!"
        );
        assert!(!fixture.destination.join("assets/deleted.bin").exists());
        assert!(!fixture.destination.join("assets/old-name.bin").exists());
        assert_eq!(
            fs::read(fixture.destination.join("assets/new-name.bin")).unwrap(),
            b"renamed"
        );
    }

    struct StagingFixture {
        root: PathBuf,
        source: PathBuf,
        destination: PathBuf,
        manifest: PathBuf,
        inventory_cache: PathBuf,
    }

    impl StagingFixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "zircon-editor-native-staging-{}-{:x}",
                std::process::id(),
                fixture_nonce()
            ));
            let _ = fs::remove_dir_all(&root);
            let source = root.join("source");
            let destination = root.join("destination");
            fs::create_dir_all(&source).unwrap();
            Self {
                source,
                destination,
                manifest: root.join("manifests/fixture.json"),
                inventory_cache: root.join("inventory.json"),
                root,
            }
        }

        fn write_source(&self, relative: &str, bytes: &[u8]) {
            let path = self.source.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, bytes).unwrap();
        }

        fn sync(&self) -> NativeStagingStats {
            let mut inventory =
                ExportGenerationInventory::with_persistent_cache(self.inventory_cache.clone());
            sync_native_package(
                &self.source,
                &self.destination,
                &self.manifest,
                &mut inventory,
            )
            .unwrap()
        }
    }

    impl Drop for StagingFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn fixture_nonce() -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::thread::current().id().hash(&mut hasher);
        std::time::SystemTime::now().hash(&mut hasher);
        hasher.finish()
    }
}
