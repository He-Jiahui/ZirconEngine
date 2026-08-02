use std::collections::HashMap;

use crate::asset::ProjectAssetManager;
use crate::text::font::{shared_font_database_generation, FontDatabase};
use crate::text::FontFaceId;

use super::{resolve_font_face, DEFAULT_FONT_ASSET};

pub(super) const MAX_CACHED_FONT_ASSET_FACE_COUNT: usize = 128;

#[derive(Default)]
pub(super) struct SdfFontAssetFaceCache {
    faces: HashMap<(u64, String), Option<FontFaceId>>,
    recency: HashMap<(u64, String), u64>,
    access_epoch: u64,
}

impl SdfFontAssetFaceCache {
    pub(super) fn resolve(
        &mut self,
        font_asset: Option<&str>,
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> Option<FontFaceId> {
        let asset = font_asset
            .filter(|asset| !asset.trim().is_empty())
            .unwrap_or(DEFAULT_FONT_ASSET);
        let key = (shared_font_database_generation(), asset.to_owned());
        if let Some(face) = self.faces.get(&key).copied() {
            self.touch(key);
            return face;
        }

        let face = resolve_font_face(Some(asset), font_database, asset_manager);
        self.faces.insert(key.clone(), face);
        self.touch(key);
        self.enforce_budget();
        face
    }

    pub(super) fn len(&self) -> usize {
        self.faces.len()
    }

    pub(super) fn is_empty(&self) -> bool {
        self.faces.is_empty()
    }

    pub(super) fn contains(&self, asset: &str) -> bool {
        self.faces
            .contains_key(&(shared_font_database_generation(), asset.to_owned()))
    }

    pub(super) fn clear(&mut self) {
        self.faces.clear();
        self.recency.clear();
        self.access_epoch = 0;
    }

    fn touch(&mut self, key: (u64, String)) {
        self.access_epoch = self.access_epoch.saturating_add(1).max(1);
        self.recency.insert(key, self.access_epoch);
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
            self.faces.remove(&victim);
            self.recency.remove(&victim);
        }
    }
}
