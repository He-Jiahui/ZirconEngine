use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

const MAX_CIRCULAR_PROGRESS_RASTER_CACHE_ENTRIES: usize = 128;
const MAX_CIRCULAR_PROGRESS_RASTER_CACHE_BYTES: usize = 16 * 1024 * 1024;

thread_local! {
    static CIRCULAR_PROGRESS_RASTER_CACHE: RefCell<CircularProgressRasterCache> =
        RefCell::new(CircularProgressRasterCache::new());
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct CircularProgressRasterKey {
    source_size: u32,
    target_size_bits: u32,
    progress_bits: u32,
    track: [u8; 4],
    fill: [u8; 4],
}

impl CircularProgressRasterKey {
    pub(super) fn new(size: u32, progress: f32, track: [u8; 4], fill: [u8; 4]) -> Self {
        Self::with_target(size, size as f32, progress, track, fill)
    }

    pub(super) fn with_target(
        source_size: u32,
        target_size: f32,
        progress: f32,
        track: [u8; 4],
        fill: [u8; 4],
    ) -> Self {
        Self {
            source_size,
            target_size_bits: target_size.to_bits(),
            progress_bits: progress.to_bits(),
            track,
            fill,
        }
    }
}

struct CircularProgressRasterCacheEntry {
    resource_key: String,
    rgba: Arc<[u8]>,
    last_used: u64,
}

#[derive(Default)]
struct CircularProgressRasterCache {
    entries: HashMap<CircularProgressRasterKey, CircularProgressRasterCacheEntry>,
    resident_bytes: usize,
    access_generation: u64,
}

impl CircularProgressRasterCache {
    fn new() -> Self {
        Self::default()
    }

    fn get(&mut self, key: CircularProgressRasterKey) -> Option<CachedCircularProgressRaster> {
        let last_used = self.next_access_generation();
        let entry = self.entries.get_mut(&key)?;
        entry.last_used = last_used;
        Some(CachedCircularProgressRaster {
            resource_key: entry.resource_key.clone(),
            rgba: Arc::clone(&entry.rgba),
        })
    }

    fn insert(&mut self, key: CircularProgressRasterKey, resource_key: String, rgba: Arc<[u8]>) {
        let byte_size = rgba.len();
        if byte_size > MAX_CIRCULAR_PROGRESS_RASTER_CACHE_BYTES {
            return;
        }
        if let Some(replaced) = self.entries.remove(&key) {
            self.resident_bytes = self.resident_bytes.saturating_sub(replaced.rgba.len());
        }
        while !self.entries.is_empty()
            && (self.entries.len() >= MAX_CIRCULAR_PROGRESS_RASTER_CACHE_ENTRIES
                || self.resident_bytes.saturating_add(byte_size)
                    > MAX_CIRCULAR_PROGRESS_RASTER_CACHE_BYTES)
        {
            let Some(evicted_key) = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.last_used)
                .map(|(key, _)| *key)
            else {
                break;
            };
            if let Some(evicted) = self.entries.remove(&evicted_key) {
                self.resident_bytes = self.resident_bytes.saturating_sub(evicted.rgba.len());
            }
        }
        self.resident_bytes = self.resident_bytes.saturating_add(byte_size);
        let last_used = self.next_access_generation();
        self.entries.insert(
            key,
            CircularProgressRasterCacheEntry {
                resource_key,
                rgba,
                last_used,
            },
        );
    }

    fn next_access_generation(&mut self) -> u64 {
        if self.access_generation == u64::MAX {
            let mut oldest_first = self
                .entries
                .iter()
                .map(|(key, entry)| (*key, entry.last_used))
                .collect::<Vec<_>>();
            oldest_first.sort_unstable_by_key(|(_, last_used)| *last_used);
            for (index, (key, _)) in oldest_first.into_iter().enumerate() {
                self.entries
                    .get_mut(&key)
                    .expect("circular progress cache key remains present")
                    .last_used = index as u64 + 1;
            }
            self.access_generation = self.entries.len() as u64;
        }
        self.access_generation += 1;
        self.access_generation
    }
}

pub(super) struct CachedCircularProgressRaster {
    pub(super) resource_key: String,
    pub(super) rgba: Arc<[u8]>,
}

pub(super) fn cached_circular_progress_raster(
    key: CircularProgressRasterKey,
) -> Option<CachedCircularProgressRaster> {
    CIRCULAR_PROGRESS_RASTER_CACHE.with(|cache| cache.borrow_mut().get(key))
}

pub(super) fn store_circular_progress_raster(
    key: CircularProgressRasterKey,
    resource_key: String,
    rgba: Arc<[u8]>,
) {
    CIRCULAR_PROGRESS_RASTER_CACHE.with(|cache| {
        cache.borrow_mut().insert(key, resource_key, rgba);
    });
}

#[cfg(test)]
#[path = "tests/cache.rs"]
mod tests;

#[cfg(test)]
#[path = "cache/tests/hash_arc_tests.rs"]
mod hash_arc_tests;
