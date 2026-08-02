use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use glyphon::{FontSystem, fontdb};

use crate::asset::assets::decode_font_source;
use crate::text::{
    CompositeFontDescriptor, FontFaceDescriptor, FontFaceId, FontFamilyName, FontMatch,
    InstancedFaceId, VariationCoords,
};

use super::asset_registration::FontAssetSourceKey;
use super::backend::BackendFaceMap;
use super::composite_resolve::CompositeFontIndex;
use super::coverage::FontCoverage;
use super::default_families::default_runtime_font_families;
use super::descriptors::descriptor_from_font_metadata;
use super::face_metadata::FontFaceMetadata;
use super::fallback::MissingGlyphLog;
use super::fallback_cache::{CompositeFontIdentity, FallbackCaches};
use super::instance::{EffectiveInstanceCache, FontInstanceRegistry};
use super::matching::{FontFamilyIdentity, font_family_identity};

mod asset_lifecycle;
mod error;
mod face_access;
mod face_matching;
mod fallback_queries;
mod instances;
mod system_fonts;

pub(crate) use error::FontDatabaseError;
use face_matching::FaceMatchCache;
pub(crate) use fallback_queries::FontShapingFaceResolver;
pub(crate) use system_fonts::SystemFontPolicy;

#[derive(Clone, Debug)]
struct StoredFontFace {
    active: bool,
    descriptor: FontFaceDescriptor,
    source: StoredFontSource,
    source_bytes: Arc<OnceLock<Arc<[u8]>>>,
    standalone_bytes: Arc<OnceLock<Arc<[u8]>>>,
    metadata: Arc<OnceLock<FontFaceMetadata>>,
}

#[derive(Clone, Debug)]
enum StoredFontSource {
    SharedBytes(Arc<[u8]>),
    FontDb { source: fontdb::Source },
}

impl StoredFontSource {
    fn is_empty(&self) -> bool {
        match self {
            Self::SharedBytes(bytes) => bytes.is_empty(),
            Self::FontDb { .. } => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct FontSourceKey {
    path: PathBuf,
    face_index: u32,
}

impl FontSourceKey {
    pub(super) fn from_path(path: impl AsRef<Path>, face_index: u32) -> Self {
        Self {
            path: canonical_source_key(path.as_ref()),
            face_index,
        }
    }
}

#[derive(Clone, Debug)]
struct SharedFontBytes {
    bytes: Arc<[u8]>,
}

#[derive(Clone, Debug)]
pub(crate) struct FontDatabase {
    faces: Vec<StoredFontFace>,
    active_face_count: usize,
    family_index: HashMap<FontFamilyIdentity, Vec<FontFaceId>>,
    source_face_index: HashMap<FontSourceKey, FontFaceId>,
    asset_source_index: HashMap<FontAssetSourceKey, FontFaceId>,
    asset_source_owners: HashMap<FontAssetSourceKey, HashSet<String>>,
    asset_owners: HashMap<String, FontAssetOwnerState>,
    fallback_base_families: Vec<FontFamilyName>,
    fallback_families: Vec<FontFamilyName>,
    project_composite_font: Option<CompositeFontDescriptor>,
    project_composite_index: Option<(CompositeFontIdentity, Arc<CompositeFontIndex>)>,
    default_ui_family: Option<String>,
    // `fontdb::Database::load_system_fonts` appends a fresh catalog on every call.
    // Keep discovery process-local and idempotent for cloned renderer databases.
    system_fonts_discovered: bool,
    backend_database: fontdb::Database,
    backend_faces: BackendFaceMap,
    instances: FontInstanceRegistry,
    default_instances: HashMap<FontFaceId, InstancedFaceId>,
    metadata_build_count: Arc<AtomicU64>,
    missing_glyphs: Arc<Mutex<MissingGlyphLog>>,
    face_match_cache: Arc<Mutex<FaceMatchCache>>,
    effective_instances: EffectiveInstanceCache,
    fallback_caches: FallbackCaches,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct FontAssetOwnerState {
    sources: Vec<FontAssetSourceKey>,
    fallback_families: Vec<FontFamilyName>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct FontAssetUpdateReport {
    pub(crate) faces: Vec<FontFaceId>,
    pub(crate) retired_faces: Vec<FontFaceId>,
    pub(crate) database_changed: bool,
    pub(crate) asset_mapping_changed: bool,
}

impl Default for FontDatabase {
    fn default() -> Self {
        Self {
            faces: Vec::new(),
            active_face_count: 0,
            family_index: HashMap::new(),
            source_face_index: HashMap::new(),
            asset_source_index: HashMap::new(),
            asset_source_owners: HashMap::new(),
            asset_owners: HashMap::new(),
            fallback_base_families: Vec::new(),
            fallback_families: Vec::new(),
            project_composite_font: None,
            project_composite_index: None,
            default_ui_family: None,
            system_fonts_discovered: false,
            backend_database: fontdb::Database::new(),
            backend_faces: BackendFaceMap::default(),
            instances: FontInstanceRegistry::default(),
            default_instances: HashMap::new(),
            metadata_build_count: Arc::new(AtomicU64::new(0)),
            missing_glyphs: Arc::new(Mutex::new(MissingGlyphLog::default())),
            face_match_cache: Arc::new(Mutex::new(FaceMatchCache::default())),
            effective_instances: EffectiveInstanceCache::default(),
            fallback_caches: FallbackCaches::default(),
        }
    }
}

impl FontDatabase {
    pub(crate) fn with_default_fallbacks() -> Self {
        let mut database = Self::default();
        database.fallback_base_families = default_runtime_font_families();
        database.fallback_families = database.fallback_base_families.clone();
        database
    }

    pub(crate) fn face_count(&self) -> usize {
        self.active_face_count
    }

    pub(crate) fn fallback_families(&self) -> &[FontFamilyName] {
        &self.fallback_families
    }

    pub(crate) fn register_font_file(
        &mut self,
        source_path: impl AsRef<Path>,
        family: Option<&str>,
        face_index: u32,
    ) -> Result<FontFaceId, FontDatabaseError> {
        let source_path = source_path.as_ref();
        let source_key = FontSourceKey::from_path(source_path, face_index);
        if let Some(face) = self.source_face_index.get(&source_key) {
            return Ok(*face);
        }

        let bytes = read_decoded_font_source(source_path)?;
        let metadata = FontFaceMetadata::from_sfnt_bytes(&bytes, face_index);
        let descriptor = descriptor_from_font_metadata(&metadata, family, source_path, face_index);
        self.register_stored_face_with_metadata(
            descriptor,
            Arc::from(bytes.into_boxed_slice()),
            metadata,
            Some(source_key.path),
        )
    }

    pub(crate) fn set_project_composite_font(
        &mut self,
        composite: Option<CompositeFontDescriptor>,
    ) -> bool {
        if self.project_composite_font == composite {
            return false;
        }
        self.fallback_caches = FallbackCaches::default();
        let composite_index = composite
            .as_ref()
            .map(|descriptor| self.fallback_caches.composite_index(descriptor));
        self.project_composite_font = composite;
        self.project_composite_index = composite_index;
        true
    }

    pub(crate) fn load_face_into_font_system(
        &self,
        face: FontFaceId,
        font_system: &mut FontSystem,
    ) -> Result<(), FontDatabaseError> {
        if self.backend_face_id(face).is_none() {
            return Err(FontDatabaseError::BackendFaceUnavailable(face));
        }
        self.sync_font_system(font_system);
        Ok(())
    }

    pub(crate) fn sync_font_system(&self, font_system: &mut FontSystem) {
        let locale = font_system.locale().to_string();
        *font_system = FontSystem::new_with_locale_and_db(locale, self.backend_database.clone());
    }

    pub(crate) fn set_default_ui_family(&mut self, family: &str) -> bool {
        if self.default_ui_family.as_deref() == Some(family) {
            return false;
        }
        self.default_ui_family = Some(family.to_string());
        self.backend_database
            .set_sans_serif_family(family.to_string());
        self.backend_database
            .set_monospace_family(family.to_string());
        true
    }

    pub(crate) fn clear_default_ui_family(&mut self) -> bool {
        if self.default_ui_family.is_none() {
            return false;
        }
        self.default_ui_family = None;
        let defaults = fontdb::Database::new();
        self.backend_database
            .set_sans_serif_family(defaults.family_name(&fontdb::Family::SansSerif).to_string());
        self.backend_database
            .set_monospace_family(defaults.family_name(&fontdb::Family::Monospace).to_string());
        true
    }

    #[cfg(test)]
    pub(crate) fn default_ui_family_for_test(&self) -> Option<&str> {
        self.default_ui_family.as_deref()
    }

    #[cfg(test)]
    pub(crate) fn project_composite_font_for_test(&self) -> Option<&CompositeFontDescriptor> {
        self.project_composite_font.as_ref()
    }

    pub(crate) fn backend_database_snapshot(&self) -> fontdb::Database {
        self.backend_database.clone()
    }

    pub(crate) fn font_face_id(&self, backend: fontdb::ID) -> Option<FontFaceId> {
        self.backend_faces.font_face_id(backend)
    }

    pub(crate) fn backend_face_id(&self, face: FontFaceId) -> Option<fontdb::ID> {
        self.backend_faces.backend_face_id(face)
    }

    fn register_stored_face(
        &mut self,
        descriptor: FontFaceDescriptor,
        bytes: Arc<[u8]>,
        source_path: Option<PathBuf>,
    ) -> Result<FontFaceId, FontDatabaseError> {
        let metadata = FontFaceMetadata::from_sfnt_bytes(bytes.as_ref(), descriptor.face_index);
        self.register_stored_face_with_metadata(descriptor, bytes, metadata, source_path)
    }

    fn register_stored_face_with_metadata(
        &mut self,
        descriptor: FontFaceDescriptor,
        bytes: Arc<[u8]>,
        metadata: FontFaceMetadata,
        source_path: Option<PathBuf>,
    ) -> Result<FontFaceId, FontDatabaseError> {
        self.register_stored_font_source_with_backend(
            descriptor,
            StoredFontSource::SharedBytes(bytes),
            initialized_metadata(metadata),
            source_path,
            None,
            true,
        )
    }

    fn register_stored_font_source(
        &mut self,
        descriptor: FontFaceDescriptor,
        source: StoredFontSource,
        coverage: FontCoverage,
        source_path: Option<PathBuf>,
    ) -> Result<FontFaceId, FontDatabaseError> {
        let metadata = match &source {
            StoredFontSource::SharedBytes(bytes) => {
                FontFaceMetadata::from_sfnt_bytes(bytes.as_ref(), descriptor.face_index)
                    .with_coverage(coverage)
            }
            StoredFontSource::FontDb { .. } => {
                FontFaceMetadata::from_sfnt_bytes(&[], descriptor.face_index)
                    .with_coverage(coverage)
            }
        };
        self.register_stored_font_source_with_backend(
            descriptor,
            source,
            initialized_metadata(metadata),
            source_path,
            None,
            true,
        )
    }

    fn register_stored_font_source_with_backend(
        &mut self,
        descriptor: FontFaceDescriptor,
        source: StoredFontSource,
        metadata: Arc<OnceLock<FontFaceMetadata>>,
        source_path: Option<PathBuf>,
        backend_face: Option<fontdb::ID>,
        detach_derived_caches: bool,
    ) -> Result<FontFaceId, FontDatabaseError> {
        if descriptor.family.is_empty() {
            return Err(FontDatabaseError::EmptyFamily);
        }
        if source.is_empty() {
            return Err(FontDatabaseError::EmptyBytes);
        }

        let id = FontFaceId(self.faces.len() as u64 + 1);
        let default_variations = metadata.get().map_or_else(
            || descriptor.variations.clone(),
            |metadata| metadata.effective_variations(&descriptor.variations, None),
        );
        let default_instance = self.instances.resolve_or_insert(id, &default_variations)?;
        let family_key = font_family_identity(descriptor.family.as_str());
        let face_index = descriptor.face_index;
        let backend_source = backend_face
            .is_none()
            .then(|| fontdb_source_from_stored(&source));
        let source_bytes = match &source {
            StoredFontSource::SharedBytes(bytes) => initialized_face_bytes(Arc::clone(bytes)),
            StoredFontSource::FontDb { .. } => Arc::new(OnceLock::new()),
        };
        self.faces.push(StoredFontFace {
            active: true,
            descriptor,
            source,
            source_bytes,
            standalone_bytes: Arc::new(OnceLock::new()),
            metadata,
        });
        self.active_face_count = self.active_face_count.saturating_add(1);
        if self
            .faces
            .last()
            .is_some_and(|stored| stored.metadata.get().is_some())
        {
            self.metadata_build_count.fetch_add(1, Ordering::Relaxed);
        }
        self.default_instances.insert(id, default_instance);
        self.family_index.entry(family_key).or_default().push(id);
        if let Some(source_path) = source_path {
            self.source_face_index.insert(
                FontSourceKey {
                    path: source_path,
                    face_index,
                },
                id,
            );
        }
        if let Some(backend_face) = backend_face.or_else(|| {
            backend_source.and_then(|source| {
                let loaded = self.backend_database.load_font_source(source);
                let selected = loaded.iter().copied().find(|backend_face| {
                    self.backend_database
                        .face(*backend_face)
                        .is_some_and(|info| info.index == face_index)
                });
                for loaded_face in loaded {
                    if Some(loaded_face) != selected {
                        self.backend_database.remove_face(loaded_face);
                    }
                }
                selected
            })
        }) {
            self.backend_faces.insert(backend_face, id);
        }
        if detach_derived_caches {
            self.detach_face_dependent_caches();
        }
        Ok(id)
    }

    fn register_asset_descriptor(
        &mut self,
        descriptor: FontFaceDescriptor,
        bytes: Arc<[u8]>,
        source_path: &Path,
    ) -> Result<FontFaceId, FontDatabaseError> {
        let metadata = FontFaceMetadata::from_sfnt_bytes(bytes.as_ref(), descriptor.face_index);
        self.register_asset_registration(descriptor, metadata, bytes, source_path)
            .map(|(_, face)| face)
    }

    fn register_asset_registration(
        &mut self,
        mut descriptor: FontFaceDescriptor,
        metadata: FontFaceMetadata,
        bytes: Arc<[u8]>,
        source_path: &Path,
    ) -> Result<(FontAssetSourceKey, FontFaceId), FontDatabaseError> {
        descriptor.variations = metadata.effective_variations(&descriptor.variations, None);
        let source_key = FontAssetSourceKey::from_descriptor(
            source_path,
            &descriptor,
            metadata.source_identity(),
        );
        if let Some(face) = self.asset_source_index.get(&source_key) {
            return Ok((source_key, *face));
        }
        let face = self.register_stored_face_with_metadata(descriptor, bytes, metadata, None)?;
        self.asset_source_index.insert(source_key.clone(), face);
        Ok((source_key, face))
    }

    pub(crate) fn face_family_name(&self, face: FontFaceId) -> Option<FontFamilyName> {
        self.face(face).map(|face| face.descriptor.family.clone())
    }

    fn face(&self, face: FontFaceId) -> Option<&StoredFontFace> {
        let index = face.0.checked_sub(1)? as usize;
        self.faces.get(index).filter(|stored| stored.active)
    }

    #[cfg(test)]
    pub(crate) fn register_test_face(
        &mut self,
        descriptor: FontFaceDescriptor,
        bytes: Arc<[u8]>,
    ) -> Result<FontFaceId, FontDatabaseError> {
        self.register_stored_face(descriptor, bytes, None)
    }

    #[cfg(test)]
    pub(crate) fn register_test_face_with_coverage(
        &mut self,
        descriptor: FontFaceDescriptor,
        codepoints: &[char],
    ) -> Result<FontFaceId, FontDatabaseError> {
        self.register_stored_font_source(
            descriptor,
            StoredFontSource::SharedBytes(Arc::from([0_u8].as_slice())),
            FontCoverage::from_codepoints(codepoints),
            None,
        )
    }

    fn missing_glyph_log(&self) -> std::sync::MutexGuard<'_, MissingGlyphLog> {
        self.missing_glyphs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    // Immutable snapshots share hot caches; a mutating clone must own the next generation's caches.
    fn detach_face_dependent_caches(&mut self) {
        self.detach_matching_and_fallback_caches();
        self.effective_instances = EffectiveInstanceCache::default();
    }

    fn detach_matching_and_fallback_caches(&mut self) {
        self.face_match_cache = Arc::new(Mutex::new(FaceMatchCache::default()));
        self.fallback_caches = FallbackCaches::default();
        self.project_composite_index = self
            .project_composite_font
            .as_ref()
            .map(|descriptor| self.fallback_caches.composite_index(descriptor));
    }

    #[cfg(test)]
    pub(super) fn coverage_is_initialized(&self, face: FontFaceId) -> bool {
        self.face(face)
            .is_some_and(|stored| stored.metadata.get().is_some())
    }
}

fn initialized_metadata(metadata: FontFaceMetadata) -> Arc<OnceLock<FontFaceMetadata>> {
    let cell = OnceLock::new();
    let _ = cell.set(metadata);
    Arc::new(cell)
}

fn initialized_face_bytes(bytes: Arc<[u8]>) -> Arc<OnceLock<Arc<[u8]>>> {
    let cell = OnceLock::new();
    let _ = cell.set(bytes);
    Arc::new(cell)
}

fn fontdb_source_from_stored(source: &StoredFontSource) -> fontdb::Source {
    match source {
        StoredFontSource::SharedBytes(bytes) => fontdb::Source::Binary(Arc::new(SharedFontBytes {
            bytes: Arc::clone(bytes),
        })),
        StoredFontSource::FontDb { source } => source.clone(),
    }
}

impl AsRef<[u8]> for SharedFontBytes {
    fn as_ref(&self) -> &[u8] {
        self.bytes.as_ref()
    }
}

pub(super) fn normalized_family_key(family: &str) -> String {
    family.trim().to_ascii_lowercase()
}

pub(super) fn canonical_source_key(source_path: &Path) -> PathBuf {
    std::fs::canonicalize(source_path).unwrap_or_else(|_| source_path.to_path_buf())
}

fn read_decoded_font_source(source_path: &Path) -> Result<Vec<u8>, FontDatabaseError> {
    let bytes = std::fs::read(source_path).map_err(|source| FontDatabaseError::ReadFailed {
        path: source_path.to_path_buf(),
        source,
    })?;
    decode_font_source(bytes)
        .map(|source| source.into_bytes())
        .map_err(|source| FontDatabaseError::SourceDecode {
            path: source_path.to_path_buf(),
            source,
        })
}

mod equivalence;

#[cfg(test)]
mod tests;
