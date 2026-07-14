use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use glyphon::{fontdb, FontSystem};

use crate::asset::assets::{
    decode_font_source, standalone_sfnt_face, FontFaceExtractionError, FontSourceDecodeError,
};
use crate::asset::FontAsset;
use crate::core::framework::render::{
    CompositeFontDescriptor, FontFaceDescriptor, FontFaceId, FontFamilyName, FontMatch, FontQuery,
    InstancedFaceId, VariationCoords,
};

#[cfg(test)]
use crate::core::framework::render::{FontStretch, FontStyle, FontWeight};

use super::asset_registration::{font_asset_descriptors, FontAssetSourceKey};
use super::backend::BackendFaceMap;
use super::coverage::FontCoverage;
use super::default_families::default_runtime_font_families;
use super::descriptors::{
    descriptor_from_font_bytes, descriptor_from_fontdb_face, source_key_from_fontdb_source,
};
use super::fallback::{MissingGlyphDiagnosticsReport, MissingGlyphLog};
use super::instance::{
    font_instance_identity, variations_for_face, variations_with_font_weight, FontInstance,
    FontInstanceError, FontInstanceRegistry,
};
use super::matching::{dedupe_families, stretch_distance, style_distance, weight_distance};

#[derive(Debug, thiserror::Error)]
pub(crate) enum FontDatabaseError {
    #[error("font family is empty")]
    EmptyFamily,
    #[error("font source contains no bytes")]
    EmptyBytes,
    #[error("font source {path} could not be read: {source}")]
    ReadFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("font source {path} could not be decoded: {source}")]
    SourceDecode {
        path: PathBuf,
        #[source]
        source: FontSourceDecodeError,
    },
    #[error("font face {face_index} could not be materialized: {source}")]
    FaceExtraction {
        face_index: u32,
        #[source]
        source: FontFaceExtractionError,
    },
    #[error("font face bytes are unavailable for {0:?}")]
    FaceBytesUnavailable(FontFaceId),
    #[error("font face is unknown: {0:?}")]
    UnknownFace(FontFaceId),
    #[error("font face has no shaping-backend identity: {0:?}")]
    BackendFaceUnavailable(FontFaceId),
    #[error(transparent)]
    FontInstance(#[from] FontInstanceError),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum SystemFontPolicy {
    #[default]
    Disabled,
    Discover,
}

pub(crate) struct FontShapingFaceResolver<'a> {
    database: &'a FontDatabase,
    primary: FontFaceId,
    fallback: super::fallback::FallbackResolver<'a>,
}

impl FontShapingFaceResolver<'_> {
    pub(crate) const fn primary_face(&self) -> FontFaceId {
        self.primary
    }

    pub(crate) fn primary_covers_all(&self, codepoints: &[char]) -> bool {
        self.database.face_covers_all(self.primary, codepoints)
    }

    pub(crate) fn resolve(
        &mut self,
        script: crate::core::framework::render::FontScript,
        codepoints: &[char],
    ) -> FontFaceId {
        self.fallback.resolve(self.primary, script, codepoints).face
    }
}

impl Drop for FontShapingFaceResolver<'_> {
    fn drop(&mut self) {
        self.database
            .missing_glyph_log()
            .append(self.fallback.take_diagnostics());
    }
}

#[derive(Clone, Debug)]
struct StoredFontFace {
    descriptor: FontFaceDescriptor,
    source: StoredFontSource,
    coverage: FontCoverage,
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
    family_index: HashMap<String, Vec<FontFaceId>>,
    source_face_index: HashMap<FontSourceKey, FontFaceId>,
    asset_source_index: HashMap<FontAssetSourceKey, FontFaceId>,
    fallback_families: Vec<FontFamilyName>,
    project_composite_font: Option<CompositeFontDescriptor>,
    backend_database: fontdb::Database,
    backend_faces: BackendFaceMap,
    instances: FontInstanceRegistry,
    default_instances: HashMap<FontFaceId, InstancedFaceId>,
    missing_glyphs: Arc<Mutex<MissingGlyphLog>>,
}

impl Default for FontDatabase {
    fn default() -> Self {
        Self {
            faces: Vec::new(),
            family_index: HashMap::new(),
            source_face_index: HashMap::new(),
            asset_source_index: HashMap::new(),
            fallback_families: Vec::new(),
            project_composite_font: None,
            backend_database: fontdb::Database::new(),
            backend_faces: BackendFaceMap::default(),
            instances: FontInstanceRegistry::default(),
            default_instances: HashMap::new(),
            missing_glyphs: Arc::new(Mutex::new(MissingGlyphLog::default())),
        }
    }
}

impl FontDatabase {
    pub(crate) fn with_default_fallbacks() -> Self {
        let mut database = Self::default();
        database.fallback_families = default_runtime_font_families();
        database
    }

    pub(crate) fn face_count(&self) -> usize {
        self.faces.len()
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
        let descriptor = descriptor_from_font_bytes(&bytes, family, source_path, face_index);
        self.register_stored_face(
            descriptor,
            Arc::from(bytes.into_boxed_slice()),
            Some(source_key.path),
        )
    }

    pub(crate) fn register_font_asset(
        &mut self,
        asset: &FontAsset,
        source_path: impl AsRef<Path>,
    ) -> Result<Vec<FontFaceId>, FontDatabaseError> {
        let source_path = source_path.as_ref();
        let bytes = read_decoded_font_source(source_path)?;
        let bytes: Arc<[u8]> = Arc::from(bytes.into_boxed_slice());
        let mut faces = Vec::new();

        for descriptor in font_asset_descriptors(asset, bytes.as_ref(), source_path) {
            let face =
                self.register_asset_descriptor(descriptor, Arc::clone(&bytes), source_path)?;
            if !faces.contains(&face) {
                faces.push(face);
            }
        }
        self.extend_fallback_families(asset.fallback_families.iter().map(String::as_str));
        Ok(faces)
    }

    pub(crate) fn set_project_composite_font(
        &mut self,
        composite: Option<CompositeFontDescriptor>,
    ) {
        self.project_composite_font = composite;
    }

    pub(crate) fn apply_system_font_policy(&mut self, policy: SystemFontPolicy) -> usize {
        if policy == SystemFontPolicy::Disabled {
            return 0;
        }
        let existing_backend_faces = self
            .backend_database
            .faces()
            .map(|face| face.id)
            .collect::<HashSet<_>>();
        self.backend_database.load_system_fonts();
        let system_faces = self
            .backend_database
            .faces()
            .filter(|face| !existing_backend_faces.contains(&face.id))
            .cloned()
            .collect::<Vec<_>>();
        let before = self.faces.len();
        for info in &system_faces {
            let _ = self.register_system_face(info);
        }
        self.faces.len().saturating_sub(before)
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

    pub(crate) fn set_default_ui_family(&mut self, family: &str) {
        self.backend_database
            .set_sans_serif_family(family.to_string());
        self.backend_database
            .set_monospace_family(family.to_string());
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
        let coverage = FontCoverage::from_sfnt_bytes(bytes.as_ref(), descriptor.face_index);
        self.register_stored_font_source(
            descriptor,
            StoredFontSource::SharedBytes(bytes),
            coverage,
            source_path,
        )
    }

    fn register_stored_font_source(
        &mut self,
        descriptor: FontFaceDescriptor,
        source: StoredFontSource,
        coverage: FontCoverage,
        source_path: Option<PathBuf>,
    ) -> Result<FontFaceId, FontDatabaseError> {
        self.register_stored_font_source_with_backend(
            descriptor,
            source,
            coverage,
            source_path,
            None,
        )
    }

    fn register_stored_font_source_with_backend(
        &mut self,
        descriptor: FontFaceDescriptor,
        source: StoredFontSource,
        coverage: FontCoverage,
        source_path: Option<PathBuf>,
        backend_face: Option<fontdb::ID>,
    ) -> Result<FontFaceId, FontDatabaseError> {
        if descriptor.family.is_empty() {
            return Err(FontDatabaseError::EmptyFamily);
        }
        if source.is_empty() {
            return Err(FontDatabaseError::EmptyBytes);
        }

        let id = FontFaceId(self.faces.len() as u64 + 1);
        let default_variations = match &source {
            StoredFontSource::SharedBytes(bytes) => variations_for_face(
                bytes.as_ref(),
                descriptor.face_index,
                &descriptor.variations,
                None,
            ),
            StoredFontSource::FontDb { .. } => descriptor.variations.clone(),
        };
        let default_instance = self.instances.resolve_or_insert(id, &default_variations)?;
        let family_key = normalized_family_key(descriptor.family.as_str());
        let face_index = descriptor.face_index;
        let backend_source = backend_face
            .is_none()
            .then(|| fontdb_source_from_stored(&source));
        self.faces.push(StoredFontFace {
            descriptor,
            source,
            coverage,
        });
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
                loaded.into_iter().find(|backend_face| {
                    self.backend_database
                        .face(*backend_face)
                        .is_some_and(|info| info.index == face_index)
                })
            })
        }) {
            self.backend_faces.insert(backend_face, id);
        }
        Ok(id)
    }

    fn register_asset_descriptor(
        &mut self,
        mut descriptor: FontFaceDescriptor,
        bytes: Arc<[u8]>,
        source_path: &Path,
    ) -> Result<FontFaceId, FontDatabaseError> {
        descriptor.variations = variations_for_face(
            bytes.as_ref(),
            descriptor.face_index,
            &descriptor.variations,
            None,
        );
        let source_key = FontAssetSourceKey::from_descriptor(source_path, &descriptor);
        if let Some(face) = self.asset_source_index.get(&source_key) {
            return Ok(*face);
        }
        let face = self.register_stored_face(descriptor, bytes, None)?;
        self.asset_source_index.insert(source_key, face);
        Ok(face)
    }

    fn extend_fallback_families<'a>(&mut self, families: impl IntoIterator<Item = &'a str>) {
        for family in families {
            let family = FontFamilyName::from(family);
            if family.is_empty() {
                continue;
            }
            let key = normalized_family_key(family.as_str());
            if self
                .fallback_families
                .iter()
                .any(|existing| normalized_family_key(existing.as_str()) == key)
            {
                continue;
            }
            self.fallback_families.push(family);
        }
    }

    pub(crate) fn match_face(&self, query: &FontQuery) -> Option<FontMatch> {
        let mut families = query.families.clone();
        families.extend(self.fallback_families.iter().cloned());
        self.match_face_in_family_order(&families, query)
    }

    pub(crate) fn fallback_candidates_for_codepoint(
        &self,
        codepoint: char,
        query: &FontQuery,
        composite: Option<&CompositeFontDescriptor>,
        language: Option<&str>,
    ) -> Vec<FontFaceId> {
        let composite = composite.or(self.project_composite_font.as_ref());
        super::fallback::FallbackResolver::new(self, query, composite, language)
            .candidates_for_codepoint(codepoint)
    }

    pub(crate) fn resolve_fallback_face_for_codepoint(
        &self,
        primary: FontFaceId,
        codepoint: char,
        query: &FontQuery,
        composite: Option<&CompositeFontDescriptor>,
        language: Option<&str>,
    ) -> FontFaceId {
        let composite = composite.or(self.project_composite_font.as_ref());
        let mut resolver = super::fallback::FallbackResolver::new(self, query, composite, language);
        let resolution = resolver.resolve_codepoint(primary, codepoint);
        self.missing_glyph_log().append(resolver.take_diagnostics());
        resolution.face
    }

    pub(crate) fn resolve_shaping_face_for_cluster(
        &self,
        script: crate::core::framework::render::FontScript,
        codepoints: &[char],
        query: &FontQuery,
        language: Option<&str>,
    ) -> Option<FontFaceId> {
        let mut resolver = self.begin_shaping_face_resolution(query, language)?;
        Some(resolver.resolve(script, codepoints))
    }

    pub(crate) fn begin_shaping_face_resolution<'a>(
        &'a self,
        query: &'a FontQuery,
        language: Option<&'a str>,
    ) -> Option<FontShapingFaceResolver<'a>> {
        let primary = self.match_face(query)?.face;
        Some(FontShapingFaceResolver {
            database: self,
            primary,
            fallback: super::fallback::FallbackResolver::new(
                self,
                query,
                self.project_composite_font.as_ref(),
                language,
            ),
        })
    }

    pub(crate) fn face_family_name(&self, face: FontFaceId) -> Option<FontFamilyName> {
        self.face(face).map(|face| face.descriptor.family.clone())
    }

    pub(crate) fn take_missing_glyph_diagnostics(&self) -> MissingGlyphDiagnosticsReport {
        self.missing_glyph_log().take_report()
    }

    pub(crate) fn face_bytes(&self, face: FontFaceId) -> Result<Arc<[u8]>, FontDatabaseError> {
        let stored = self
            .face(face)
            .ok_or(FontDatabaseError::UnknownFace(face))?;
        match &stored.source {
            StoredFontSource::SharedBytes(bytes) => Ok(Arc::clone(bytes)),
            StoredFontSource::FontDb { .. } => {
                let backend = self
                    .backend_face_id(face)
                    .ok_or(FontDatabaseError::BackendFaceUnavailable(face))?;
                self.backend_database
                    .with_face_data(backend, |bytes, _| Arc::<[u8]>::from(bytes))
                    .ok_or(FontDatabaseError::FaceBytesUnavailable(face))
            }
        }
    }

    pub(crate) fn face_index(&self, face: FontFaceId) -> Result<u32, FontDatabaseError> {
        Ok(self
            .face(face)
            .ok_or(FontDatabaseError::UnknownFace(face))?
            .descriptor
            .face_index)
    }

    pub(crate) fn standalone_face_bytes(
        &self,
        face: FontFaceId,
    ) -> Result<Arc<[u8]>, FontDatabaseError> {
        let bytes = self.face_bytes(face)?;
        let face_index = self.face_index(face)?;
        if face_index == 0 && !bytes.starts_with(b"ttcf") {
            return Ok(bytes);
        }
        standalone_sfnt_face(bytes.as_ref(), face_index)
            .map(|bytes| Arc::from(bytes.into_boxed_slice()))
            .map_err(|source| FontDatabaseError::FaceExtraction { face_index, source })
    }

    pub(crate) fn instance(
        &mut self,
        face: FontFaceId,
        variations: &VariationCoords,
    ) -> Result<InstancedFaceId, FontDatabaseError> {
        if self.face(face).is_none() {
            return Err(FontDatabaseError::UnknownFace(face));
        }
        let bytes = self.face_bytes(face)?;
        let face_index = self.face_index(face)?;
        let variations = variations_for_face(bytes.as_ref(), face_index, variations, None);
        self.instances
            .resolve_or_insert(face, &variations)
            .map_err(FontDatabaseError::from)
    }

    pub(crate) fn default_instance_id(
        &self,
        face: FontFaceId,
    ) -> Result<InstancedFaceId, FontDatabaseError> {
        self.default_instances
            .get(&face)
            .copied()
            .ok_or(FontDatabaseError::UnknownFace(face))
    }

    pub(crate) fn font_instance(&self, id: InstancedFaceId) -> Option<&FontInstance> {
        self.instances.get(id)
    }

    pub(crate) fn default_font_instance(
        &self,
        face: FontFaceId,
    ) -> Result<&FontInstance, FontDatabaseError> {
        let instance = self.default_instance_id(face)?;
        self.font_instance(instance)
            .ok_or(FontDatabaseError::UnknownFace(face))
    }

    pub(crate) fn effective_variations(
        &self,
        face: FontFaceId,
        font_weight: u16,
    ) -> Result<VariationCoords, FontDatabaseError> {
        self.effective_instance_variations(face, None, font_weight)
    }

    pub(crate) fn effective_instance_id(
        &self,
        face: FontFaceId,
        font_weight: u16,
    ) -> Result<InstancedFaceId, FontDatabaseError> {
        let variations = self.effective_variations(face, font_weight)?;
        font_instance_identity(face, &variations).map_err(FontDatabaseError::from)
    }

    pub(crate) fn effective_instance_variations(
        &self,
        face: FontFaceId,
        instance: Option<InstancedFaceId>,
        font_weight: u16,
    ) -> Result<VariationCoords, FontDatabaseError> {
        let bytes = self.face_bytes(face)?;
        let face_index = self.face_index(face)?;
        let base = instance
            .and_then(|instance| self.font_instance(instance))
            .filter(|instance| instance.face == face)
            .unwrap_or(self.default_font_instance(face)?);
        Ok(variations_with_font_weight(
            bytes.as_ref(),
            face_index,
            &base.variations,
            font_weight,
        ))
    }

    fn face(&self, face: FontFaceId) -> Option<&StoredFontFace> {
        let index = face.0.checked_sub(1)? as usize;
        self.faces.get(index)
    }

    fn match_face_in_family_order(
        &self,
        families: &[FontFamilyName],
        query: &FontQuery,
    ) -> Option<FontMatch> {
        dedupe_families(families.iter().cloned())
            .into_iter()
            .filter_map(|family| self.family_candidates(&family, query).into_iter().next())
            .next()
            .map(|face| FontMatch {
                face,
                synthetic_bold: false,
                synthetic_oblique: false,
            })
    }

    fn family_candidates(&self, family: &FontFamilyName, query: &FontQuery) -> Vec<FontFaceId> {
        let mut candidates = self
            .family_index
            .get(&normalized_family_key(family.as_str()))
            .cloned()
            .unwrap_or_default();
        candidates.sort_by_key(|id| self.match_score(*id, query));
        candidates
    }

    pub(super) fn family_candidates_for_codepoint(
        &self,
        family: &FontFamilyName,
        query: &FontQuery,
        codepoint: char,
    ) -> Vec<FontFaceId> {
        self.family_candidates(family, query)
            .into_iter()
            .filter(|face| self.face_covers_codepoint(*face, codepoint))
            .collect()
    }

    pub(super) fn face_covers_all(&self, face: FontFaceId, codepoints: &[char]) -> bool {
        codepoints
            .iter()
            .all(|codepoint| self.face_covers_codepoint(face, *codepoint))
    }

    pub(super) fn face_covers_codepoint(&self, face: FontFaceId, codepoint: char) -> bool {
        self.face(face)
            .is_some_and(|stored| stored.coverage.contains(codepoint))
    }

    pub(super) fn face_coverage_count(&self, face: FontFaceId, codepoints: &[char]) -> usize {
        codepoints
            .iter()
            .filter(|codepoint| self.face_covers_codepoint(face, **codepoint))
            .count()
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

    fn register_system_face(
        &mut self,
        info: &fontdb::FaceInfo,
    ) -> Result<Option<FontFaceId>, FontDatabaseError> {
        let Some(descriptor) = descriptor_from_fontdb_face(info) else {
            return Ok(None);
        };
        let Some(source_key) = source_key_from_fontdb_source(&info.source, info.index) else {
            return Ok(None);
        };
        if let Some(face) = self.source_face_index.get(&source_key) {
            return Ok(Some(*face));
        }

        let coverage = self
            .backend_database
            .with_face_data(info.id, |bytes, face_index| {
                FontCoverage::from_sfnt_bytes(bytes, face_index)
            })
            .unwrap_or(FontCoverage::Unknown);
        let id = self.register_stored_font_source_with_backend(
            descriptor,
            StoredFontSource::FontDb {
                source: info.source.clone(),
            },
            coverage,
            None,
            Some(info.id),
        )?;
        self.source_face_index.insert(source_key, id);
        Ok(Some(id))
    }

    fn match_score(&self, face: FontFaceId, query: &FontQuery) -> (u16, u16, u8) {
        let Some(stored) = self.face(face) else {
            return (u16::MAX, u16::MAX, u8::MAX);
        };
        (
            weight_distance(stored.descriptor.weight, query.weight),
            stretch_distance(stored.descriptor.stretch, query.stretch),
            style_distance(stored.descriptor.style, query.style),
        )
    }

    fn missing_glyph_log(&self) -> std::sync::MutexGuard<'_, MissingGlyphLog> {
        self.missing_glyphs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
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

#[cfg(test)]
mod tests;
