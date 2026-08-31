use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use super::super::super::super::visual_assets::HostPaintImagePixels;

const MAX_AVATAR_MASK_CACHE_ENTRIES: usize = 64;
const MAX_AVATAR_MASK_CACHE_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) struct AvatarMaskCacheKey {
    resource_key: String,
    width: u32,
    height: u32,
    radius_bits: u32,
}

impl AvatarMaskCacheKey {
    pub(super) fn new(image: &HostPaintImagePixels, mask_radius: f32) -> Self {
        Self {
            resource_key: image.resource_key.clone(),
            width: image.width,
            height: image.height,
            radius_bits: mask_radius.to_bits(),
        }
    }
}

struct AvatarMaskCacheEntry {
    image: HostPaintImagePixels,
    byte_size: usize,
    last_used: u64,
}

#[derive(Default)]
struct AvatarMaskCache {
    entries: HashMap<AvatarMaskCacheKey, AvatarMaskCacheEntry>,
    resident_bytes: usize,
    access_clock: u64,
}

pub(super) fn cached_avatar_mask(key: &AvatarMaskCacheKey) -> Option<HostPaintImagePixels> {
    let cache = AVATAR_MASK_CACHE.get_or_init(|| Mutex::new(AvatarMaskCache::default()));
    cache
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .get(key)
}

pub(super) fn store_avatar_mask(key: AvatarMaskCacheKey, image: HostPaintImagePixels) {
    let cache = AVATAR_MASK_CACHE.get_or_init(|| Mutex::new(AvatarMaskCache::default()));
    cache
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .insert(key, image);
}

impl AvatarMaskCache {
    fn get(&mut self, key: &AvatarMaskCacheKey) -> Option<HostPaintImagePixels> {
        let last_used = self.next_access();
        self.entries.get_mut(key).map(|entry| {
            entry.last_used = last_used;
            entry.image.clone()
        })
    }

    fn insert(&mut self, key: AvatarMaskCacheKey, image: HostPaintImagePixels) {
        self.remove(&key);
        let byte_size = image.rgba.len();
        if byte_size > MAX_AVATAR_MASK_CACHE_BYTES {
            return;
        }
        while !self.entries.is_empty()
            && (self.entries.len() >= MAX_AVATAR_MASK_CACHE_ENTRIES
                || self.resident_bytes.saturating_add(byte_size) > MAX_AVATAR_MASK_CACHE_BYTES)
        {
            let Some(evicted_key) = self
                .entries
                .iter()
                .min_by_key(|(key, entry)| (entry.last_used, *key))
                .map(|(key, _)| key.clone())
            else {
                break;
            };
            self.remove(&evicted_key);
        }
        let last_used = self.next_access();
        self.resident_bytes = self.resident_bytes.saturating_add(byte_size);
        self.entries.insert(
            key,
            AvatarMaskCacheEntry {
                image,
                byte_size,
                last_used,
            },
        );
    }

    fn remove(&mut self, key: &AvatarMaskCacheKey) {
        if let Some(entry) = self.entries.remove(key) {
            self.resident_bytes = self.resident_bytes.saturating_sub(entry.byte_size);
        }
    }

    fn next_access(&mut self) -> u64 {
        self.access_clock = self.access_clock.saturating_add(1);
        self.access_clock
    }
}

static AVATAR_MASK_CACHE: OnceLock<Mutex<AvatarMaskCache>> = OnceLock::new();

#[cfg(test)]
mod hash_index_tests;

#[cfg(test)]
mod tests {
    use super::{AvatarMaskCache, AvatarMaskCacheKey, MAX_AVATAR_MASK_CACHE_ENTRIES};
    use crate::ui::retained_host::host_contract::paint_template_nodes::visual_assets::HostPaintImagePixels;

    fn image(resource_key: &str) -> HostPaintImagePixels {
        HostPaintImagePixels {
            resource_key: resource_key.to_string(),
            width: 1,
            height: 1,
            rgba: vec![255; 4].into(),
            atlas: None,
        }
    }

    #[test]
    fn cache_evicts_the_least_recently_used_mask_variant() {
        let mut cache = AvatarMaskCache::default();
        for index in 0..MAX_AVATAR_MASK_CACHE_ENTRIES {
            let image = image(format!("avatar-{index:03}").as_str());
            cache.insert(AvatarMaskCacheKey::new(&image, 1.0), image);
        }
        let oldest = AvatarMaskCacheKey::new(&image("avatar-000"), 1.0);
        assert!(cache.get(&oldest).is_some());

        let newest = image("avatar-new");
        let newest_key = AvatarMaskCacheKey::new(&newest, 1.0);
        cache.insert(newest_key.clone(), newest);

        assert_eq!(cache.entries.len(), MAX_AVATAR_MASK_CACHE_ENTRIES);
        assert!(cache.get(&oldest).is_some());
        assert!(cache
            .get(&AvatarMaskCacheKey::new(&image("avatar-001"), 1.0))
            .is_none());
        assert!(cache.get(&newest_key).is_some());
    }
}
