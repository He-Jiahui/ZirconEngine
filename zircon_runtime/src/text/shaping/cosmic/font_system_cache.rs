use std::cell::RefCell;

use glyphon::cosmic_text::{FontSystem, fontdb};

use crate::text::default_text_locale;
use crate::text::font::{FontCollectionSnapshot, FontDatabase};

const MAX_LOCALE_FONT_SYSTEMS: usize = 4;

thread_local! {
    static FONT_SYSTEMS: RefCell<Option<LocaleFontSystemCache>> = RefCell::new(None);
}

struct LocaleFontSystemEntry {
    locale: String,
    font_system: FontSystem,
    last_used: u64,
}

struct LocaleFontSystemCache {
    seed_database: fontdb::Database,
    font_collection: FontCollectionSnapshot,
    fallback_locale: String,
    entries: Vec<LocaleFontSystemEntry>,
    use_order: u64,
}

impl LocaleFontSystemCache {
    fn new(font_collection: &FontCollectionSnapshot) -> Self {
        let fallback_locale = default_text_locale();
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        let face_count = font_collection.database().face_count();
        let (seed_database, font_system) = {
            crate::profile_scope!("runtime", "text.font_system_cache", "thread_initialize");
            let seed_database = font_collection.database().backend_database_snapshot();
            // Keep cosmic on the process-level FontDatabase from first use. Constructing a
            // standalone FontSystem here asks fontdb to rediscover the operating-system fonts,
            // which both violates the text-plan ownership boundary and stalls the first UI layout.
            let font_system =
                FontSystem::new_with_locale_and_db(fallback_locale.clone(), seed_database.clone());
            (seed_database, font_system)
        };
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        crate::profile_counter!(
            "runtime",
            "text.font_system_cache.thread_initialize_face_count",
            face_count
        );
        Self {
            seed_database,
            font_collection: font_collection.clone(),
            fallback_locale: fallback_locale.clone(),
            entries: vec![LocaleFontSystemEntry {
                locale: fallback_locale,
                font_system,
                last_used: 1,
            }],
            use_order: 1,
        }
    }

    fn with_font_system<R>(
        &mut self,
        font_collection: &FontCollectionSnapshot,
        language: Option<&str>,
        use_system: impl FnOnce(&mut FontSystem, &FontDatabase) -> R,
    ) -> R {
        self.refresh_font_collection(font_collection);
        let locale = language.unwrap_or(self.fallback_locale.as_str());
        self.use_order = self.use_order.saturating_add(1);
        let existing_index = self.entries.iter().position(|entry| entry.locale == locale);
        let index = if let Some(index) = existing_index {
            index
        } else {
            let locale = locale.to_owned();
            self.insert_locale(locale)
        };
        self.entries[index].last_used = self.use_order;
        use_system(
            &mut self.entries[index].font_system,
            self.font_collection.database(),
        )
    }

    fn refresh_font_collection(&mut self, font_collection: &FontCollectionSnapshot) {
        // Equal generations have equivalent shaping/raster inputs even when a
        // diagnostic-only mutation published a different Arc.
        if self.font_collection.collection_id() == font_collection.collection_id()
            && self.font_collection.generation() == font_collection.generation()
        {
            return;
        }
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        let refreshed_entry_count = self.entries.len();
        {
            crate::profile_scope!("runtime", "text.font_system_cache", "generation_refresh");
            self.seed_database = font_collection.database().backend_database_snapshot();
            for entry in &mut self.entries {
                entry.font_system = FontSystem::new_with_locale_and_db(
                    entry.locale.clone(),
                    self.seed_database.clone(),
                );
            }
        }
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        let refreshed_face_count = font_collection.database().face_count();
        self.font_collection = font_collection.clone();
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        crate::profile_counter!(
            "runtime",
            "text.font_system_cache.generation_refresh_entry_count",
            refreshed_entry_count
        );
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        crate::profile_counter!(
            "runtime",
            "text.font_system_cache.generation_refresh_face_count",
            refreshed_face_count
        );
    }

    fn insert_locale(&mut self, locale: String) -> usize {
        let (index, evicted_entry) = {
            crate::profile_scope!("runtime", "text.font_system_cache", "locale_insert");
            let mut evicted_entry = false;
            if self.entries.len() >= MAX_LOCALE_FONT_SYSTEMS {
                let eviction_index = self
                    .entries
                    .iter()
                    .enumerate()
                    .filter(|(_, entry)| entry.locale != self.fallback_locale)
                    .min_by_key(|(_, entry)| entry.last_used)
                    .map(|(index, _)| index)
                    .unwrap_or(0);
                self.entries.remove(eviction_index);
                evicted_entry = true;
            }
            self.entries.push(LocaleFontSystemEntry {
                locale: locale.clone(),
                font_system: FontSystem::new_with_locale_and_db(locale, self.seed_database.clone()),
                last_used: self.use_order,
            });
            (self.entries.len() - 1, evicted_entry)
        };
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        let entry_count = self.entries.len();
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        crate::profile_counter!(
            "runtime",
            "text.font_system_cache.locale_insert_entry_count",
            entry_count
        );
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        crate::profile_counter!(
            "runtime",
            "text.font_system_cache.locale_insert_evicted_entry",
            u8::from(evicted_entry)
        );
        #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
        let _ = evicted_entry;
        index
    }
}

pub(super) fn with_font_system<R>(
    font_collection: &FontCollectionSnapshot,
    language: Option<&str>,
    use_system: impl FnOnce(&mut FontSystem, &FontDatabase) -> R,
) -> R {
    FONT_SYSTEMS.with(|systems| {
        let mut systems = systems.borrow_mut();
        let cache = systems.get_or_insert_with(|| LocaleFontSystemCache::new(font_collection));
        cache.with_font_system(font_collection, language, use_system)
    })
}

#[cfg(test)]
#[path = "font_system_cache/tests.rs"]
mod tests;
