use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(test)]
use std::sync::{Mutex, MutexGuard, RwLockReadGuard};
use std::sync::{OnceLock, RwLock};

use super::{FontDatabase, SystemFontPolicy};
use crate::asset::FontAsset;
use crate::text::FontFamilyName;

const PACKAGED_DEFAULT_FONT_OWNER: &str = "zircon.runtime.packaged-default-font";
const PACKAGED_DEFAULT_FONT_FAMILY: &str = "Fira Mono";
const PACKAGED_RUNTIME_FALLBACK_FAMILY: &str = "Zircon Runtime Fallback Mono";
const PACKAGED_DEFAULT_FONT_MANIFEST: &str = "assets/fonts/default.font.toml";

struct SharedFontDatabase {
    generation: AtomicU64,
    database: RwLock<FontDatabase>,
}

impl SharedFontDatabase {
    fn new() -> Self {
        Self::from_database(runtime_default_font_database())
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

    fn mutate<R>(&self, mutation: impl FnOnce(&mut FontDatabase) -> R) -> (u64, FontDatabase, R) {
        let mut current = self
            .database
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let before = current.clone();
        let result = mutation(&mut current);
        let render_inputs_changed = !before.has_same_render_inputs(&current);
        let generation = if render_inputs_changed {
            self.generation.fetch_add(1, Ordering::AcqRel) + 1
        } else {
            self.generation.load(Ordering::Acquire)
        };
        (generation, current.clone(), result)
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

fn runtime_default_font_database() -> FontDatabase {
    let mut database = FontDatabase::with_default_fallbacks();
    database.apply_system_font_policy(SystemFontPolicy::Discover);
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(PACKAGED_DEFAULT_FONT_MANIFEST);
    // Headless measurement can run before the GPU UI renderer loads the manifest. Register the
    // complete checked-in manifest under a permanent owner so the later UI owner shares its exact
    // face/source keys. Face 0 also has a private alias, keeping retained fallback distinct from a
    // same-named system Fira Mono face while user-selected typography keeps the normal family.
    if let Some(asset) = std::fs::read_to_string(&manifest_path)
        .ok()
        .and_then(|contents| FontAsset::from_toml_str(&contents).ok())
    {
        let source_path = manifest_path
            .parent()
            .map(|parent| parent.join(&asset.source));
        if let Some(source_path) = source_path {
            if let Ok(registration) =
                database.replace_font_asset(PACKAGED_DEFAULT_FONT_OWNER, &asset, source_path)
            {
                if let Some(face) = registration.faces.first().copied() {
                    database.register_font_family_alias(
                        face,
                        FontFamilyName::from(PACKAGED_RUNTIME_FALLBACK_FAMILY),
                    );
                }
            }
        }
    }
    database
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

pub(crate) fn mutate_shared_font_database<R>(
    mutation: impl FnOnce(&mut FontDatabase) -> R,
) -> (u64, FontDatabase, R) {
    shared_database().mutate(mutation)
}

#[cfg(test)]
pub(crate) fn force_publish_shared_font_database(database: &FontDatabase) -> u64 {
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
pub(crate) fn shared_font_database_test_serial_guard() -> MutexGuard<'static, ()> {
    // Shared database tests hold this across the complete mutation and observation window.
    static SERIAL: OnceLock<Mutex<()>> = OnceLock::new();
    SERIAL
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests;
