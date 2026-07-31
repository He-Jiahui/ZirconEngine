use std::cell::RefCell;

use glyphon::cosmic_text::{fontdb, FontSystem};

use crate::text::font::{
    shared_font_database_generation, shared_font_database_snapshot, FontDatabase,
};
use crate::text::{default_text_locale, normalize_text_language_tag};

const MAX_LOCALE_FONT_SYSTEMS: usize = 4;

thread_local! {
    static FONT_SYSTEMS: RefCell<LocaleFontSystemCache> =
        RefCell::new(LocaleFontSystemCache::new());
}

struct LocaleFontSystemEntry {
    locale: String,
    font_system: FontSystem,
    last_used: u64,
}

struct LocaleFontSystemCache {
    seed_database: fontdb::Database,
    font_database: FontDatabase,
    font_database_generation: u64,
    fallback_locale: String,
    entries: Vec<LocaleFontSystemEntry>,
    use_order: u64,
}

impl LocaleFontSystemCache {
    fn new() -> Self {
        let (font_database_generation, font_database) = shared_font_database_snapshot();
        let seed_database = font_database.backend_database_snapshot();
        let fallback_locale = default_text_locale();
        // Keep cosmic on the process-level FontDatabase from first use.  Constructing a
        // standalone FontSystem here asks fontdb to rediscover the operating-system fonts,
        // which both violates the text-plan ownership boundary and stalls the first UI layout.
        let font_system =
            FontSystem::new_with_locale_and_db(fallback_locale.clone(), seed_database.clone());
        Self {
            seed_database,
            font_database,
            font_database_generation,
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
        language: Option<&str>,
        use_system: impl FnOnce(&mut FontSystem, &FontDatabase) -> R,
    ) -> R {
        self.refresh_shared_database();
        let locale =
            normalize_text_language_tag(language).unwrap_or_else(|| self.fallback_locale.clone());
        self.use_order = self.use_order.saturating_add(1);
        let index = self
            .entries
            .iter()
            .position(|entry| entry.locale == locale)
            .unwrap_or_else(|| self.insert_locale(locale));
        self.entries[index].last_used = self.use_order;
        use_system(&mut self.entries[index].font_system, &self.font_database)
    }

    fn refresh_shared_database(&mut self) {
        if shared_font_database_generation() == self.font_database_generation {
            return;
        }
        let (generation, database) = shared_font_database_snapshot();
        self.seed_database = database.backend_database_snapshot();
        self.font_database = database;
        self.font_database_generation = generation;
        for entry in &mut self.entries {
            entry.font_system = FontSystem::new_with_locale_and_db(
                entry.locale.clone(),
                self.seed_database.clone(),
            );
        }
    }

    fn insert_locale(&mut self, locale: String) -> usize {
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
        }
        self.entries.push(LocaleFontSystemEntry {
            locale: locale.clone(),
            font_system: FontSystem::new_with_locale_and_db(locale, self.seed_database.clone()),
            last_used: self.use_order,
        });
        self.entries.len() - 1
    }
}

pub(super) fn with_font_system<R>(
    language: Option<&str>,
    use_system: impl FnOnce(&mut FontSystem, &FontDatabase) -> R,
) -> R {
    FONT_SYSTEMS.with(|systems| systems.borrow_mut().with_font_system(language, use_system))
}
