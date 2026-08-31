use std::collections::HashMap;

use crate::text::atlas::GlyphRasterKey;

use super::NativeBitmapAtlasCachedGlyphImage;

#[derive(Clone, Debug)]
pub(super) struct NativeBitmapAtlasSourceCacheEntry {
    pub(super) image: NativeBitmapAtlasCachedGlyphImage,
    pub(super) raster_key: Option<GlyphRasterKey>,
    previous: Option<GlyphRasterKey>,
    next: Option<GlyphRasterKey>,
}

#[derive(Debug, Default)]
pub(super) struct NativeBitmapAtlasSourceLru {
    head: Option<GlyphRasterKey>,
    tail: Option<GlyphRasterKey>,
}

impl NativeBitmapAtlasSourceLru {
    pub(super) fn clear(&mut self) {
        self.head = None;
        self.tail = None;
    }

    pub(super) fn remove(
        &mut self,
        entries: &mut HashMap<GlyphRasterKey, NativeBitmapAtlasSourceCacheEntry>,
        cache_key: GlyphRasterKey,
    ) -> (Option<NativeBitmapAtlasSourceCacheEntry>, bool) {
        if !entries.contains_key(&cache_key) {
            return (None, false);
        }
        let repaired = self.detach_or_repair(entries, cache_key);
        (entries.remove(&cache_key), repaired)
    }

    pub(super) fn insert_most_recent(
        &mut self,
        entries: &mut HashMap<GlyphRasterKey, NativeBitmapAtlasSourceCacheEntry>,
        cache_key: GlyphRasterKey,
        image: NativeBitmapAtlasCachedGlyphImage,
    ) -> bool {
        let replaced = entries.insert(
            cache_key,
            NativeBitmapAtlasSourceCacheEntry {
                image,
                raster_key: None,
                previous: None,
                next: None,
            },
        );
        let mut repaired = replaced.is_some();
        if repaired || self.attach_once(entries, cache_key).is_err() {
            self.rebuild(entries);
            repaired = true;
            self.move_to_most_recent_after_rebuild(entries, cache_key);
        }
        repaired
    }

    pub(super) fn pop_least_recent(
        &mut self,
        entries: &mut HashMap<GlyphRasterKey, NativeBitmapAtlasSourceCacheEntry>,
    ) -> (
        Option<(GlyphRasterKey, NativeBitmapAtlasSourceCacheEntry)>,
        bool,
    ) {
        let mut repaired = false;
        let cache_key = match self.head {
            Some(cache_key) if entries.contains_key(&cache_key) => cache_key,
            Some(_) => {
                self.rebuild(entries);
                repaired = true;
                let Some(cache_key) = self.head else {
                    return (None, repaired);
                };
                cache_key
            }
            None if entries.is_empty() => return (None, false),
            None => {
                self.rebuild(entries);
                repaired = true;
                let Some(cache_key) = self.head else {
                    return (None, repaired);
                };
                cache_key
            }
        };
        repaired |= self.detach_or_repair(entries, cache_key);
        (
            entries.remove(&cache_key).map(|entry| (cache_key, entry)),
            repaired,
        )
    }

    pub(super) fn touch(
        &mut self,
        entries: &mut HashMap<GlyphRasterKey, NativeBitmapAtlasSourceCacheEntry>,
        cache_key: GlyphRasterKey,
    ) -> (Option<NativeBitmapAtlasCachedGlyphImage>, bool) {
        let Some(image) = entries.get(&cache_key).map(|entry| entry.image.clone()) else {
            return (None, false);
        };
        let mut repaired = self.detach_or_repair(entries, cache_key);
        if self.attach_once(entries, cache_key).is_err() {
            self.rebuild(entries);
            repaired = true;
            self.move_to_most_recent_after_rebuild(entries, cache_key);
        }
        (Some(image), repaired)
    }

    fn detach_or_repair(
        &mut self,
        entries: &mut HashMap<GlyphRasterKey, NativeBitmapAtlasSourceCacheEntry>,
        cache_key: GlyphRasterKey,
    ) -> bool {
        if self.detach_once(entries, cache_key).is_ok() {
            return false;
        }
        self.rebuild(entries);
        let _ = self.detach_once(entries, cache_key);
        true
    }

    fn move_to_most_recent_after_rebuild(
        &mut self,
        entries: &mut HashMap<GlyphRasterKey, NativeBitmapAtlasSourceCacheEntry>,
        cache_key: GlyphRasterKey,
    ) {
        let _ = self.detach_once(entries, cache_key);
        let _ = self.attach_once(entries, cache_key);
    }

    fn attach_once(
        &mut self,
        entries: &mut HashMap<GlyphRasterKey, NativeBitmapAtlasSourceCacheEntry>,
        cache_key: GlyphRasterKey,
    ) -> Result<(), ()> {
        let Some(entry) = entries.get(&cache_key) else {
            return Err(());
        };
        if entry.previous.is_some() || entry.next.is_some() || self.head == Some(cache_key) {
            return Err(());
        }
        let previous = self.tail;
        match previous {
            Some(previous) => {
                let Some(previous_entry) = entries.get(&previous) else {
                    return Err(());
                };
                if previous == cache_key || previous_entry.next.is_some() {
                    return Err(());
                }
            }
            None if self.head.is_some() => return Err(()),
            None => {}
        }

        if let Some(previous) = previous {
            let Some(previous_entry) = entries.get_mut(&previous) else {
                return Err(());
            };
            previous_entry.next = Some(cache_key);
        } else {
            self.head = Some(cache_key);
        }
        let Some(entry) = entries.get_mut(&cache_key) else {
            return Err(());
        };
        entry.previous = previous;
        entry.next = None;
        self.tail = Some(cache_key);
        Ok(())
    }

    fn detach_once(
        &mut self,
        entries: &mut HashMap<GlyphRasterKey, NativeBitmapAtlasSourceCacheEntry>,
        cache_key: GlyphRasterKey,
    ) -> Result<(), ()> {
        let Some((previous, next)) = entries
            .get(&cache_key)
            .map(|entry| (entry.previous, entry.next))
        else {
            return Err(());
        };
        let previous_is_valid = previous.map_or(self.head == Some(cache_key), |previous| {
            entries
                .get(&previous)
                .is_some_and(|entry| entry.next == Some(cache_key))
        });
        let next_is_valid = next.map_or(self.tail == Some(cache_key), |next| {
            entries
                .get(&next)
                .is_some_and(|entry| entry.previous == Some(cache_key))
        });
        if !previous_is_valid || !next_is_valid {
            return Err(());
        }

        if let Some(previous) = previous {
            let Some(previous_entry) = entries.get_mut(&previous) else {
                return Err(());
            };
            previous_entry.next = next;
        } else {
            self.head = next;
        }
        if let Some(next) = next {
            let Some(next_entry) = entries.get_mut(&next) else {
                return Err(());
            };
            next_entry.previous = previous;
        } else {
            self.tail = previous;
        }
        let Some(entry) = entries.get_mut(&cache_key) else {
            return Err(());
        };
        entry.previous = None;
        entry.next = None;
        Ok(())
    }

    fn rebuild(
        &mut self,
        entries: &mut HashMap<GlyphRasterKey, NativeBitmapAtlasSourceCacheEntry>,
    ) {
        let cache_keys = entries.keys().copied().collect::<Vec<_>>();
        self.clear();
        for entry in entries.values_mut() {
            entry.previous = None;
            entry.next = None;
        }
        for cache_key in cache_keys {
            if self.attach_once(entries, cache_key).is_err() {
                self.clear();
                break;
            }
        }
    }

    #[cfg(test)]
    pub(super) fn corrupt_tail_for_test(&mut self, cache_key: GlyphRasterKey) {
        self.tail = Some(cache_key);
    }
}
