use std::sync::atomic::{AtomicU64, Ordering};
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
        Self {
            generation: AtomicU64::new(1),
            database: RwLock::new(database),
        }
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
    let shared = shared_database();
    loop {
        let before = shared.generation.load(Ordering::Acquire);
        let database = shared
            .database
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        let after = shared.generation.load(Ordering::Acquire);
        if before == after {
            return (after, database);
        }
    }
}

/// Publish the authoritative renderer lineage after project-font mutation.
///
/// Readers keep immutable clones and refresh by generation at a shaping-call
/// boundary, so no shaping hot path holds the process-wide lock.
pub(crate) fn publish_shared_font_database(database: &FontDatabase) -> u64 {
    let shared = shared_database();
    *shared
        .database
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = database.clone();
    shared.generation.fetch_add(1, Ordering::AcqRel) + 1
}
