use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::{
    ArtifactStore, AssetId, AssetImportError, AssetImporter, AssetKind, AssetMetaDocument,
    AssetMetadata, AssetReference, AssetRegistry, AssetUri, AssetUuid, ImportedAsset, PreviewState,
    ProjectManifest, ProjectPaths, ResourceId,
};

const IMPORTER_VERSION: u32 = 1;

#[derive(Clone, Debug)]
pub struct ProjectManager {
    paths: ProjectPaths,
    manifest: ProjectManifest,
    registry: AssetRegistry,
    asset_ids_by_uuid: HashMap<AssetUuid, AssetId>,
    asset_uuids_by_id: HashMap<AssetId, AssetUuid>,
    importer: AssetImporter,
    artifact_store: ArtifactStore,
}

impl ProjectManager {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, AssetImportError> {
        let paths = ProjectPaths::from_root(root)?;
        paths.ensure_layout()?;
        let manifest = ProjectManifest::load(paths.manifest_path())?;
        Ok(Self {
            paths,
            manifest,
            registry: AssetRegistry::default(),
            asset_ids_by_uuid: HashMap::new(),
            asset_uuids_by_id: HashMap::new(),
            importer: AssetImporter::default(),
            artifact_store: ArtifactStore,
        })
    }

    pub fn scan_and_import(&mut self) -> Result<Vec<AssetMetadata>, AssetImportError> {
        let mut files = Vec::new();
        collect_files(self.paths.assets_root(), &mut files)?;
        files.sort();

        let mut registry = AssetRegistry::default();
        let mut asset_ids_by_uuid = HashMap::new();
        let mut asset_uuids_by_id = HashMap::new();
        let mut imported = Vec::with_capacity(files.len());

        for file in files {
            let uri = self.source_uri_for_path(&file)?;
            let imported_asset = self.importer.import_from_source(&file, &uri)?;
            let kind = asset_kind(&imported_asset);
            let source_bytes = fs::read(&file)?;
            let source_hash = hash_bytes(&source_bytes);
            let source_mtime_unix_ms = source_mtime_unix_ms(&file)?;
            let meta_path = meta_path_for_source(&file);
            let meta_exists = meta_path.exists();
            let mut meta = load_or_create_meta(&meta_path, &uri, kind)?;
            let previous_meta = meta.clone();
            let previous_source_hash = meta.source_hash.clone();
            meta.primary_locator = uri.clone();
            meta.kind = kind;
            meta.source_hash = source_hash.clone();
            meta.source_mtime_unix_ms = source_mtime_unix_ms;
            if meta.source_hash != previous_source_hash {
                meta.preview_state = PreviewState::Dirty;
            }
            if !meta_exists || meta != previous_meta {
                meta.save(&meta_path)?;
            }

            let asset_id = ResourceId::from_asset_uuid_label(meta.asset_uuid, uri.label());
            let artifact_uri = self.artifact_store.write(
                &self.paths,
                &AssetMetadata::new(asset_id, kind, uri.clone()),
                &imported_asset,
            )?;
            let metadata = AssetMetadata::new(asset_id, kind, uri)
                .with_source_hash(source_hash)
                .with_importer_version(IMPORTER_VERSION)
                .with_config_hash("")
                .with_artifact_locator(artifact_uri);
            registry.upsert(metadata.clone());
            asset_ids_by_uuid.insert(meta.asset_uuid, asset_id);
            asset_uuids_by_id.insert(asset_id, meta.asset_uuid);
            imported.push(metadata);
        }

        self.registry = registry;
        self.asset_ids_by_uuid = asset_ids_by_uuid;
        self.asset_uuids_by_id = asset_uuids_by_id;
        Ok(imported)
    }

    pub fn manifest(&self) -> &ProjectManifest {
        &self.manifest
    }

    pub fn paths(&self) -> &ProjectPaths {
        &self.paths
    }

    pub fn registry(&self) -> &AssetRegistry {
        &self.registry
    }

    pub fn asset_id_for_uri(&self, uri: &AssetUri) -> Option<AssetId> {
        self.registry
            .get_by_locator(uri)
            .map(|metadata| metadata.id())
    }

    pub fn asset_id_for_uuid(&self, uuid: AssetUuid) -> Option<AssetId> {
        self.asset_ids_by_uuid.get(&uuid).copied()
    }

    pub fn asset_uri_for_id(&self, id: AssetId) -> Option<&AssetUri> {
        self.registry
            .get(id)
            .map(|metadata| metadata.primary_locator())
    }

    pub fn asset_reference_for_id(&self, id: AssetId) -> Option<AssetReference> {
        let locator = self.asset_uri_for_id(id)?.clone();
        let uuid = self
            .asset_uuids_by_id
            .get(&id)
            .copied()
            .unwrap_or_else(|| AssetUuid::from_stable_label(&locator.to_string()));
        Some(AssetReference::new(uuid, locator))
    }

    pub fn source_path_for_uri(&self, uri: &AssetUri) -> Result<PathBuf, AssetImportError> {
        match uri.scheme() {
            crate::AssetUriScheme::Res => Ok(self.paths.assets_root().join(uri.path())),
            crate::AssetUriScheme::Library => Err(AssetImportError::UnsupportedFormat(format!(
                "source path requested for library uri {uri}"
            ))),
            crate::AssetUriScheme::Builtin | crate::AssetUriScheme::Memory => {
                Err(AssetImportError::UnsupportedFormat(format!(
                    "source path requested for non-project uri {uri}"
                )))
            }
        }
    }

    pub fn load_artifact(&self, uri: &AssetUri) -> Result<ImportedAsset, AssetImportError> {
        let metadata = self.registry.get_by_locator(uri).ok_or_else(|| {
            AssetImportError::Parse(format!("missing asset metadata for source uri {uri}"))
        })?;
        let artifact_uri = metadata.artifact_locator().ok_or_else(|| {
            AssetImportError::Parse(format!("missing artifact uri for source uri {uri}"))
        })?;
        self.artifact_store.read(&self.paths, artifact_uri)
    }

    pub fn load_artifact_by_id(&self, id: AssetId) -> Result<ImportedAsset, AssetImportError> {
        let metadata = self.registry.get(id).ok_or_else(|| {
            AssetImportError::Parse(format!("missing asset metadata for asset id {id}"))
        })?;
        let artifact_uri = metadata.artifact_locator().ok_or_else(|| {
            AssetImportError::Parse(format!("missing artifact uri for asset id {id}"))
        })?;
        self.artifact_store.read(&self.paths, artifact_uri)
    }

    fn source_uri_for_path(&self, path: &Path) -> Result<AssetUri, AssetImportError> {
        let relative = path
            .strip_prefix(self.paths.assets_root())
            .map_err(|error| {
                AssetImportError::Parse(format!(
                    "asset path {} is outside assets root {}: {error}",
                    path.display(),
                    self.paths.assets_root().display()
                ))
            })?;
        let relative = relative
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        Ok(AssetUri::parse(&format!("res://{relative}"))?)
    }
}

fn collect_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, files)?;
        } else if path.is_file() && !is_meta_sidecar(&path) {
            files.push(path);
        }
    }
    Ok(())
}

fn asset_kind(imported: &ImportedAsset) -> AssetKind {
    match imported {
        ImportedAsset::Texture(_) => AssetKind::Texture,
        ImportedAsset::Shader(_) => AssetKind::Shader,
        ImportedAsset::Material(_) => AssetKind::Material,
        ImportedAsset::Scene(_) => AssetKind::Scene,
        ImportedAsset::Model(_) => AssetKind::Model,
        ImportedAsset::UiLayout(_) => AssetKind::UiLayout,
        ImportedAsset::UiWidget(_) => AssetKind::UiWidget,
        ImportedAsset::UiStyle(_) => AssetKind::UiStyle,
    }
}

fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn source_mtime_unix_ms(path: &Path) -> Result<u64, std::io::Error> {
    let modified = fs::metadata(path)?.modified()?;
    let duration = modified.duration_since(UNIX_EPOCH).unwrap_or_default();
    Ok(duration.as_millis() as u64)
}

fn meta_path_for_source(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("asset");
    path.with_file_name(format!("{file_name}.meta.toml"))
}

fn is_meta_sidecar(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".meta.toml"))
}

fn load_or_create_meta(
    meta_path: &Path,
    uri: &AssetUri,
    kind: AssetKind,
) -> Result<AssetMetaDocument, AssetImportError> {
    if meta_path.exists() {
        let mut meta = AssetMetaDocument::load(meta_path)?;
        meta.primary_locator = uri.clone();
        meta.kind = kind;
        return Ok(meta);
    }

    Ok(AssetMetaDocument::new(AssetUuid::new(), uri.clone(), kind))
}
