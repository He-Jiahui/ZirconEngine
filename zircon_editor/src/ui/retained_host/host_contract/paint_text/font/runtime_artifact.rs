use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, OnceLock};

use fontdue::Font;
use zircon_runtime::core::framework::text::{TextFontCollectionHandle, TextFontFaceHandle};
use zircon_runtime::ui::surface::UiTextGlyphArtifactRasterFace;

use super::{font_settings_for_collection_index, HostTextFont, HostTextFontSnapshot};
use crate::ui::retained_host::host_contract::paint_text::sync::lock_recovering_poison;

const RUNTIME_ARTIFACT_FONT_CACHE_CAPACITY: usize = 64;
const RUNTIME_ARTIFACT_FONT_FAMILY: &str = "Zircon Runtime Artifact";
const RUNTIME_ARTIFACT_FONT_WEIGHT: u16 = 400;
const RUNTIME_ARTIFACT_FONT_CACHE_DOMAIN: &[u8] = b"zircon-retained-runtime-artifact-font-v1";

/// Builds an editor raster snapshot from the exact runtime face selected by shaping.
///
/// The cache is intentionally bounded and keyed by the runtime source identity and generation.
/// `fontdue` cannot apply OpenType variation coordinates, so non-default instances fail closed and
/// leave the caller on its complete host-layout fallback rather than painting mismatched glyphs.
pub(in crate::ui::retained_host::host_contract) fn host_runtime_artifact_font_snapshot(
    runtime_face: &UiTextGlyphArtifactRasterFace,
) -> Option<HostTextFontSnapshot> {
    if runtime_face
        .variations()
        .is_some_and(|variations| !variations.0.is_empty())
    {
        return None;
    }

    let key = RuntimeArtifactFontKey {
        source_identity: runtime_face.source_identity(),
        font_generation: runtime_face.font_generation(),
        font_face: runtime_face.font_face(),
        font_instance: runtime_face.font_instance(),
        collection_index: runtime_face.collection_index(),
    };
    if let Some(font) = cached_runtime_artifact_font(key) {
        return Some(HostTextFontSnapshot { font });
    }

    let bytes = runtime_face.bytes();
    let font = Arc::new(HostTextFont {
        font: Some(Arc::new(
            Font::from_bytes(
                Arc::clone(&bytes),
                font_settings_for_collection_index(key.collection_index),
            )
            .ok()?,
        )),
        bytes,
        runtime_family: Arc::from(RUNTIME_ARTIFACT_FONT_FAMILY),
        weight: RUNTIME_ARTIFACT_FONT_WEIGHT,
        collection_index: key.collection_index,
        cache_key: runtime_artifact_font_cache_key(key),
    });

    Some(HostTextFontSnapshot {
        font: insert_runtime_artifact_font(key, font),
    })
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct RuntimeArtifactFontKey {
    source_identity: [u8; 16],
    font_generation: u64,
    font_face: TextFontFaceHandle,
    font_instance: Option<TextFontFaceHandle>,
    collection_index: u32,
}

fn cached_runtime_artifact_font(key: RuntimeArtifactFontKey) -> Option<Arc<HostTextFont>> {
    let cache = runtime_artifact_font_cache();
    let mut cache = lock_recovering_poison(cache);
    cache.get(key)
}

fn insert_runtime_artifact_font(
    key: RuntimeArtifactFontKey,
    font: Arc<HostTextFont>,
) -> Arc<HostTextFont> {
    let cache = runtime_artifact_font_cache();
    let mut cache = lock_recovering_poison(cache);
    cache.insert(key, font)
}

struct RuntimeArtifactFontCacheEntry {
    font: Arc<HostTextFont>,
    last_used: u64,
}

#[derive(Default)]
struct RuntimeArtifactFontCache {
    entries: HashMap<RuntimeArtifactFontKey, RuntimeArtifactFontCacheEntry>,
    access_generation: u64,
}

impl RuntimeArtifactFontCache {
    fn get(&mut self, key: RuntimeArtifactFontKey) -> Option<Arc<HostTextFont>> {
        let generation = self.next_access_generation();
        let entry = self.entries.get_mut(&key)?;
        entry.last_used = generation;
        Some(Arc::clone(&entry.font))
    }

    fn insert(
        &mut self,
        key: RuntimeArtifactFontKey,
        font: Arc<HostTextFont>,
    ) -> Arc<HostTextFont> {
        let generation = self.next_access_generation();
        if let Some(entry) = self.entries.get_mut(&key) {
            entry.last_used = generation;
            return Arc::clone(&entry.font);
        }
        if self.entries.len() >= RUNTIME_ARTIFACT_FONT_CACHE_CAPACITY {
            let least_recent_key = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.last_used)
                .map(|(key, _)| *key)
                .expect("a full runtime artifact font cache has an entry");
            self.entries.remove(&least_recent_key);
        }
        self.entries.insert(
            key,
            RuntimeArtifactFontCacheEntry {
                font: Arc::clone(&font),
                last_used: generation,
            },
        );
        font
    }

    fn next_access_generation(&mut self) -> u64 {
        if self.access_generation == u64::MAX {
            self.rebase_access_generations();
        }
        let generation = self.access_generation;
        self.access_generation += 1;
        generation
    }

    fn rebase_access_generations(&mut self) {
        let mut order = self
            .entries
            .iter()
            .map(|(key, entry)| (*key, entry.last_used))
            .collect::<Vec<_>>();
        order.sort_unstable_by_key(|(_, generation)| *generation);
        for (generation, (key, _)) in order.into_iter().enumerate() {
            self.entries
                .get_mut(&key)
                .expect("runtime artifact font cache key remains present")
                .last_used = generation as u64;
        }
        self.access_generation = self.entries.len() as u64;
    }
}

fn runtime_artifact_font_cache() -> &'static Mutex<RuntimeArtifactFontCache> {
    static CACHE: OnceLock<Mutex<RuntimeArtifactFontCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(RuntimeArtifactFontCache::default()))
}

fn runtime_artifact_font_cache_key(key: RuntimeArtifactFontKey) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    RUNTIME_ARTIFACT_FONT_CACHE_DOMAIN.hash(&mut hasher);
    key.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_FONT_COLLECTION: TextFontCollectionHandle = TextFontCollectionHandle::new(1);

    #[test]
    fn runtime_artifact_font_cache_key_keeps_generation_and_instance_identity() {
        let base = RuntimeArtifactFontKey {
            source_identity: [7; 16],
            font_generation: 12,
            font_face: TextFontFaceHandle::new(TEST_FONT_COLLECTION, 3, 12),
            font_instance: Some(TextFontFaceHandle::new(TEST_FONT_COLLECTION, 5, 12)),
            collection_index: 1,
        };
        let stale_generation = RuntimeArtifactFontKey {
            font_generation: 13,
            font_face: TextFontFaceHandle::new(TEST_FONT_COLLECTION, 3, 13),
            font_instance: Some(TextFontFaceHandle::new(TEST_FONT_COLLECTION, 5, 13)),
            ..base
        };
        let other_instance = RuntimeArtifactFontKey {
            font_instance: Some(TextFontFaceHandle::new(TEST_FONT_COLLECTION, 6, 12)),
            ..base
        };

        assert_ne!(
            runtime_artifact_font_cache_key(base),
            runtime_artifact_font_cache_key(stale_generation)
        );
        assert_ne!(
            runtime_artifact_font_cache_key(base),
            runtime_artifact_font_cache_key(other_instance)
        );
    }
}

#[cfg(test)]
#[path = "runtime_artifact/hash_lru_tests.rs"]
mod hash_lru_tests;
