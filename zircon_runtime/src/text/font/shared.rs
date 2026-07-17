use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(test)]
use std::sync::RwLockReadGuard;
use std::sync::{OnceLock, RwLock};

use super::{FontDatabase, SystemFontPolicy};

struct SharedFontDatabase {
    generation: AtomicU64,
    database: RwLock<FontDatabase>,
}

impl SharedFontDatabase {
    fn new() -> Self {
        let mut database = FontDatabase::with_default_fallbacks();
        database.apply_system_font_policy(SystemFontPolicy::Discover);
        Self::from_database(database)
    }

    fn from_database(database: FontDatabase) -> Self {
        Self {
            generation: AtomicU64::new(1),
            database: RwLock::new(database),
        }
    }

    fn snapshot(&self) -> (u64, FontDatabase) {
        // A publisher increments the generation before releasing this same
        // lock, so a snapshot cannot pair a replacement database with the old
        // generation.
        let database = self
            .database
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let generation = self.generation.load(Ordering::Acquire);
        (generation, database.clone())
    }

    fn publish(&self, database: &FontDatabase) -> u64 {
        // Equality, replacement, and the generation transition share one
        // critical section. Repeated renderer construction therefore keeps
        // SDF/shaping caches resident when the effective font inputs did not
        // change, while a real mutation remains an atomic lineage change.
        let mut current = self
            .database
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let generation = self.generation.load(Ordering::Acquire);
        if current.has_same_render_inputs(database) {
            return generation;
        }
        *current = database.clone();
        self.generation.fetch_add(1, Ordering::AcqRel) + 1
    }

    #[cfg(test)]
    fn force_publish(&self, database: &FontDatabase) -> u64 {
        let mut current = self
            .database
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *current = database.clone();
        self.generation.fetch_add(1, Ordering::AcqRel) + 1
    }
}

fn shared_database() -> &'static SharedFontDatabase {
    static SHARED: OnceLock<SharedFontDatabase> = OnceLock::new();
    SHARED.get_or_init(SharedFontDatabase::new)
}

pub(crate) fn shared_font_database_generation() -> u64 {
    shared_database().generation.load(Ordering::Acquire)
}

pub(crate) fn shared_font_database_snapshot() -> (u64, FontDatabase) {
    shared_database().snapshot()
}

/// Publish the authoritative renderer lineage after project-font mutation.
///
/// Readers keep immutable clones and refresh by generation at a shaping-call
/// boundary, so no shaping hot path holds the process-wide lock.
pub(crate) fn publish_shared_font_database(database: &FontDatabase) -> u64 {
    shared_database().publish(database)
}

#[cfg(test)]
pub(super) fn force_publish_shared_font_database(database: &FontDatabase) -> u64 {
    shared_database().force_publish(database)
}

#[cfg(test)]
pub(crate) fn shared_font_database_test_read_guard() -> (u64, RwLockReadGuard<'static, FontDatabase>)
{
    let shared = shared_database();
    let database = shared
        .database
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let generation = shared.generation.load(Ordering::Acquire);
    (generation, database)
}

#[cfg(test)]
mod tests;
