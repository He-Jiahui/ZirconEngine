use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::asset::project::manager::durable_transaction::PreparedFileWrite;
use crate::asset::project::ProjectPaths;
use crate::asset::{AssetImportError, AssetUri};

use super::ProjectManager;

const MODEL_IMPORT_DIRECTORY: &str = "models";

#[derive(Clone, Debug)]
pub(crate) struct ImportSourceWatchEcho {
    watched_uri: AssetUri,
    source_uri: AssetUri,
    target_path: PathBuf,
    content_hash: blake3::Hash,
}

impl ImportSourceWatchEcho {
    pub(in crate::asset) fn new(
        watched_uri: AssetUri,
        source_uri: AssetUri,
        target_path: PathBuf,
        bytes: &[u8],
    ) -> Self {
        Self {
            watched_uri,
            source_uri,
            target_path,
            content_hash: blake3::hash(bytes),
        }
    }

    pub(in crate::asset) fn watched_uri(&self) -> &AssetUri {
        &self.watched_uri
    }

    pub(in crate::asset) fn source_uri(&self) -> &AssetUri {
        &self.source_uri
    }

    pub(in crate::asset) fn target_path(&self) -> &Path {
        &self.target_path
    }

    pub(in crate::asset) fn content_hash(&self) -> blake3::Hash {
        self.content_hash
    }
}

/// A validated, write-free plan for bringing one model source into a project transaction.
///
/// The source bytes and optional OBJ material companion are captured before preparation so import
/// never reads an uncommitted destination. The corresponding writes are consumed only by the
/// final project durable transaction.
pub(crate) struct ImportSourcePlan {
    source_uri: AssetUri,
    source_path: PathBuf,
    source_bytes: Option<Vec<u8>>,
    source_mtime_unix_ms: Option<u64>,
    source_file_snapshots: BTreeMap<PathBuf, Vec<u8>>,
    staged_writes: Vec<PreparedFileWrite>,
    watch_echoes: Vec<ImportSourceWatchEcho>,
}

impl ImportSourcePlan {
    pub(crate) fn source_uri(&self) -> &AssetUri {
        &self.source_uri
    }

    pub(crate) fn source_path(&self) -> &Path {
        &self.source_path
    }

    pub(crate) fn source_bytes(&self) -> Option<&[u8]> {
        self.source_bytes.as_deref()
    }

    pub(crate) fn source_mtime_unix_ms(&self) -> Option<u64> {
        self.source_mtime_unix_ms
    }

    pub(crate) fn source_file_snapshots(&self) -> &BTreeMap<PathBuf, Vec<u8>> {
        &self.source_file_snapshots
    }

    pub(crate) fn into_staged_writes(self) -> Vec<PreparedFileWrite> {
        self.staged_writes
    }

    pub(crate) fn watch_echoes(&self) -> &[ImportSourceWatchEcho] {
        &self.watch_echoes
    }
}

impl ProjectManager {
    pub(crate) fn prepare_model_import_source(
        &self,
        source: &Path,
    ) -> Result<ImportSourcePlan, AssetImportError> {
        let source = ProjectPaths::resolve_existing_path(source)?;
        let extension = validate_model_source_extension(&source)?;
        match self.project_uri_for_source_path(&source) {
            Ok(source_uri) => Ok(ImportSourcePlan {
                source_path: source,
                source_uri,
                source_bytes: None,
                source_mtime_unix_ms: None,
                source_file_snapshots: BTreeMap::new(),
                staged_writes: Vec::new(),
                watch_echoes: Vec::new(),
            }),
            Err(AssetImportError::SourceOutsideProjectAssetRoots { .. }) => {
                self.prepare_external_model_import_source(source, extension)
            }
            Err(error) => Err(error),
        }
    }

    fn prepare_external_model_import_source(
        &self,
        source: PathBuf,
        extension: String,
    ) -> Result<ImportSourcePlan, AssetImportError> {
        let file_name = source
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| {
                AssetImportError::Parse(format!(
                    "model path has no UTF-8 file name: {}",
                    source.display()
                ))
            })?;
        let source_uri = AssetUri::parse(&format!("res://{MODEL_IMPORT_DIRECTORY}/{file_name}"))?;
        let source_path = match self.resolve_source_path_for_uri(&source_uri) {
            Ok(existing) => {
                return Err(AssetImportError::DuplicateProjectAssetUri {
                    uri: source_uri,
                    first: existing.into_operation_path(),
                    second: source,
                });
            }
            Err(AssetImportError::MissingProjectAssetUri { .. }) => {
                self.primary_project_source_path_for_uri(&source_uri)?
            }
            Err(error) => return Err(error),
        };
        let source_bytes = fs::read(&source)?;
        let source_mtime_unix_ms = source_mtime_unix_ms(&source)?;
        let mut source_file_snapshots = BTreeMap::new();
        let mut staged_writes = vec![PreparedFileWrite::new(
            source_path.clone(),
            source_bytes.clone(),
        )];
        let mut watch_echoes = vec![ImportSourceWatchEcho::new(
            source_uri.clone(),
            source_uri.clone(),
            source_path.clone(),
            &source_bytes,
        )];

        if extension == "obj" {
            let companion_source = source.with_extension("mtl");
            if companion_source.is_file() {
                let companion_target = source_path.with_extension("mtl");
                let companion_bytes = fs::read(companion_source)?;
                let companion_uri = self.project_uri_for_source_path(&companion_target)?;
                source_file_snapshots.insert(companion_target.clone(), companion_bytes.clone());
                staged_writes.push(PreparedFileWrite::new(
                    companion_target.clone(),
                    companion_bytes.clone(),
                ));
                watch_echoes.push(ImportSourceWatchEcho::new(
                    companion_uri,
                    source_uri.clone(),
                    companion_target,
                    &companion_bytes,
                ));
            }
        }

        Ok(ImportSourcePlan {
            source_uri,
            source_path,
            source_bytes: Some(source_bytes),
            source_mtime_unix_ms: Some(source_mtime_unix_ms),
            source_file_snapshots,
            staged_writes,
            watch_echoes,
        })
    }
}

fn validate_model_source_extension(source: &Path) -> Result<String, AssetImportError> {
    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if extension == "gltf" {
        return Err(AssetImportError::UnsupportedFormat(
            "model import does not support .gltf sources; copy the model folder into a configured project asset root or use .glb".to_owned(),
        ));
    }
    if !matches!(extension.as_str(), "obj" | "glb") {
        return Err(AssetImportError::UnsupportedFormat(format!(
            "model import supports .obj or .glb sources, found {}",
            source.display()
        )));
    }
    Ok(extension)
}

fn source_mtime_unix_ms(path: &Path) -> Result<u64, AssetImportError> {
    let modified = fs::metadata(path)?.modified()?;
    Ok(modified
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64)
}
