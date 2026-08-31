use std::collections::HashMap;

#[cfg(test)]
use crate::asset::assets::FontSourceBudgetError;
use crate::text::FontFaceId;
use crate::text::font::FontDatabase;
#[cfg(test)]
use crate::text::font::{FontDatabaseError, FontLoadError, FontLoadIoFailure};

use super::DEFAULT_FONT_ASSET;

pub(super) const MAX_CACHED_FONT_ASSET_FACE_COUNT: usize = 128;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct SdfFontAssetCacheReport {
    pub(super) resident_error_count: usize,
    pub(super) resident_no_registered_faces_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SdfFontAssetLoadError {
    #[cfg(test)]
    Source(FontLoadError),
    #[cfg(test)]
    SourceReadFailed(FontLoadIoFailure),
    #[cfg(test)]
    SourceDecodeFailed,
    #[cfg(test)]
    SourceBudgetExceeded(FontSourceBudgetError),
    #[cfg(test)]
    RegistrationFailed,
    NoRegisteredFaces,
}

#[cfg(test)]
impl SdfFontAssetLoadError {
    pub(super) fn from_database_error(error: FontDatabaseError) -> Self {
        match error {
            FontDatabaseError::ReadFailed { source, .. } => {
                Self::SourceReadFailed(source.kind().into())
            }
            FontDatabaseError::SourceDecode { .. } => Self::SourceDecodeFailed,
            FontDatabaseError::SourceBudget { source, .. } => Self::SourceBudgetExceeded(source),
            _ => Self::RegistrationFailed,
        }
    }
}

#[derive(Default)]
pub(super) struct SdfFontAssetFaceCache {
    faces: HashMap<(u64, String), Result<FontFaceId, SdfFontAssetLoadError>>,
    report: SdfFontAssetCacheReport,
    recency: HashMap<(u64, String), u64>,
    access_epoch: u64,
}

impl SdfFontAssetFaceCache {
    pub(super) fn resolve(
        &mut self,
        font_generation: u64,
        font_asset: Option<&str>,
        font_database: &FontDatabase,
    ) -> Option<FontFaceId> {
        let asset = font_asset
            .filter(|asset| !asset.trim().is_empty())
            .unwrap_or(DEFAULT_FONT_ASSET);
        let key = (font_generation, asset.to_owned());
        if let Some(face) = self.faces.get(&key).copied() {
            self.touch(key);
            return face.ok();
        }

        let face = resolve_registered_font_face(asset, font_database);
        self.insert(key.clone(), face);
        self.touch(key);
        self.enforce_budget();
        face.ok()
    }

    pub(super) fn len(&self) -> usize {
        self.faces.len()
    }

    pub(super) fn is_empty(&self) -> bool {
        self.faces.is_empty()
    }

    pub(super) fn report(&self) -> SdfFontAssetCacheReport {
        self.report
    }

    pub(super) fn contains(&self, font_generation: u64, asset: &str) -> bool {
        self.faces
            .contains_key(&(font_generation, asset.to_owned()))
    }

    pub(super) fn clear(&mut self) {
        self.faces.clear();
        self.report = SdfFontAssetCacheReport::default();
        self.recency.clear();
        self.access_epoch = 0;
    }

    fn touch(&mut self, key: (u64, String)) {
        self.access_epoch = self.access_epoch.saturating_add(1).max(1);
        self.recency.insert(key, self.access_epoch);
    }

    fn insert(&mut self, key: (u64, String), face: Result<FontFaceId, SdfFontAssetLoadError>) {
        if let Err(error) = face {
            self.report.record_failure(error);
        }
        self.faces.insert(key, face);
    }

    fn enforce_budget(&mut self) {
        while self.faces.len() > MAX_CACHED_FONT_ASSET_FACE_COUNT {
            let Some(victim) = self
                .recency
                .iter()
                .min_by(|(left_key, left_epoch), (right_key, right_epoch)| {
                    left_epoch
                        .cmp(right_epoch)
                        .then_with(|| left_key.cmp(right_key))
                })
                .map(|(key, _)| key.clone())
            else {
                break;
            };
            if let Some(Err(error)) = self.faces.remove(&victim) {
                self.report.remove_failure(error);
            }
            self.recency.remove(&victim);
        }
    }
}

fn resolve_registered_font_face(
    asset: &str,
    font_database: &FontDatabase,
) -> Result<FontFaceId, SdfFontAssetLoadError> {
    if let Some(face) = font_database.font_asset_primary_face(asset) {
        return Ok(face);
    }
    if asset == DEFAULT_FONT_ASSET {
        if let Some(face) = font_database.runtime_default_primary_face() {
            return Ok(face);
        }
    }

    // Runtime rasterization is lookup-only. Source I/O and registration belong to the
    // collection admission owner before shaping/rasterization reaches this cache.
    Err(SdfFontAssetLoadError::NoRegisteredFaces)
}

impl SdfFontAssetCacheReport {
    fn record_failure(&mut self, error: SdfFontAssetLoadError) {
        self.adjust_failure(error, true);
    }

    fn remove_failure(&mut self, error: SdfFontAssetLoadError) {
        self.adjust_failure(error, false);
    }

    fn adjust_failure(&mut self, _error: SdfFontAssetLoadError, add: bool) {
        adjust_count(&mut self.resident_error_count, add);
        adjust_count(&mut self.resident_no_registered_faces_count, add);
    }
}

fn adjust_count(count: &mut usize, add: bool) {
    *count = if add {
        count.saturating_add(1)
    } else {
        count.saturating_sub(1)
    };
}
