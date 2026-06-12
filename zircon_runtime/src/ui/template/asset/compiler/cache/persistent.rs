use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::template::{
    UiAssetFingerprint, UiCompileCacheKey, UI_COMPILED_ASSET_BINARY_ARTIFACT_SCHEMA_VERSION,
    UI_COMPILED_ASSET_COMPILER_SCHEMA_VERSION, UI_COMPILED_ASSET_PACKAGE_SCHEMA_VERSION,
};

use super::super::package::UiRuntimeCompiledAssetArtifact;

const STORE_RECORD_VERSION: u32 = 1;
const STORE_ARTIFACT_EXTENSION: &str = "zuiart";
const STORE_PAYLOAD_EXTENSION: &str = "zuicache";
const MAX_ASSET_STEM_LEN: usize = 80;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UiCompiledArtifactKey {
    pub asset_id: String,
    pub fingerprint: u64,
    pub schema_version: u32,
    pub compiler_version: u32,
}

impl UiCompiledArtifactKey {
    pub fn new(
        asset_id: impl Into<String>,
        fingerprint: u64,
        schema_version: u32,
        compiler_version: u32,
    ) -> Self {
        Self {
            asset_id: asset_id.into(),
            fingerprint,
            schema_version,
            compiler_version,
        }
    }

    pub fn from_compile_cache_key(
        asset_id: impl Into<String>,
        cache_key: &UiCompileCacheKey,
    ) -> Self {
        Self::from_compile_cache_key_with_versions(
            asset_id,
            cache_key,
            UI_COMPILED_ASSET_BINARY_ARTIFACT_SCHEMA_VERSION,
            UI_COMPILED_ASSET_COMPILER_SCHEMA_VERSION,
        )
    }

    pub fn from_compile_cache_key_with_versions(
        asset_id: impl Into<String>,
        cache_key: &UiCompileCacheKey,
        schema_version: u32,
        compiler_version: u32,
    ) -> Self {
        Self::new(
            asset_id,
            Self::fingerprint_compile_cache_key(cache_key),
            schema_version,
            compiler_version,
        )
    }

    pub fn from_artifact(artifact: &UiRuntimeCompiledAssetArtifact) -> Self {
        let header = &artifact.report.header;
        Self::from_compile_cache_key_with_versions(
            header.asset.id.clone(),
            &header.compile_cache_key,
            UI_COMPILED_ASSET_BINARY_ARTIFACT_SCHEMA_VERSION,
            header.compiler_schema_version,
        )
    }

    pub fn fingerprint_compile_cache_key(cache_key: &UiCompileCacheKey) -> u64 {
        let mut bytes = Vec::new();
        push_fingerprint(&mut bytes, cache_key.root_document);
        push_fingerprint_map(&mut bytes, &cache_key.widget_imports);
        push_fingerprint_map(&mut bytes, &cache_key.style_imports);
        push_fingerprint(&mut bytes, cache_key.declared_widget_imports_revision);
        push_fingerprint(&mut bytes, cache_key.declared_style_imports_revision);
        push_u64(&mut bytes, cache_key.descriptor_registry_revision);
        push_fingerprint(&mut bytes, cache_key.component_contract_revision);
        push_fingerprint(&mut bytes, cache_key.resource_dependencies_revision);
        UiAssetFingerprint::from_bytes(&bytes).value
    }
}

#[derive(Clone, Debug)]
pub struct UiCompiledArtifactStore {
    root: PathBuf,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UiCompiledArtifactStoreEvictionReport {
    pub files_removed: usize,
    pub bytes_removed: u64,
}

impl UiCompiledArtifactStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn artifact_path(&self, key: &UiCompiledArtifactKey) -> PathBuf {
        self.path_for_key(key, STORE_ARTIFACT_EXTENSION)
    }

    pub fn load(
        &self,
        key: &UiCompiledArtifactKey,
    ) -> io::Result<Option<UiRuntimeCompiledAssetArtifact>> {
        let Some(artifact_bytes) = self.load_bytes(key)? else {
            return Ok(None);
        };
        Ok(UiRuntimeCompiledAssetArtifact::from_bytes(&artifact_bytes).ok())
    }

    pub fn load_bytes(&self, key: &UiCompiledArtifactKey) -> io::Result<Option<Vec<u8>>> {
        let payload = match fs::read(self.artifact_path(key)) {
            Ok(payload) => payload,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        let record = match bincode::deserialize::<UiCompiledArtifactDiskRecord>(&payload) {
            Ok(record) => record,
            Err(_) => return Ok(None),
        };
        if record.record_version != STORE_RECORD_VERSION || record.key != *key {
            return Ok(None);
        }
        let artifact = match UiRuntimeCompiledAssetArtifact::from_bytes(&record.artifact_bytes) {
            Ok(artifact) => artifact,
            Err(_) => return Ok(None),
        };
        if !artifact_matches_key(key, &artifact) {
            return Ok(None);
        }
        Ok(Some(record.artifact_bytes))
    }

    pub fn load_payload_bytes(&self, key: &UiCompiledArtifactKey) -> io::Result<Option<Vec<u8>>> {
        let payload = match fs::read(self.payload_path(key)) {
            Ok(payload) => payload,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        let record = match bincode::deserialize::<UiCompiledPayloadDiskRecord>(&payload) {
            Ok(record) => record,
            Err(_) => return Ok(None),
        };
        if record.record_version != STORE_RECORD_VERSION || record.key != *key {
            return Ok(None);
        }
        Ok(Some(record.payload_bytes))
    }

    pub fn store(
        &self,
        key: &UiCompiledArtifactKey,
        artifact: &UiRuntimeCompiledAssetArtifact,
    ) -> io::Result<PathBuf> {
        let artifact_bytes = artifact.to_bytes().map_err(invalid_data)?;
        self.store_bytes(key, &artifact_bytes)
    }

    pub fn store_bytes(
        &self,
        key: &UiCompiledArtifactKey,
        artifact_bytes: &[u8],
    ) -> io::Result<PathBuf> {
        let artifact =
            UiRuntimeCompiledAssetArtifact::from_bytes(artifact_bytes).map_err(invalid_data)?;
        if !artifact_matches_key(key, &artifact) {
            return Err(invalid_data(format!(
                "compiled artifact does not match persistent cache key for {}",
                key.asset_id
            )));
        }

        let path = self.artifact_path(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let record = UiCompiledArtifactDiskRecord {
            record_version: STORE_RECORD_VERSION,
            key: key.clone(),
            artifact_bytes: artifact_bytes.to_vec(),
        };
        let payload = bincode::serialize(&record).map_err(invalid_data)?;
        fs::write(&path, payload)?;
        Ok(path)
    }

    pub fn store_payload_bytes(
        &self,
        key: &UiCompiledArtifactKey,
        payload_bytes: &[u8],
    ) -> io::Result<PathBuf> {
        let path = self.payload_path(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let record = UiCompiledPayloadDiskRecord {
            record_version: STORE_RECORD_VERSION,
            key: key.clone(),
            payload_bytes: payload_bytes.to_vec(),
        };
        let payload = bincode::serialize(&record).map_err(invalid_data)?;
        fs::write(&path, payload)?;
        Ok(path)
    }

    pub fn remove(&self, key: &UiCompiledArtifactKey) -> io::Result<bool> {
        match fs::remove_file(self.artifact_path(key)) {
            Ok(()) => Ok(true),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }

    pub fn evict_asset(&self, asset_id: &str) -> io::Result<UiCompiledArtifactStoreEvictionReport> {
        let mut report = UiCompiledArtifactStoreEvictionReport::default();
        self.evict_asset_in_dir(&self.root, asset_id, &mut report)?;
        Ok(report)
    }

    fn evict_asset_in_dir(
        &self,
        directory: &Path,
        asset_id: &str,
        report: &mut UiCompiledArtifactStoreEvictionReport,
    ) -> io::Result<()> {
        let entries = match fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error),
        };

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry.metadata()?;
            if metadata.is_dir() {
                self.evict_asset_in_dir(&path, asset_id, report)?;
                continue;
            }
            if path.extension().and_then(|extension| extension.to_str())
                != Some(STORE_ARTIFACT_EXTENSION)
                && path.extension().and_then(|extension| extension.to_str())
                    != Some(STORE_PAYLOAD_EXTENSION)
            {
                continue;
            }
            let Ok(payload) = fs::read(&path) else {
                continue;
            };
            if !disk_payload_matches_asset_id(&payload, asset_id) {
                continue;
            }
            fs::remove_file(&path)?;
            report.files_removed += 1;
            report.bytes_removed += metadata.len();
        }
        Ok(())
    }

    fn payload_path(&self, key: &UiCompiledArtifactKey) -> PathBuf {
        self.path_for_key(key, STORE_PAYLOAD_EXTENSION)
    }

    fn path_for_key(&self, key: &UiCompiledArtifactKey, extension: &str) -> PathBuf {
        let asset_stem = sanitized_asset_file_stem(&key.asset_id);
        let asset_hash = UiAssetFingerprint::from_bytes(key.asset_id.as_bytes()).value;
        self.root
            .join(format!("schema-{:08x}", key.schema_version))
            .join(format!("compiler-{:08x}", key.compiler_version))
            .join(format!("{:016x}", key.fingerprint))
            .join(format!("{asset_stem}-{asset_hash:016x}.{extension}"))
    }
}

fn disk_payload_matches_asset_id(payload: &[u8], asset_id: &str) -> bool {
    bincode::deserialize::<UiCompiledArtifactDiskRecord>(payload)
        .map(|record| record.key.asset_id == asset_id)
        .unwrap_or_else(|_| {
            bincode::deserialize::<UiCompiledPayloadDiskRecord>(payload)
                .map(|record| record.key.asset_id == asset_id)
                .unwrap_or(false)
        })
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct UiCompiledArtifactDiskRecord {
    record_version: u32,
    key: UiCompiledArtifactKey,
    artifact_bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct UiCompiledPayloadDiskRecord {
    record_version: u32,
    key: UiCompiledArtifactKey,
    payload_bytes: Vec<u8>,
}

fn artifact_matches_key(
    key: &UiCompiledArtifactKey,
    artifact: &UiRuntimeCompiledAssetArtifact,
) -> bool {
    let header = &artifact.report.header;
    key.schema_version == UI_COMPILED_ASSET_BINARY_ARTIFACT_SCHEMA_VERSION
        && header.asset.id == key.asset_id
        && header.compiler_schema_version == key.compiler_version
        && header.package_schema_version == UI_COMPILED_ASSET_PACKAGE_SCHEMA_VERSION
        && UiCompiledArtifactKey::fingerprint_compile_cache_key(&header.compile_cache_key)
            == key.fingerprint
}

fn sanitized_asset_file_stem(asset_id: &str) -> String {
    let mut stem = asset_id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else if matches!(character, '.' | '_' | '-') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    if stem.is_empty() {
        stem.push_str("asset");
    }
    if stem.len() > MAX_ASSET_STEM_LEN {
        stem.truncate(MAX_ASSET_STEM_LEN);
    }
    stem
}

fn push_fingerprint(bytes: &mut Vec<u8>, fingerprint: UiAssetFingerprint) {
    push_u64(bytes, fingerprint.value);
}

fn push_fingerprint_map(
    bytes: &mut Vec<u8>,
    fingerprints: &std::collections::BTreeMap<String, UiAssetFingerprint>,
) {
    push_u64(bytes, fingerprints.len() as u64);
    for (reference, fingerprint) in fingerprints {
        push_str(bytes, reference);
        push_fingerprint(bytes, *fingerprint);
    }
}

fn push_str(bytes: &mut Vec<u8>, value: &str) {
    push_u64(bytes, value.len() as u64);
    bytes.extend_from_slice(value.as_bytes());
}

fn push_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn invalid_data(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}
