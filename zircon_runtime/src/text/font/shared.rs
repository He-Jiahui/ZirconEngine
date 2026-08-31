use std::fmt;
#[cfg(test)]
use std::ops::Deref;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};
#[cfg(test)]
use std::sync::{MutexGuard, RwLockReadGuard};

use super::FontDatabase;
#[cfg(all(test, target_os = "windows"))]
use super::SystemFontPolicy;
use super::handle_registry::FontHandleRegistryService;
use super::runtime_asset::RuntimeFontAssetClaimRegistry;
use super::source_manifest::cooked_font_asset_source_key;
use crate::asset::{AssetUri, FontAsset, FontAssetSourceFormat, FontBlobArtifact};
use crate::core::framework::text::TextFontCollectionHandle;
use crate::text::FontFamilyName;

const PACKAGED_DEFAULT_FONT_OWNER: &str = "zircon.runtime.packaged-default-font";
const PACKAGED_DEFAULT_FONT_FAMILY: &str = "Fira Mono";
const PACKAGED_RUNTIME_FALLBACK_FAMILY: &str = "Zircon Runtime Fallback Mono";
const PACKAGED_DEFAULT_FONT_ASSET_URI: &str = super::DEFAULT_UI_FONT_ASSET;

static NEXT_FONT_COLLECTION_ID: AtomicU64 = AtomicU64::new(1);

fn allocate_font_collection_handle() -> TextFontCollectionHandle {
    let id = NEXT_FONT_COLLECTION_ID
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
            current.checked_add(1)
        })
        .expect("font collection identity space exhausted");
    TextFontCollectionHandle::new(id)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct FontCollectionRevision {
    collection_id: TextFontCollectionHandle,
    generation: u64,
}

impl FontCollectionRevision {
    pub(crate) const fn new(collection_id: TextFontCollectionHandle, generation: u64) -> Self {
        Self {
            collection_id,
            generation,
        }
    }

    pub(crate) const fn collection_id(self) -> TextFontCollectionHandle {
        self.collection_id
    }

    pub(crate) const fn generation(self) -> u64 {
        self.generation
    }
}

pub(crate) struct FontCollectionService {
    id: TextFontCollectionHandle,
    generation: AtomicU64,
    database: RwLock<Arc<FontDatabase>>,
    handle_registry: FontHandleRegistryService,
    pub(super) runtime_asset_claims: Mutex<RuntimeFontAssetClaimRegistry>,
}

impl fmt::Debug for FontCollectionService {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FontCollectionService")
            .field("collection", &self.id)
            .field("generation", &self.generation())
            .finish_non_exhaustive()
    }
}

impl PartialEq for FontCollectionService {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for FontCollectionService {}

/// Immutable publication of one exact font collection generation.
///
/// Keeping the database Arc alive is the resource lease for shaping work that
/// already started when a newer collection is published.
#[derive(Clone)]
pub(crate) struct FontCollectionSnapshot {
    collection: Arc<FontCollectionService>,
    generation: u64,
    database: Arc<FontDatabase>,
}

impl FontCollectionSnapshot {
    fn new(
        collection: Arc<FontCollectionService>,
        generation: u64,
        database: Arc<FontDatabase>,
    ) -> Self {
        Self {
            collection,
            generation,
            database,
        }
    }

    pub(crate) fn collection_id(&self) -> TextFontCollectionHandle {
        self.collection.id
    }

    pub(crate) const fn generation(&self) -> u64 {
        self.generation
    }

    pub(crate) fn revision(&self) -> FontCollectionRevision {
        FontCollectionRevision::new(self.collection_id(), self.generation)
    }

    pub(crate) fn service(&self) -> &FontCollectionService {
        &self.collection
    }

    pub(crate) fn service_handle(&self) -> Arc<FontCollectionService> {
        Arc::clone(&self.collection)
    }

    pub(crate) fn database(&self) -> &FontDatabase {
        &self.database
    }

    #[cfg(test)]
    pub(crate) fn shares_database_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.database, &other.database)
    }
}

impl fmt::Debug for FontCollectionSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FontCollectionSnapshot")
            .field("collection", &self.collection_id())
            .field("generation", &self.generation)
            .field("face_count", &self.database.face_count())
            .finish()
    }
}

impl FontCollectionService {
    pub(crate) fn new() -> Arc<Self> {
        Self::from_database(runtime_default_font_database())
    }

    pub(crate) fn from_database(database: FontDatabase) -> Arc<Self> {
        let id = allocate_font_collection_handle();
        Arc::new(Self {
            id,
            generation: AtomicU64::new(1),
            database: RwLock::new(Arc::new(database)),
            handle_registry: FontHandleRegistryService::new(id),
            runtime_asset_claims: Mutex::new(RuntimeFontAssetClaimRegistry::default()),
        })
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }

    pub(crate) const fn collection_id(&self) -> TextFontCollectionHandle {
        self.id
    }

    pub(crate) fn revision(&self) -> FontCollectionRevision {
        FontCollectionRevision::new(self.id, self.generation())
    }

    pub(crate) const fn handle_registry(&self) -> &FontHandleRegistryService {
        &self.handle_registry
    }

    pub(crate) fn collection_snapshot(self: &Arc<Self>) -> FontCollectionSnapshot {
        // A publisher increments the generation before releasing this same
        // lock, so a snapshot cannot pair a replacement database with the old
        // generation.
        let (generation, snapshot, face_count) = {
            crate::profile_scope!("runtime", "text.font_database", "shared_snapshot");
            let database = self
                .database
                .read()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let generation = self.generation.load(Ordering::Acquire);
            let face_count = database.face_count();
            (generation, Arc::clone(&*database), face_count)
        };
        crate::profile_counter!(
            "runtime",
            "text.font_database.snapshot_face_count",
            face_count
        );
        #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
        let _ = face_count;
        FontCollectionSnapshot::new(Arc::clone(self), generation, snapshot)
    }

    pub(crate) fn snapshot(self: &Arc<Self>) -> (u64, FontDatabase) {
        let snapshot = self.collection_snapshot();
        let database = {
            crate::profile_scope!(
                "runtime",
                "text.font_database",
                "shared_owned_snapshot_clone"
            );
            snapshot.database().clone()
        };
        (snapshot.generation(), database)
    }

    /// Apply one collection mutation and return a lease for the exact published
    /// database. Receipt-only callers can use this path without cloning the
    /// complete database after publication.
    pub(crate) fn mutate_published_snapshot<R>(
        self: &Arc<Self>,
        mutation: impl FnOnce(&mut FontDatabase) -> R,
    ) -> (FontCollectionSnapshot, R) {
        let (generation, published, result) = self.publish_mutation(mutation);
        (
            FontCollectionSnapshot::new(Arc::clone(self), generation, published),
            result,
        )
    }

    pub(crate) fn mutate<R>(
        &self,
        mutation: impl FnOnce(&mut FontDatabase) -> R,
    ) -> (u64, FontDatabase, R) {
        let (generation, published, result) = self.publish_mutation(mutation);
        let database = {
            crate::profile_scope!(
                "runtime",
                "text.font_database",
                "shared_owned_mutation_result_clone"
            );
            let database = published.as_ref().clone();
            crate::profile_counter!(
                "runtime",
                "text.font_database.mutation_result_clone_face_count",
                database.face_count()
            );
            database
        };
        (generation, database, result)
    }

    fn publish_mutation<R>(
        &self,
        mutation: impl FnOnce(&mut FontDatabase) -> R,
    ) -> (u64, Arc<FontDatabase>, R) {
        let (
            generation,
            published,
            result,
            before_face_count,
            after_face_count,
            render_inputs_changed,
        ) = {
            crate::profile_scope!("runtime", "text.font_database", "shared_mutation");
            let mut current = self
                .database
                .write()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let before = Arc::clone(&*current);
            let before_face_count = before.face_count();
            let mut next = {
                crate::profile_scope!(
                    "runtime",
                    "text.font_database",
                    "shared_mutation_outer_database_clone"
                );
                let next = before.as_ref().clone();
                crate::profile_counter!(
                    "runtime",
                    "text.font_database.mutation_outer_clone_face_count",
                    before_face_count
                );
                next
            };
            let result = mutation(&mut next);
            let render_inputs_changed = !before.has_same_render_inputs(&next);
            let generation = if render_inputs_changed {
                self.generation.fetch_add(1, Ordering::AcqRel) + 1
            } else {
                self.generation.load(Ordering::Acquire)
            };
            let after_face_count = next.face_count();
            let published = Arc::new(next);
            *current = Arc::clone(&published);
            (
                generation,
                published,
                result,
                before_face_count,
                after_face_count,
                render_inputs_changed,
            )
        };
        crate::profile_counter!(
            "runtime",
            "text.font_database.mutation_before_face_count",
            before_face_count
        );
        crate::profile_counter!(
            "runtime",
            "text.font_database.mutation_after_face_count",
            after_face_count
        );
        crate::profile_counter!(
            "runtime",
            "text.font_database.mutation_render_inputs_changed",
            u8::from(render_inputs_changed)
        );
        #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
        let _ = (before_face_count, after_face_count);
        (generation, published, result)
    }

    #[cfg(test)]
    fn force_publish(&self, database: &FontDatabase) -> u64 {
        let mut current = self
            .database
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *current = Arc::new(database.clone());
        self.generation.fetch_add(1, Ordering::AcqRel) + 1
    }
}

fn runtime_default_font_database() -> FontDatabase {
    let mut database = FontDatabase::with_default_fallbacks();
    // This fallback is part of the runtime image, not a development-tree source dependency.
    // Project fonts follow the artifact cache path; this permanent owner merely keeps headless
    // measurement usable before a project asset manager is attached.
    let asset = FontAsset::from_toml_str(include_str!("../../../assets/fonts/default.font.toml"))
        .expect("the compiled packaged default font manifest must remain valid");
    let blob = FontBlobArtifact::from_decoded_bytes(
        FontAssetSourceFormat::TrueTypeCollection,
        include_bytes!("../../../assets/fonts/ZirconDefaultComposite-subset.ttc").to_vec(),
    );
    let source_uri = AssetUri::parse(PACKAGED_DEFAULT_FONT_ASSET_URI)
        .expect("the packaged default font URI must remain valid");
    let registration = database
        .replace_font_asset_blob(
            PACKAGED_DEFAULT_FONT_OWNER,
            &asset,
            cooked_font_asset_source_key(&source_uri),
            &blob,
        )
        .expect("the compiled packaged default font blob must remain registrable");
    let primary_face = registration
        .faces
        .first()
        .copied()
        .expect("the packaged default font manifest must register a primary face");
    database.register_font_family_alias(
        primary_face,
        FontFamilyName::from(PACKAGED_RUNTIME_FALLBACK_FAMILY),
    );
    database.set_runtime_default_primary_face(primary_face);
    database.set_runtime_last_resort_face(primary_face);
    database.set_runtime_default_composite_font(asset.composite_font.clone());
    database.set_runtime_default_ui_family(
        asset
            .family
            .as_deref()
            .unwrap_or(PACKAGED_DEFAULT_FONT_FAMILY),
    );
    database
}

#[cfg(test)]
pub(crate) fn runtime_default_font_database_for_test() -> FontDatabase {
    runtime_default_font_database()
}

fn process_font_collection_service() -> &'static Arc<FontCollectionService> {
    static SERVICE: OnceLock<Arc<FontCollectionService>> = OnceLock::new();
    SERVICE.get_or_init(FontCollectionService::new)
}

pub(crate) fn shared_font_database_generation() -> u64 {
    process_font_collection_service().generation()
}

pub(crate) fn shared_font_database_snapshot() -> (u64, FontDatabase) {
    process_font_collection_service().snapshot()
}

pub(crate) fn shared_font_collection_snapshot() -> FontCollectionSnapshot {
    process_font_collection_service().collection_snapshot()
}

pub(crate) fn shared_font_collection_service() -> Arc<FontCollectionService> {
    Arc::clone(process_font_collection_service())
}

pub(crate) fn shared_font_collection_handle() -> TextFontCollectionHandle {
    process_font_collection_service().collection_id()
}

#[cfg(test)]
pub(crate) fn force_publish_shared_font_database(database: &FontDatabase) -> u64 {
    process_font_collection_service().force_publish(database)
}

#[cfg(test)]
pub(crate) struct FontCollectionServiceTestReadGuard {
    guard: RwLockReadGuard<'static, Arc<FontDatabase>>,
}

#[cfg(test)]
impl Deref for FontCollectionServiceTestReadGuard {
    type Target = FontDatabase;

    fn deref(&self) -> &Self::Target {
        self.guard.as_ref()
    }
}

#[cfg(test)]
pub(crate) fn shared_font_database_test_read_guard() -> (u64, FontCollectionServiceTestReadGuard) {
    let service = process_font_collection_service();
    let database = service
        .database
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let generation = service.generation.load(Ordering::Acquire);
    (
        generation,
        FontCollectionServiceTestReadGuard { guard: database },
    )
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
