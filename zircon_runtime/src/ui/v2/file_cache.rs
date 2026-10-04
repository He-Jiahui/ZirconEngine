use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zircon_runtime_interface::ui::template::{
    UiAssetFingerprint, UI_COMPILED_ASSET_COMPILER_SCHEMA_VERSION,
};
use zircon_runtime_interface::ui::v2::{UiV2AssetDocument, UiV2AssetError, UiV2CompiledDocument};

use super::{
    cache::source_path_identity_for_path as workspace_source_path_identity_for_path,
    component_reference::{parse_v2_widget_import_reference, UiV2WidgetImportReference},
    UiV2DocumentCompiler, UiV2PrototypeStore, UiV2PrototypeStoreBuilder, UiZuiAssetLoader,
};
use crate::ui::template::{UiCompiledArtifactKey, UiCompiledArtifactStore};

const UI_V2_PERSISTENT_FILE_CACHE_SCHEMA_VERSION: u32 = 5;
const UI_V2_PERSISTENT_FILE_CACHE_RECORD_VERSION: u32 = 5;

/// Raw-source proof for a parsed `.zui` file. `physical_path` names the selected
/// package/development file; `source_path` and `resource_uri` retain its stable identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiV2SourceFileReceipt {
    pub asset_id: String,
    pub source_path: Option<String>,
    pub resource_uri: Option<String>,
    pub physical_path: PathBuf,
    pub sha256: String,
}

/// A widget import that named an asset ID absent from the selected asset root.
/// The document compiler may resolve built-in nodes by other means, but strict
/// captures must keep that source provenance incomplete.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiV2UnresolvedSourceImport {
    pub source_asset_id: String,
    pub source_path: Option<String>,
    pub resource_uri: Option<String>,
    pub reference: String,
}

#[derive(Clone, Debug)]
pub struct UiV2PrototypeStoreLoadOutcome {
    pub root_asset_id: String,
    pub root_document: Arc<UiV2AssetDocument>,
    pub compiled: Arc<UiV2CompiledDocument>,
    pub store: Arc<UiV2PrototypeStore>,
    pub source_receipts: Vec<UiV2SourceFileReceipt>,
    pub unresolved_imports: Vec<UiV2UnresolvedSourceImport>,
    pub cache_hit: bool,
    pub persistent_cache_hit: bool,
}

#[derive(Clone, Debug)]
pub struct UiV2PrototypeStoreFileCache {
    entries: BTreeMap<UiV2FileStoreCacheKey, UiV2FileStoreCacheEntry>,
    request_entries: BTreeMap<UiV2FileStoreRequestKey, UiV2FileStoreCacheKey>,
    persistent_store: Option<UiCompiledArtifactStore>,
}

impl UiV2PrototypeStoreFileCache {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            request_entries: BTreeMap::new(),
            persistent_store: None,
        }
    }

    pub fn with_persistent_cache(root: impl Into<PathBuf>) -> Self {
        Self {
            entries: BTreeMap::new(),
            request_entries: BTreeMap::new(),
            persistent_store: Some(UiCompiledArtifactStore::new(root)),
        }
    }

    pub fn persistent_store(&self) -> Option<&UiCompiledArtifactStore> {
        self.persistent_store.as_ref()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.request_entries.clear();
    }

    /// Returns an already compiled request without probing source metadata.
    ///
    /// Event-driven clients can use this on their interactive path after they
    /// clear the cache in response to file-system notifications. A miss still
    /// performs the same validated load as [`Self::load_store`].
    pub fn load_store_cached<P, I>(
        &mut self,
        paths: I,
    ) -> Result<UiV2PrototypeStoreLoadOutcome, UiV2AssetError>
    where
        P: AsRef<Path>,
        I: IntoIterator<Item = P>,
    {
        let paths = collect_paths(paths)?;
        let request_key = UiV2FileStoreRequestKey::from_paths(&paths);
        if let Some(entry) = self
            .request_entries
            .get(&request_key)
            .and_then(|entry_key| self.entries.get(entry_key))
        {
            return Ok(entry.to_outcome(true, false));
        }
        self.load_store_from_paths(paths)
    }

    pub fn load_store<P, I>(
        &mut self,
        paths: I,
    ) -> Result<UiV2PrototypeStoreLoadOutcome, UiV2AssetError>
    where
        P: AsRef<Path>,
        I: IntoIterator<Item = P>,
    {
        let paths = collect_paths(paths)?;
        self.load_store_from_paths(paths)
    }

    fn load_store_from_paths(
        &mut self,
        paths: Vec<PathBuf>,
    ) -> Result<UiV2PrototypeStoreLoadOutcome, UiV2AssetError> {
        let request_key = UiV2FileStoreRequestKey::from_paths(&paths);
        let explicit_cache_key = UiV2FileStoreCacheKey::from_paths(&paths);
        if let Some(entry) = self.entries.get(&explicit_cache_key) {
            let current_source_key =
                UiV2FileStoreCacheKey::from_canonical_paths(&entry.source_paths);
            if entry.source_key == current_source_key {
                let outcome = entry.to_outcome(true, false);
                self.request_entries.insert(request_key, explicit_cache_key);
                return Ok(outcome);
            }
        }

        if let Some(store) = &self.persistent_store {
            if let Some(entry) = load_persistent_entry(store, &explicit_cache_key)? {
                let outcome = entry.to_outcome(true, true);
                let _ = self.entries.insert(explicit_cache_key.clone(), entry);
                self.request_entries.insert(request_key, explicit_cache_key);
                return Ok(outcome);
            }
        }

        let (sources, unresolved_imports) = collect_v2_sources(&paths)?;
        let entry = build_file_store_cache_entry(sources, unresolved_imports)?;
        if let Some(store) = &self.persistent_store {
            store_persistent_entry(store, &explicit_cache_key, &entry)?;
        }
        let outcome = entry.to_outcome(false, false);
        let _ = self.entries.insert(explicit_cache_key.clone(), entry);
        self.request_entries.insert(request_key, explicit_cache_key);
        Ok(outcome)
    }
}

impl Default for UiV2PrototypeStoreFileCache {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
struct UiV2FileStoreCacheEntry {
    root_asset_id: String,
    root_document: Arc<UiV2AssetDocument>,
    compiled: Arc<UiV2CompiledDocument>,
    store: Arc<UiV2PrototypeStore>,
    source_paths: Vec<PathBuf>,
    source_receipts: Vec<UiV2SourceFileReceipt>,
    unresolved_imports: Vec<UiV2UnresolvedSourceImport>,
    source_key: UiV2FileStoreCacheKey,
}

impl UiV2FileStoreCacheEntry {
    fn to_outcome(
        &self,
        cache_hit: bool,
        persistent_cache_hit: bool,
    ) -> UiV2PrototypeStoreLoadOutcome {
        UiV2PrototypeStoreLoadOutcome {
            root_asset_id: self.root_asset_id.clone(),
            root_document: Arc::clone(&self.root_document),
            compiled: Arc::clone(&self.compiled),
            store: Arc::clone(&self.store),
            source_receipts: self.source_receipts.clone(),
            unresolved_imports: self.unresolved_imports.clone(),
            cache_hit,
            persistent_cache_hit,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct UiV2FileStoreCacheKey {
    sources: Vec<UiV2FileCacheSourceKey>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct UiV2FileStoreRequestKey {
    paths: Vec<PathBuf>,
}

impl UiV2FileStoreRequestKey {
    fn from_paths(paths: &[PathBuf]) -> Self {
        Self {
            paths: paths.to_vec(),
        }
    }
}

impl UiV2FileStoreCacheKey {
    fn from_paths(paths: &[PathBuf]) -> Self {
        Self {
            sources: paths
                .iter()
                .map(|path| UiV2FileCacheSourceKey::from_path(path))
                .collect(),
        }
    }

    fn from_canonical_paths(paths: &[PathBuf]) -> Self {
        Self {
            sources: paths
                .iter()
                .map(|path| UiV2FileCacheSourceKey::from_canonical_path(path))
                .collect(),
        }
    }

    fn from_receipts(receipts: &[UiV2SourceFileReceipt]) -> Self {
        Self {
            sources: receipts
                .iter()
                .map(|receipt| UiV2FileCacheSourceKey {
                    path: receipt.physical_path.clone(),
                    sha256: Some(receipt.sha256.clone()),
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct UiV2FileCacheSourceKey {
    path: PathBuf,
    sha256: Option<String>,
}

impl UiV2FileCacheSourceKey {
    fn from_path(path: &Path) -> Self {
        let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        Self::from_canonical_path(&path)
    }

    fn from_canonical_path(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            sha256: std::fs::read(path).ok().map(|bytes| sha256_hex(&bytes)),
        }
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn collect_paths<P, I>(paths: I) -> Result<Vec<PathBuf>, UiV2AssetError>
where
    P: AsRef<Path>,
    I: IntoIterator<Item = P>,
{
    let paths = paths
        .into_iter()
        .map(|path| path.as_ref().to_path_buf())
        .collect::<Vec<_>>();
    if paths.is_empty() {
        return Err(UiV2AssetError::InvalidDocument {
            asset_id: "v2-prototype-store-file-cache".to_string(),
            detail: "v2 prototype store file cache requires at least one source".to_string(),
        });
    }
    Ok(paths)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct UiV2PersistentFileCacheRecord {
    record_version: u32,
    root_asset_id: String,
    root_document: UiV2AssetDocument,
    compiled: UiV2CompiledDocument,
    documents: Vec<UiV2PersistentFileCacheDocument>,
    source_paths: Vec<PathBuf>,
    source_receipts: Vec<UiV2SourceFileReceipt>,
    unresolved_imports: Vec<UiV2UnresolvedSourceImport>,
    source_key: UiV2FileStoreCacheKey,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct UiV2PersistentFileCacheDocument {
    document: UiV2AssetDocument,
    aliases: Vec<String>,
    #[serde(default)]
    source_path: Option<String>,
}

fn load_persistent_entry(
    store: &UiCompiledArtifactStore,
    explicit_cache_key: &UiV2FileStoreCacheKey,
) -> Result<Option<UiV2FileStoreCacheEntry>, UiV2AssetError> {
    let Some(key) = persistent_key_for_source_key("ui-v2-file-cache", explicit_cache_key) else {
        return Ok(None);
    };
    let Some(bytes) = store
        .load_payload_bytes(&key)
        .map_err(|error| persistent_cache_io_error("read", &key.asset_id, error))?
    else {
        return Ok(None);
    };
    let record = match bincode::deserialize::<UiV2PersistentFileCacheRecord>(&bytes) {
        Ok(record) => record,
        Err(_) => return Ok(None),
    };
    if record.record_version != UI_V2_PERSISTENT_FILE_CACHE_RECORD_VERSION {
        return Ok(None);
    }
    if !persistent_receipts_match_paths(&record.source_paths, &record.source_receipts) {
        return Ok(None);
    }
    let current_source_key = UiV2FileStoreCacheKey::from_canonical_paths(&record.source_paths);
    let receipt_source_key = UiV2FileStoreCacheKey::from_receipts(&record.source_receipts);
    if record.source_key != current_source_key || record.source_key != receipt_source_key {
        return Ok(None);
    }

    let mut builder = UiV2PrototypeStoreBuilder::new();
    for document in &record.documents {
        if let Some(source_path) = &document.source_path {
            let _ = builder.insert_with_aliases_and_source_path(
                document.document.clone(),
                document.aliases.clone(),
                source_path.clone(),
            );
        } else {
            let _ =
                builder.insert_with_aliases(document.document.clone(), document.aliases.clone());
        }
    }
    let store = Arc::new(builder.build()?);
    let root_document =
        store
            .get(&record.root_asset_id)
            .ok_or_else(|| UiV2AssetError::InvalidDocument {
                asset_id: record.root_asset_id.clone(),
                detail: "v2 persistent file cache did not retain the root asset document"
                    .to_string(),
            })?;

    Ok(Some(UiV2FileStoreCacheEntry {
        root_asset_id: record.root_asset_id,
        root_document,
        compiled: Arc::new(record.compiled),
        store,
        source_paths: record.source_paths,
        source_receipts: record.source_receipts,
        unresolved_imports: record.unresolved_imports,
        source_key: record.source_key,
    }))
}

fn persistent_receipts_match_paths(
    source_paths: &[PathBuf],
    source_receipts: &[UiV2SourceFileReceipt],
) -> bool {
    source_paths.len() == source_receipts.len()
        && source_paths
            .iter()
            .zip(source_receipts)
            .all(|(path, receipt)| {
                let canonical_path = path.canonicalize().unwrap_or_else(|_| path.clone());
                canonical_path == receipt.physical_path
                    && !receipt.asset_id.is_empty()
                    && receipt.sha256.len() == 64
                    && receipt.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
            })
}

fn store_persistent_entry(
    store: &UiCompiledArtifactStore,
    explicit_cache_key: &UiV2FileStoreCacheKey,
    entry: &UiV2FileStoreCacheEntry,
) -> Result<(), UiV2AssetError> {
    let Some(key) = persistent_key_for_source_key("ui-v2-file-cache", explicit_cache_key) else {
        return Ok(());
    };
    let record = UiV2PersistentFileCacheRecord {
        record_version: UI_V2_PERSISTENT_FILE_CACHE_RECORD_VERSION,
        root_asset_id: entry.root_asset_id.clone(),
        root_document: entry.root_document.as_ref().clone(),
        compiled: entry.compiled.as_ref().clone(),
        documents: persistent_documents_for_entry(entry),
        source_paths: entry.source_paths.clone(),
        source_receipts: entry.source_receipts.clone(),
        unresolved_imports: entry.unresolved_imports.clone(),
        source_key: entry.source_key.clone(),
    };
    let bytes = bincode::serialize(&record)
        .map_err(|error| persistent_cache_data_error("serialize", &entry.root_asset_id, error))?;
    store
        .store_payload_bytes(&key, &bytes)
        .map_err(|error| persistent_cache_io_error("write", &entry.root_asset_id, error))?;
    Ok(())
}

fn persistent_documents_for_entry(
    entry: &UiV2FileStoreCacheEntry,
) -> Vec<UiV2PersistentFileCacheDocument> {
    let mut documents = BTreeMap::<String, UiV2PersistentFileCacheDocument>::new();
    for document in entry.store.documents() {
        let canonical_id = document.asset.id.clone();
        let _ = documents.entry(canonical_id.clone()).or_insert_with(|| {
            UiV2PersistentFileCacheDocument {
                document: document.as_ref().clone(),
                aliases: Vec::new(),
                source_path: entry
                    .store
                    .source_path_for_asset_id(&canonical_id)
                    .map(str::to_string),
            }
        });
    }
    for source_path in &entry.source_paths {
        let Ok(document) = load_ui_v2_source_file(source_path) else {
            continue;
        };
        let canonical_id = document.asset.id;
        if let (Some(record), Some(alias)) = (
            documents.get_mut(&canonical_id),
            resource_alias_for_path(source_path),
        ) {
            if alias != canonical_id && !record.aliases.contains(&alias) {
                record.aliases.push(alias);
            }
        }
    }
    documents.into_values().collect()
}

fn persistent_key_for_source_key(
    asset_id: &str,
    source_key: &UiV2FileStoreCacheKey,
) -> Option<UiCompiledArtifactKey> {
    let bytes = bincode::serialize(source_key).ok()?;
    let fingerprint = UiAssetFingerprint::from_bytes(&bytes).value;
    Some(UiCompiledArtifactKey::new(
        asset_id,
        fingerprint,
        UI_V2_PERSISTENT_FILE_CACHE_SCHEMA_VERSION,
        UI_COMPILED_ASSET_COMPILER_SCHEMA_VERSION,
    ))
}

fn persistent_cache_io_error(
    operation: &str,
    asset_id: &str,
    error: std::io::Error,
) -> UiV2AssetError {
    persistent_cache_data_error(operation, asset_id, error)
}

fn persistent_cache_data_error(
    operation: &str,
    asset_id: &str,
    error: impl std::fmt::Display,
) -> UiV2AssetError {
    UiV2AssetError::InvalidDocument {
        asset_id: asset_id.to_string(),
        detail: format!("failed to {operation} v2 persistent file cache: {error}"),
    }
}

struct UiV2FileSource {
    path: PathBuf,
    document: UiV2AssetDocument,
    receipt: UiV2SourceFileReceipt,
}

fn collect_v2_sources(
    paths: &[PathBuf],
) -> Result<(Vec<UiV2FileSource>, Vec<UiV2UnresolvedSourceImport>), UiV2AssetError> {
    // Root paths provide the initial queue/output bound; canonical de-duplication
    // may leave spare capacity and transitive imports may grow both collections.
    let mut queue = Vec::with_capacity(paths.len());
    let mut seen = BTreeSet::new();
    let mut asset_id_index = BTreeMap::new();
    for path in paths {
        push_source_path(&mut queue, &mut seen, path.clone());
    }

    let mut sources = Vec::with_capacity(paths.len());
    let mut unresolved_imports = Vec::new();
    let mut index = 0;
    while index < queue.len() {
        let path = queue[index].clone();
        index += 1;
        let (document, receipt) = load_ui_v2_source_with_receipt(&path)?;
        for reference in v2_import_references(&document)? {
            if let Some(import_path) = resolve_resource_reference_path(&path, &reference)
                .or_else(|| resolve_asset_id_reference_path(&path, &reference, &mut asset_id_index))
            {
                push_source_path(&mut queue, &mut seen, import_path);
            } else {
                unresolved_imports.push(UiV2UnresolvedSourceImport {
                    source_asset_id: receipt.asset_id.clone(),
                    source_path: receipt.source_path.clone(),
                    resource_uri: receipt.resource_uri.clone(),
                    reference,
                });
            }
        }
        sources.push(UiV2FileSource {
            path,
            document,
            receipt,
        });
    }

    Ok((sources, unresolved_imports))
}

fn push_source_path(queue: &mut Vec<PathBuf>, seen: &mut BTreeSet<PathBuf>, path: PathBuf) {
    let key = path.canonicalize().unwrap_or(path);
    if seen.insert(key.clone()) {
        queue.push(key);
    }
}

fn v2_import_references(document: &UiV2AssetDocument) -> Result<Vec<String>, UiV2AssetError> {
    let mut references =
        Vec::with_capacity(document.imports.widgets.len() + document.imports.styles.len());
    for reference in &document.imports.widgets {
        match parse_v2_widget_import_reference(&document.asset.id, reference)? {
            UiV2WidgetImportReference::WholeAsset(asset_id)
            | UiV2WidgetImportReference::Component { asset_id, .. } => {
                references.push(asset_id.to_string());
            }
        }
    }
    references.extend(document.imports.styles.iter().cloned());
    Ok(references)
}

fn build_file_store_cache_entry(
    sources: Vec<UiV2FileSource>,
    unresolved_imports: Vec<UiV2UnresolvedSourceImport>,
) -> Result<UiV2FileStoreCacheEntry, UiV2AssetError> {
    let source_count = sources.len();
    let mut sources = sources.into_iter();
    let root_source = sources
        .next()
        .expect("source collection rejects empty input");
    let root_asset_id = root_source.document.asset.id.clone();
    let mut root_document = root_source.document;
    let root_alias = resource_alias_for_path(&root_source.path);
    let root_source_path = root_source.receipt.source_path.clone();
    let mut source_paths = Vec::with_capacity(source_count);
    let mut source_receipts = Vec::with_capacity(source_count);
    source_paths.push(root_source.path);
    source_receipts.push(root_source.receipt);
    let mut imported_documents = Vec::with_capacity(source_count.saturating_sub(1));
    for source in sources {
        root_document.tokens.extend(source.document.tokens.clone());
        root_document
            .stylesheets
            .extend(source.document.stylesheets.clone());
        let alias = resource_alias_for_path(&source.path);
        let source_path = source.receipt.source_path.clone();
        source_paths.push(source.path);
        source_receipts.push(source.receipt);
        imported_documents.push((source.document, alias, source_path));
    }
    let source_key = UiV2FileStoreCacheKey::from_receipts(&source_receipts);

    let mut builder = UiV2PrototypeStoreBuilder::new();
    if let Some(source_path) = root_source_path {
        let _ = builder.insert_with_aliases_and_source_path(root_document, root_alias, source_path);
    } else {
        let _ = builder.insert_with_aliases(root_document, root_alias);
    }
    for (document, alias, source_path) in imported_documents {
        if let Some(source_path) = source_path {
            let _ = builder.insert_with_aliases_and_source_path(document, alias, source_path);
        } else {
            let _ = builder.insert_with_aliases(document, alias);
        }
    }
    let store = Arc::new(builder.build()?);
    let root_document =
        store
            .get(&root_asset_id)
            .ok_or_else(|| UiV2AssetError::InvalidDocument {
                asset_id: root_asset_id.clone(),
                detail: "v2 file cache did not retain the root asset document".to_string(),
            })?;
    let compiled = Arc::new(UiV2DocumentCompiler::compile_with_prototype_store(
        root_document.as_ref(),
        store.as_ref(),
    )?);

    Ok(UiV2FileStoreCacheEntry {
        root_asset_id,
        root_document,
        compiled,
        store,
        source_paths,
        source_receipts,
        unresolved_imports,
        source_key,
    })
}

fn resolve_resource_reference_path(source_path: &Path, reference: &str) -> Option<PathBuf> {
    let relative = reference.strip_prefix("res://")?;
    let asset_root = asset_root_for_path(source_path)?;
    let mut path = asset_root.to_path_buf();
    for segment in relative.split('/') {
        path.push(segment);
    }
    Some(path)
}

fn resolve_asset_id_reference_path(
    source_path: &Path,
    reference: &str,
    asset_id_index: &mut BTreeMap<String, PathBuf>,
) -> Option<PathBuf> {
    if reference.contains("://") {
        return None;
    }
    if asset_id_index.is_empty() {
        *asset_id_index = build_v2_asset_id_index(source_path);
    }
    asset_id_index.get(reference).cloned()
}

fn build_v2_asset_id_index(source_path: &Path) -> BTreeMap<String, PathBuf> {
    let Some(asset_root) = asset_root_for_path(source_path) else {
        return BTreeMap::new();
    };
    let mut index = BTreeMap::new();
    let mut stack = vec![asset_root.to_path_buf()];
    while let Some(path) = stack.pop() {
        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&path) {
                let mut entries = entries
                    .flatten()
                    .map(|entry| entry.path())
                    .collect::<Vec<_>>();
                entries.sort();
                stack.extend(entries);
            }
            continue;
        }
        if !is_ui_v2_source_path(&path) {
            continue;
        }
        if let Ok(document) = load_ui_v2_source_file(&path) {
            match index.entry(document.asset.id) {
                Entry::Vacant(entry) => {
                    entry.insert(path);
                }
                Entry::Occupied(mut entry) => {
                    if should_replace_v2_asset_id_index_path(entry.get(), &path) {
                        entry.insert(path);
                    }
                }
            }
        }
    }
    index
}

fn should_replace_v2_asset_id_index_path(existing: &Path, candidate: &Path) -> bool {
    candidate.to_string_lossy() < existing.to_string_lossy()
}

fn resource_alias_for_path(path: &Path) -> Option<String> {
    let asset_root = asset_root_for_path(path)?;
    let relative = path.strip_prefix(asset_root).ok()?;
    let parts = relative
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect::<Vec<_>>();
    (!parts.is_empty()).then(|| format!("res://{}", parts.join("/")))
}

fn source_path_identity_for_path(path: &Path) -> Option<String> {
    workspace_source_path_identity_for_path(path).or_else(|| {
        let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        resource_alias_for_path(&canonical_path)
    })
}

fn asset_root_for_path(path: &Path) -> Option<&Path> {
    let mut fallback = None;
    for ancestor in path
        .ancestors()
        .filter(|ancestor| ancestor.file_name().and_then(|name| name.to_str()) == Some("assets"))
    {
        if fallback.is_none() {
            fallback = Some(ancestor);
        }
        // `res://` is rooted at the package asset directory. Component content
        // can legitimately live in nested folders named `assets`, so prefer the
        // first ancestor that owns the package-level UI tree.
        if ancestor.join("ui").is_dir() {
            return Some(ancestor);
        }
    }
    fallback
}

fn load_ui_v2_source_file(path: &Path) -> Result<UiV2AssetDocument, UiV2AssetError> {
    UiZuiAssetLoader::load_zui_file(path)
}

fn load_ui_v2_source_with_receipt(
    path: &Path,
) -> Result<(UiV2AssetDocument, UiV2SourceFileReceipt), UiV2AssetError> {
    let input = std::fs::read_to_string(path)
        .map_err(|error| UiV2AssetError::Io(format!("{}: {error}", path.display())))?;
    let document = UiZuiAssetLoader::load_zui_str(&input)?;
    let physical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let receipt = UiV2SourceFileReceipt {
        asset_id: document.asset.id.clone(),
        source_path: source_path_identity_for_path(&physical_path),
        resource_uri: resource_alias_for_path(&physical_path),
        physical_path,
        sha256: sha256_hex(input.as_bytes()),
    };
    Ok((document, receipt))
}

fn is_ui_v2_source_path(path: &Path) -> bool {
    lower_file_name(path).is_some_and(|name| name.ends_with(".zui"))
}

fn lower_file_name(path: &Path) -> Option<String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_ascii_lowercase)
}

#[cfg(test)]
#[path = "tests/file_cache_performance_tests.rs"]
mod performance_tests;

#[cfg(test)]
#[path = "file_cache/tests/canonical_revalidation_tests.rs"]
mod canonical_revalidation_tests;

#[cfg(test)]
#[path = "tests/file_source_capacity_tests.rs"]
mod file_source_capacity_tests;
