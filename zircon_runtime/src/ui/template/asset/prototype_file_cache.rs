use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use zircon_runtime_interface::ui::template::{UiAssetError, UiRawAssetPrototype};

use super::{UiAssetLoader, UiPrototypeStore, UiPrototypeStoreBuilder};

#[derive(Clone, Debug)]
pub struct UiPrototypeStoreLoadOutcome {
    pub root_asset_id: String,
    pub store: Arc<UiPrototypeStore>,
    pub cache_hit: bool,
}

#[derive(Clone, Debug, Default)]
pub struct UiPrototypeStoreFileCache {
    entries: BTreeMap<UiPrototypeFileStoreCacheKey, UiPrototypeFileStoreCacheEntry>,
}

impl UiPrototypeStoreFileCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn load_flat_store<P, I>(
        &mut self,
        paths: I,
    ) -> Result<UiPrototypeStoreLoadOutcome, UiAssetError>
    where
        P: AsRef<Path>,
        I: IntoIterator<Item = P>,
    {
        self.load_flat_store_inner(paths, FlatProbeMode::RequireFlat)
            .map(|outcome| outcome.expect("required flat prototype loading always returns outcome"))
    }

    pub fn try_load_flat_store<P, I>(
        &mut self,
        paths: I,
    ) -> Result<Option<UiPrototypeStoreLoadOutcome>, UiAssetError>
    where
        P: AsRef<Path>,
        I: IntoIterator<Item = P>,
    {
        self.load_flat_store_inner(paths, FlatProbeMode::ReturnNoneForTreeSchema)
    }

    fn load_flat_store_inner<P, I>(
        &mut self,
        paths: I,
        probe_mode: FlatProbeMode,
    ) -> Result<Option<UiPrototypeStoreLoadOutcome>, UiAssetError>
    where
        P: AsRef<Path>,
        I: IntoIterator<Item = P>,
    {
        let paths = collect_paths(paths)?;
        let explicit_cache_key = UiPrototypeFileStoreCacheKey::from_paths(&paths);
        if let Some(entry) = self.entries.get(&explicit_cache_key) {
            let current_source_key = UiPrototypeFileStoreCacheKey::from_paths(&entry.source_paths);
            if entry.source_key == current_source_key {
                return Ok(Some(entry.to_outcome(true)));
            }
        }

        let Some(sources) = collect_flat_prototype_sources(&paths, probe_mode)? else {
            return Ok(None);
        };
        let entry = build_file_store_cache_entry(sources)?;
        let outcome = entry.to_outcome(false);
        let _ = self.entries.insert(explicit_cache_key, entry);
        Ok(Some(outcome))
    }
}

#[derive(Clone, Debug)]
struct UiPrototypeFileStoreCacheEntry {
    root_asset_id: String,
    store: Arc<UiPrototypeStore>,
    source_paths: Vec<PathBuf>,
    source_key: UiPrototypeFileStoreCacheKey,
}

impl UiPrototypeFileStoreCacheEntry {
    fn to_outcome(&self, cache_hit: bool) -> UiPrototypeStoreLoadOutcome {
        UiPrototypeStoreLoadOutcome {
            root_asset_id: self.root_asset_id.clone(),
            store: Arc::clone(&self.store),
            cache_hit,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct UiPrototypeFileStoreCacheKey {
    sources: Vec<UiPrototypeFileCacheSourceKey>,
}

impl UiPrototypeFileStoreCacheKey {
    fn from_paths(paths: &[PathBuf]) -> Self {
        Self {
            sources: paths
                .iter()
                .map(|path| UiPrototypeFileCacheSourceKey::from_path(path))
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct UiPrototypeFileCacheSourceKey {
    path: PathBuf,
    modified_unix_ns: Option<u128>,
    len: Option<u64>,
}

impl UiPrototypeFileCacheSourceKey {
    fn from_path(path: &Path) -> Self {
        let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let metadata = std::fs::metadata(&path).ok();
        let modified_unix_ns = metadata
            .as_ref()
            .and_then(|metadata| metadata.modified().ok())
            .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_nanos());
        let len = metadata.as_ref().map(std::fs::Metadata::len);
        Self {
            path,
            modified_unix_ns,
            len,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FlatProbeMode {
    RequireFlat,
    ReturnNoneForTreeSchema,
}

fn collect_paths<P, I>(paths: I) -> Result<Vec<PathBuf>, UiAssetError>
where
    P: AsRef<Path>,
    I: IntoIterator<Item = P>,
{
    let paths = paths
        .into_iter()
        .map(|path| path.as_ref().to_path_buf())
        .collect::<Vec<_>>();
    if paths.is_empty() {
        return Err(UiAssetError::InvalidDocument {
            asset_id: "prototype-store-file-cache".to_string(),
            detail: "prototype store file cache requires at least one source".to_string(),
        });
    }
    Ok(paths)
}

struct UiPrototypeFileSource {
    path: PathBuf,
    prototype: UiRawAssetPrototype,
}

fn collect_flat_prototype_sources(
    paths: &[PathBuf],
    probe_mode: FlatProbeMode,
) -> Result<Option<Vec<UiPrototypeFileSource>>, UiAssetError> {
    let mut queue = Vec::new();
    let mut seen = BTreeSet::new();
    for path in paths {
        push_source_path(&mut queue, &mut seen, path.clone());
    }

    let mut sources = Vec::new();
    let mut index = 0;
    while index < queue.len() {
        let path = queue[index].clone();
        index += 1;
        let input =
            std::fs::read_to_string(&path).map_err(|error| UiAssetError::Io(error.to_string()))?;
        if sources.is_empty()
            && probe_mode == FlatProbeMode::ReturnNoneForTreeSchema
            && !looks_like_flat_prototype_source(&input)
        {
            return Ok(None);
        }

        let prototype = UiAssetLoader::load_flat_prototype_toml_str(&input)?;
        for reference in prototype_import_references(&prototype) {
            if let Some(import_path) = resolve_resource_reference_path(&path, &reference) {
                push_source_path(&mut queue, &mut seen, import_path);
            }
        }
        sources.push(UiPrototypeFileSource { path, prototype });
    }

    Ok(Some(sources))
}

fn push_source_path(queue: &mut Vec<PathBuf>, seen: &mut BTreeSet<PathBuf>, path: PathBuf) {
    let key = path.canonicalize().unwrap_or(path);
    if seen.insert(key.clone()) {
        queue.push(key);
    }
}

fn prototype_import_references(prototype: &UiRawAssetPrototype) -> Vec<String> {
    let mut references =
        Vec::with_capacity(prototype.imports.widgets.len() + prototype.imports.styles.len());
    references.extend(prototype.imports.widgets.iter().map(|reference| {
        reference
            .split_once('#')
            .map_or(reference.as_str(), |(asset_id, _)| asset_id)
            .to_string()
    }));
    references.extend(prototype.imports.styles.iter().cloned());
    references
}

fn build_file_store_cache_entry(
    sources: Vec<UiPrototypeFileSource>,
) -> Result<UiPrototypeFileStoreCacheEntry, UiAssetError> {
    let mut builder = UiPrototypeStoreBuilder::new();
    let mut root_asset_id = None;
    let source_paths = sources
        .iter()
        .map(|source| source.path.clone())
        .collect::<Vec<_>>();
    let source_key = UiPrototypeFileStoreCacheKey::from_paths(&source_paths);
    for source in sources {
        let aliases = resource_alias_for_path(&source.path).into_iter();
        let prototype = source.prototype;
        root_asset_id.get_or_insert_with(|| prototype.asset.id.clone());
        let _ = builder.insert_with_aliases(prototype, aliases);
    }
    let store = Arc::new(builder.build()?);
    Ok(UiPrototypeFileStoreCacheEntry {
        root_asset_id: root_asset_id.expect("sources are checked as non-empty"),
        store,
        source_paths,
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

fn resource_alias_for_path(path: &Path) -> Option<String> {
    let asset_root = asset_root_for_path(path)?;
    let relative = path.strip_prefix(asset_root).ok()?;
    let parts = relative
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect::<Vec<_>>();
    (!parts.is_empty()).then(|| format!("res://{}", parts.join("/")))
}

fn asset_root_for_path(path: &Path) -> Option<&Path> {
    path.ancestors()
        .find(|ancestor| ancestor.file_name().and_then(|name| name.to_str()) == Some("assets"))
}

fn looks_like_flat_prototype_source(source: &str) -> bool {
    source
        .lines()
        .map(str::trim_start)
        .any(|line| line.starts_with("[nodes.") || line == "[nodes]")
}
