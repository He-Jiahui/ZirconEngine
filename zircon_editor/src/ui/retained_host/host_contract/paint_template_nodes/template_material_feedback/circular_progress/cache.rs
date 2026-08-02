use std::cell::RefCell;
use std::collections::VecDeque;

const MAX_CIRCULAR_PROGRESS_RASTER_CACHE_ENTRIES: usize = 128;
const MAX_CIRCULAR_PROGRESS_RASTER_CACHE_BYTES: usize = 16 * 1024 * 1024;

thread_local! {
    static CIRCULAR_PROGRESS_RASTER_CACHE: RefCell<CircularProgressRasterCache> =
        const { RefCell::new(CircularProgressRasterCache::new()) };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct CircularProgressRasterKey {
    size: u32,
    progress_bits: u32,
    track: [u8; 4],
    fill: [u8; 4],
}

impl CircularProgressRasterKey {
    pub(super) fn new(size: u32, progress: f32, track: [u8; 4], fill: [u8; 4]) -> Self {
        Self {
            size,
            progress_bits: progress.to_bits(),
            track,
            fill,
        }
    }
}

struct CircularProgressRasterCacheEntry {
    key: CircularProgressRasterKey,
    resource_key: String,
    rgba: Vec<u8>,
}

struct CircularProgressRasterCache {
    entries: VecDeque<CircularProgressRasterCacheEntry>,
    resident_bytes: usize,
}

impl CircularProgressRasterCache {
    const fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            resident_bytes: 0,
        }
    }

    fn get(&mut self, key: CircularProgressRasterKey) -> Option<CachedCircularProgressRaster> {
        let entry_index = self.entries.iter().position(|entry| entry.key == key)?;
        let entry = self
            .entries
            .remove(entry_index)
            .expect("located circular progress cache entry exists");
        let cached = CachedCircularProgressRaster {
            resource_key: entry.resource_key.clone(),
            rgba: entry.rgba.clone(),
        };
        self.entries.push_back(entry);
        Some(cached)
    }

    fn insert(&mut self, entry: CircularProgressRasterCacheEntry) {
        if entry.rgba.len() > MAX_CIRCULAR_PROGRESS_RASTER_CACHE_BYTES {
            return;
        }
        if let Some(entry_index) = self
            .entries
            .iter()
            .position(|cached| cached.key == entry.key)
        {
            let replaced = self
                .entries
                .remove(entry_index)
                .expect("located circular progress cache entry exists");
            self.resident_bytes = self.resident_bytes.saturating_sub(replaced.rgba.len());
        }
        while !self.entries.is_empty()
            && (self.entries.len() >= MAX_CIRCULAR_PROGRESS_RASTER_CACHE_ENTRIES
                || self.resident_bytes.saturating_add(entry.rgba.len())
                    > MAX_CIRCULAR_PROGRESS_RASTER_CACHE_BYTES)
        {
            if let Some(evicted) = self.entries.pop_front() {
                self.resident_bytes = self.resident_bytes.saturating_sub(evicted.rgba.len());
            }
        }
        self.resident_bytes = self.resident_bytes.saturating_add(entry.rgba.len());
        self.entries.push_back(entry);
    }
}

pub(super) struct CachedCircularProgressRaster {
    pub(super) resource_key: String,
    pub(super) rgba: Vec<u8>,
}

pub(super) fn cached_circular_progress_raster(
    key: CircularProgressRasterKey,
) -> Option<CachedCircularProgressRaster> {
    CIRCULAR_PROGRESS_RASTER_CACHE.with(|cache| cache.borrow_mut().get(key))
}

pub(super) fn store_circular_progress_raster(
    key: CircularProgressRasterKey,
    resource_key: String,
    rgba: Vec<u8>,
) {
    CIRCULAR_PROGRESS_RASTER_CACHE.with(|cache| {
        cache.borrow_mut().insert(CircularProgressRasterCacheEntry {
            key,
            resource_key,
            rgba,
        });
    });
}

#[cfg(test)]
mod tests {
    use super::{
        CircularProgressRasterCache, CircularProgressRasterCacheEntry, CircularProgressRasterKey,
        MAX_CIRCULAR_PROGRESS_RASTER_CACHE_ENTRIES,
    };

    fn key(index: u32) -> CircularProgressRasterKey {
        CircularProgressRasterKey::new(index, 0.5, [1; 4], [2; 4])
    }

    fn entry(index: u32) -> CircularProgressRasterCacheEntry {
        CircularProgressRasterCacheEntry {
            key: key(index),
            resource_key: format!("progress-{index}"),
            rgba: vec![index as u8],
        }
    }

    #[test]
    fn raster_cache_evicts_the_least_recently_used_variant() {
        let mut cache = CircularProgressRasterCache::new();
        for index in 0..MAX_CIRCULAR_PROGRESS_RASTER_CACHE_ENTRIES {
            cache.insert(entry(index as u32));
        }
        assert!(cache.get(key(0)).is_some());

        cache.insert(entry(u32::MAX));

        assert_eq!(
            cache.entries.len(),
            MAX_CIRCULAR_PROGRESS_RASTER_CACHE_ENTRIES
        );
        assert!(cache.get(key(0)).is_some());
        assert!(cache.get(key(1)).is_none());
        assert!(cache.get(key(u32::MAX)).is_some());
    }
}
