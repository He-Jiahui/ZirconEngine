use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use glyphon::{fontdb, FontSystem};

use crate::asset::FontAsset;
use crate::core::framework::render::{
    CompositeFontDescriptor, FontFaceDescriptor, FontFaceId, FontFamilyName, FontMatch, FontQuery,
    FontScript, FontStretch, FontStyle, FontWeight, InstancedFaceId, VariationCoords,
};

use super::asset_registration::{font_asset_descriptors, FontAssetSourceKey};
use super::coverage::FontCoverage;
use super::default_families::default_runtime_font_families;
use super::descriptors::{
    descriptor_from_font_bytes, descriptor_from_fontdb_face, source_key_from_fontdb_source,
};
use super::matching::{dedupe_families, stretch_distance, style_distance, weight_distance};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FontDatabaseError {
    EmptyFamily,
    EmptyBytes,
    ReadFailed,
    FaceBytesUnavailable(FontFaceId),
    UnknownFace(FontFaceId),
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
}

impl Default for FontDatabase {
    fn default() -> Self {
        Self {
            faces: Vec::new(),
            family_index: HashMap::new(),
            source_face_index: HashMap::new(),
            asset_source_index: HashMap::new(),
            fallback_families: Vec::new(),
        }
    }
}

impl FontDatabase {
    pub(crate) fn with_default_fallbacks() -> Self {
        let mut database = Self::default();
        database.fallback_families = default_runtime_font_families();
        database
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

        let bytes = std::fs::read(source_path).map_err(|_| FontDatabaseError::ReadFailed)?;
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
        let bytes = std::fs::read(source_path).map_err(|_| FontDatabaseError::ReadFailed)?;
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

    pub(crate) fn load_system_fonts(&mut self) -> usize {
        let mut system_database = fontdb::Database::new();
        system_database.load_system_fonts();
        let before = self.faces.len();
        for info in system_database.faces() {
            let _ = self.register_system_face(info);
        }
        self.faces.len().saturating_sub(before)
    }

    pub(crate) fn load_face_into_font_system(
        &self,
        face: FontFaceId,
        font_system: &mut FontSystem,
    ) -> Result<(), FontDatabaseError> {
        let source = self.glyphon_source(face)?;
        font_system.db_mut().load_font_source(source);
        Ok(())
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
        if descriptor.family.is_empty() {
            return Err(FontDatabaseError::EmptyFamily);
        }
        if source.is_empty() {
            return Err(FontDatabaseError::EmptyBytes);
        }

        let id = FontFaceId(self.faces.len() as u64 + 1);
        let family_key = normalized_family_key(descriptor.family.as_str());
        let face_index = descriptor.face_index;
        self.faces.push(StoredFontFace {
            descriptor,
            source,
            coverage,
        });
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
        let _ = self.instance(id, &VariationCoords::default())?;
        Ok(id)
    }

    fn register_asset_descriptor(
        &mut self,
        descriptor: FontFaceDescriptor,
        bytes: Arc<[u8]>,
        source_path: &Path,
    ) -> Result<FontFaceId, FontDatabaseError> {
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

    #[cfg(test)]
    pub(crate) fn fallback_candidates(
        &self,
        codepoint: char,
        query: &FontQuery,
        composite: Option<&CompositeFontDescriptor>,
    ) -> Vec<FontFaceId> {
        super::fallback::FallbackResolver::new(self, query, composite)
            .candidates_for_codepoint(codepoint)
    }

    pub(crate) fn resolve_fallback_face_for_codepoint(
        &self,
        primary: FontFaceId,
        codepoint: char,
        query: &FontQuery,
        composite: Option<&CompositeFontDescriptor>,
    ) -> FontFaceId {
        super::fallback::FallbackResolver::new(self, query, composite)
            .resolve_codepoint(primary, codepoint)
            .face
    }

    pub(crate) fn resolve_fallback_face_for_cluster(
        &self,
        primary: FontFaceId,
        script: FontScript,
        codepoints: &[char],
        query: &FontQuery,
        composite: Option<&CompositeFontDescriptor>,
    ) -> FontFaceId {
        super::fallback::FallbackResolver::new(self, query, composite)
            .resolve(primary, script, codepoints)
            .face
    }

    pub(crate) fn face_bytes(&self, face: FontFaceId) -> Result<Arc<[u8]>, FontDatabaseError> {
        let stored = self
            .face(face)
            .ok_or(FontDatabaseError::UnknownFace(face))?;
        match &stored.source {
            StoredFontSource::SharedBytes(bytes) => Ok(Arc::clone(bytes)),
            StoredFontSource::FontDb { .. } => Err(FontDatabaseError::FaceBytesUnavailable(face)),
        }
    }

    pub(crate) fn face_index(&self, face: FontFaceId) -> Result<u32, FontDatabaseError> {
        Ok(self
            .face(face)
            .ok_or(FontDatabaseError::UnknownFace(face))?
            .descriptor
            .face_index)
    }

    pub(crate) fn instance(
        &self,
        face: FontFaceId,
        variations: &VariationCoords,
    ) -> Result<InstancedFaceId, FontDatabaseError> {
        if self.face(face).is_none() {
            return Err(FontDatabaseError::UnknownFace(face));
        }

        let mut hash = face.0.wrapping_mul(1_099_511_628_211);
        for (tag, value) in &variations.0 {
            hash ^= *tag as u64;
            hash = hash.wrapping_mul(1_099_511_628_211);
            hash ^= value.to_bits() as u64;
            hash = hash.wrapping_mul(1_099_511_628_211);
        }
        Ok(InstancedFaceId(hash))
    }

    fn face(&self, face: FontFaceId) -> Option<&StoredFontFace> {
        let index = face.0.checked_sub(1)? as usize;
        self.faces.get(index)
    }

    fn glyphon_source(&self, face: FontFaceId) -> Result<fontdb::Source, FontDatabaseError> {
        let stored = self
            .face(face)
            .ok_or(FontDatabaseError::UnknownFace(face))?;
        match &stored.source {
            StoredFontSource::SharedBytes(bytes) => {
                Ok(fontdb::Source::Binary(Arc::new(SharedFontBytes {
                    bytes: Arc::clone(bytes),
                })))
            }
            StoredFontSource::FontDb { source, .. } => Ok(source.clone()),
        }
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

    fn face_covers_codepoint(&self, face: FontFaceId, codepoint: char) -> bool {
        self.face(face)
            .is_some_and(|stored| stored.coverage.contains(codepoint))
    }

    #[cfg(test)]
    pub(crate) fn register_test_face(
        &mut self,
        descriptor: FontFaceDescriptor,
        bytes: Arc<[u8]>,
    ) -> Result<FontFaceId, FontDatabaseError> {
        self.register_stored_face(descriptor, bytes, None)
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

        let id = self.register_stored_font_source(
            descriptor,
            StoredFontSource::FontDb {
                source: info.source.clone(),
            },
            FontCoverage::Unknown,
            None,
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

#[cfg(test)]
mod tests;
