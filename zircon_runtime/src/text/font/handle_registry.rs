use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::time::{Duration, Instant};

use crate::core::framework::text::TextFontFaceHandle;
use crate::text::{FontFaceId, InstancedFaceId};

use super::shared_font_database_generation;

type BackendFontHandlePair = (Option<FontFaceId>, Option<InstancedFaceId>);
type TextFontHandlePair = (Option<TextFontFaceHandle>, Option<TextFontFaceHandle>);

/// Monotonic counters; a frame owner takes two snapshots to obtain its per-frame delta.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct FontHandleRegistryReport {
    pub registration_batch_count: u64,
    pub registration_lock_acquire_count: u64,
    pub registration_lock_wait_nanos: u64,
    pub registration_lock_hold_nanos: u64,
    pub registration_snapshot_publish_count: u64,
    pub registration_unique_pair_count: u64,
    pub registration_rejected_pair_count: u64,
    pub resolution_batch_count: u64,
    pub resolution_snapshot_acquire_count: u64,
    pub resolution_snapshot_wait_nanos: u64,
    pub resolution_snapshot_hold_nanos: u64,
    pub resolution_unique_pair_count: u64,
    pub resolution_rejected_pair_count: u64,
}

#[derive(Default)]
struct FontHandleRegistryMetrics {
    registration_batch_count: AtomicU64,
    registration_lock_acquire_count: AtomicU64,
    registration_lock_wait_nanos: AtomicU64,
    registration_lock_hold_nanos: AtomicU64,
    registration_snapshot_publish_count: AtomicU64,
    registration_unique_pair_count: AtomicU64,
    registration_rejected_pair_count: AtomicU64,
    resolution_batch_count: AtomicU64,
    resolution_snapshot_acquire_count: AtomicU64,
    resolution_snapshot_wait_nanos: AtomicU64,
    resolution_snapshot_hold_nanos: AtomicU64,
    resolution_unique_pair_count: AtomicU64,
    resolution_rejected_pair_count: AtomicU64,
}

impl FontHandleRegistryMetrics {
    fn report(&self) -> FontHandleRegistryReport {
        FontHandleRegistryReport {
            registration_batch_count: self.registration_batch_count.load(Ordering::Relaxed),
            registration_lock_acquire_count: self
                .registration_lock_acquire_count
                .load(Ordering::Relaxed),
            registration_lock_wait_nanos: self.registration_lock_wait_nanos.load(Ordering::Relaxed),
            registration_lock_hold_nanos: self.registration_lock_hold_nanos.load(Ordering::Relaxed),
            registration_snapshot_publish_count: self
                .registration_snapshot_publish_count
                .load(Ordering::Relaxed),
            registration_unique_pair_count: self
                .registration_unique_pair_count
                .load(Ordering::Relaxed),
            registration_rejected_pair_count: self
                .registration_rejected_pair_count
                .load(Ordering::Relaxed),
            resolution_batch_count: self.resolution_batch_count.load(Ordering::Relaxed),
            resolution_snapshot_acquire_count: self
                .resolution_snapshot_acquire_count
                .load(Ordering::Relaxed),
            resolution_snapshot_wait_nanos: self
                .resolution_snapshot_wait_nanos
                .load(Ordering::Relaxed),
            resolution_snapshot_hold_nanos: self
                .resolution_snapshot_hold_nanos
                .load(Ordering::Relaxed),
            resolution_unique_pair_count: self.resolution_unique_pair_count.load(Ordering::Relaxed),
            resolution_rejected_pair_count: self
                .resolution_rejected_pair_count
                .load(Ordering::Relaxed),
        }
    }
}

#[derive(Default)]
struct FontHandleRegistry {
    generation: u64,
    faces: Vec<FontFaceId>,
    face_slots: HashMap<FontFaceId, u32>,
    instances: Vec<InstancedFaceId>,
    instance_slots: HashMap<InstancedFaceId, u32>,
}

impl FontHandleRegistry {
    fn reset_for_generation(&mut self, generation: u64) -> bool {
        if self.generation == generation {
            return true;
        }
        if generation < self.generation {
            return false;
        }
        self.generation = generation;
        self.faces.clear();
        self.face_slots.clear();
        self.instances.clear();
        self.instance_slots.clear();
        true
    }

    fn register_face_for_current_generation(
        &mut self,
        face: FontFaceId,
    ) -> Option<TextFontFaceHandle> {
        let index = match self.face_slots.get(&face) {
            Some(index) => *index,
            None => {
                let index = u32::try_from(self.faces.len()).ok()?;
                self.faces.push(face);
                self.face_slots.insert(face, index);
                index
            }
        };
        Some(TextFontFaceHandle::new(index, self.generation))
    }

    fn register_instance_for_current_generation(
        &mut self,
        instance: InstancedFaceId,
    ) -> Option<TextFontFaceHandle> {
        let index = match self.instance_slots.get(&instance) {
            Some(index) => *index,
            None => {
                let index = u32::try_from(self.instances.len()).ok()?;
                self.instances.push(instance);
                self.instance_slots.insert(instance, index);
                index
            }
        };
        Some(TextFontFaceHandle::new(index, self.generation))
    }

    fn register_unique_pairs(
        &mut self,
        pairs: &[BackendFontHandlePair],
        generation: u64,
    ) -> Vec<TextFontHandlePair> {
        if !self.reset_for_generation(generation) {
            return vec![(None, None); pairs.len()];
        }
        pairs
            .iter()
            .map(|(face, instance)| {
                (
                    face.and_then(|face| self.register_face_for_current_generation(face)),
                    instance.and_then(|instance| {
                        self.register_instance_for_current_generation(instance)
                    }),
                )
            })
            .collect()
    }

    #[cfg(test)]
    fn resolve_face(&self, handle: TextFontFaceHandle) -> Option<FontFaceId> {
        (self.generation == handle.generation)
            .then(|| self.faces.get(handle.index as usize).copied())
            .flatten()
    }
}

#[derive(Default)]
struct FontHandleRegistrySnapshot {
    generation: u64,
    faces: Vec<FontFaceId>,
    instances: Vec<InstancedFaceId>,
}

impl From<&FontHandleRegistry> for FontHandleRegistrySnapshot {
    fn from(registry: &FontHandleRegistry) -> Self {
        Self {
            generation: registry.generation,
            faces: registry.faces.clone(),
            instances: registry.instances.clone(),
        }
    }
}

impl FontHandleRegistrySnapshot {
    fn resolve_face(&self, handle: TextFontFaceHandle) -> Option<FontFaceId> {
        (self.generation == handle.generation)
            .then(|| self.faces.get(handle.index as usize).copied())
            .flatten()
    }

    fn resolve_instance(&self, handle: TextFontFaceHandle) -> Option<InstancedFaceId> {
        (self.generation == handle.generation)
            .then(|| self.instances.get(handle.index as usize).copied())
            .flatten()
    }
}

fn registry() -> &'static Mutex<FontHandleRegistry> {
    static REGISTRY: OnceLock<Mutex<FontHandleRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(FontHandleRegistry::default()))
}

fn registry_snapshot() -> &'static RwLock<Arc<FontHandleRegistrySnapshot>> {
    static SNAPSHOT: OnceLock<RwLock<Arc<FontHandleRegistrySnapshot>>> = OnceLock::new();
    SNAPSHOT.get_or_init(|| RwLock::new(Arc::new(FontHandleRegistrySnapshot::default())))
}

fn metrics() -> &'static FontHandleRegistryMetrics {
    static METRICS: OnceLock<FontHandleRegistryMetrics> = OnceLock::new();
    METRICS.get_or_init(FontHandleRegistryMetrics::default)
}

fn duration_to_nanos(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

pub(crate) fn font_handle_registry_report() -> FontHandleRegistryReport {
    metrics().report()
}

fn publish_registry_snapshot(registry: &FontHandleRegistry) {
    *registry_snapshot()
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) =
        Arc::new(FontHandleRegistrySnapshot::from(registry));
}

fn current_registry_snapshot() -> (Arc<FontHandleRegistrySnapshot>, Duration, Duration) {
    let wait_started = Instant::now();
    let snapshot_guard = registry_snapshot()
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let wait = wait_started.elapsed();
    let hold_started = Instant::now();
    let snapshot = snapshot_guard.clone();
    let hold = hold_started.elapsed();
    drop(snapshot_guard);
    (snapshot, wait, hold)
}

fn snapshot_matches_font_database_generation(
    snapshot: &FontHandleRegistrySnapshot,
    expected_generation: u64,
    observed_generation: u64,
) -> bool {
    snapshot.generation == expected_generation && observed_generation == expected_generation
}

fn unique_backend_pairs(pairs: &[BackendFontHandlePair]) -> Vec<BackendFontHandlePair> {
    let mut seen = HashSet::with_capacity(pairs.len());
    let mut unique = Vec::new();
    for pair in pairs.iter().copied() {
        if (pair.0.is_some() || pair.1.is_some()) && seen.insert(pair) {
            unique.push(pair);
        }
    }
    unique
}

fn unique_text_pairs(pairs: &[TextFontHandlePair]) -> Vec<TextFontHandlePair> {
    let mut seen = HashSet::with_capacity(pairs.len());
    let mut unique = Vec::new();
    for pair in pairs.iter().copied() {
        if (pair.0.is_some() || pair.1.is_some()) && seen.insert(pair) {
            unique.push(pair);
        }
    }
    unique
}

pub(crate) fn register_font_face_handle(
    face: FontFaceId,
    generation: u64,
) -> Option<TextFontFaceHandle> {
    register_font_handle_batch(&[(Some(face), None)], generation)
        .into_iter()
        .next()
        .and_then(|(face, _)| face)
}

pub(crate) fn register_font_instance_handle(
    instance: InstancedFaceId,
    generation: u64,
) -> Option<TextFontFaceHandle> {
    register_font_handle_batch(&[(None, Some(instance))], generation)
        .into_iter()
        .next()
        .and_then(|(_, instance)| instance)
}

pub(crate) fn register_font_handles(
    face: Option<FontFaceId>,
    instance: Option<InstancedFaceId>,
    generation: u64,
) -> TextFontHandlePair {
    register_font_handle_batch(&[(face, instance)], generation)
        .into_iter()
        .next()
        .unwrap_or((None, None))
}

pub(crate) fn register_font_handle_batch(
    pairs: &[BackendFontHandlePair],
    generation: u64,
) -> Vec<TextFontHandlePair> {
    if pairs.is_empty() {
        return Vec::new();
    }
    let metrics = metrics();
    metrics
        .registration_batch_count
        .fetch_add(1, Ordering::Relaxed);
    let unique = unique_backend_pairs(pairs);
    if unique.is_empty() {
        return vec![(None, None); pairs.len()];
    }
    metrics
        .registration_unique_pair_count
        .fetch_add(unique.len() as u64, Ordering::Relaxed);
    metrics
        .registration_lock_acquire_count
        .fetch_add(1, Ordering::Relaxed);
    let registered = {
        let wait_started = Instant::now();
        let mut registry = registry()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        metrics
            .registration_lock_wait_nanos
            .fetch_add(duration_to_nanos(wait_started.elapsed()), Ordering::Relaxed);
        let hold_started = Instant::now();
        let previous_generation = registry.generation;
        let previous_face_count = registry.faces.len();
        let previous_instance_count = registry.instances.len();
        let registered = registry.register_unique_pairs(&unique, generation);
        if registry.generation != previous_generation
            || registry.faces.len() != previous_face_count
            || registry.instances.len() != previous_instance_count
        {
            publish_registry_snapshot(&registry);
            metrics
                .registration_snapshot_publish_count
                .fetch_add(1, Ordering::Relaxed);
        }
        metrics
            .registration_lock_hold_nanos
            .fetch_add(duration_to_nanos(hold_started.elapsed()), Ordering::Relaxed);
        registered
    };
    let registered_by_pair = unique
        .into_iter()
        .zip(registered)
        .collect::<HashMap<_, _>>();
    let result = pairs
        .iter()
        .map(|pair| {
            registered_by_pair
                .get(pair)
                .copied()
                .unwrap_or((None, None))
        })
        .collect::<Vec<_>>();
    let rejected_count = pairs
        .iter()
        .zip(&result)
        .filter(
            |((face, instance), (registered_face, registered_instance))| {
                (face.is_some() && registered_face.is_none())
                    || (instance.is_some() && registered_instance.is_none())
            },
        )
        .count();
    metrics
        .registration_rejected_pair_count
        .fetch_add(rejected_count as u64, Ordering::Relaxed);
    result
}

pub(crate) fn resolve_font_face_handle(handle: TextFontFaceHandle) -> Option<FontFaceId> {
    resolve_font_handle_batch(&[(Some(handle), None)])
        .into_iter()
        .next()
        .and_then(|(face, _)| face)
}

pub(crate) fn resolve_font_instance_handle(handle: TextFontFaceHandle) -> Option<InstancedFaceId> {
    resolve_font_handle_batch(&[(None, Some(handle))])
        .into_iter()
        .next()
        .and_then(|(_, instance)| instance)
}

pub(crate) fn resolve_font_handles(
    face: Option<TextFontFaceHandle>,
    instance: Option<TextFontFaceHandle>,
) -> BackendFontHandlePair {
    resolve_font_handle_batch(&[(face, instance)])
        .into_iter()
        .next()
        .unwrap_or((None, None))
}

pub(crate) fn resolve_font_handle_batch(
    pairs: &[TextFontHandlePair],
) -> Vec<BackendFontHandlePair> {
    if pairs.is_empty() {
        return Vec::new();
    }
    let metrics = metrics();
    metrics
        .resolution_batch_count
        .fetch_add(1, Ordering::Relaxed);
    let generation = shared_font_database_generation();
    let normalized = pairs
        .iter()
        .map(|(face, instance)| {
            (
                face.filter(|handle| handle.generation == generation),
                instance.filter(|handle| handle.generation == generation),
            )
        })
        .collect::<Vec<_>>();
    let unique = unique_text_pairs(&normalized);
    if unique.is_empty() {
        let rejected_count = pairs
            .iter()
            .filter(|(face, instance)| face.is_some() || instance.is_some())
            .count();
        metrics
            .resolution_rejected_pair_count
            .fetch_add(rejected_count as u64, Ordering::Relaxed);
        return vec![(None, None); pairs.len()];
    }
    metrics
        .resolution_unique_pair_count
        .fetch_add(unique.len() as u64, Ordering::Relaxed);
    metrics
        .resolution_snapshot_acquire_count
        .fetch_add(1, Ordering::Relaxed);
    let (snapshot, snapshot_wait, snapshot_hold) = current_registry_snapshot();
    metrics
        .resolution_snapshot_wait_nanos
        .fetch_add(duration_to_nanos(snapshot_wait), Ordering::Relaxed);
    metrics
        .resolution_snapshot_hold_nanos
        .fetch_add(duration_to_nanos(snapshot_hold), Ordering::Relaxed);
    // The shared database can publish between the first generation probe and
    // snapshot acquisition. Resolving through an older snapshot would revive
    // a handle from the retired font database, so defer the whole batch.
    if !snapshot_matches_font_database_generation(
        &snapshot,
        generation,
        shared_font_database_generation(),
    ) {
        let rejected_count = pairs
            .iter()
            .filter(|(face, instance)| face.is_some() || instance.is_some())
            .count();
        metrics
            .resolution_rejected_pair_count
            .fetch_add(rejected_count as u64, Ordering::Relaxed);
        return vec![(None, None); pairs.len()];
    }
    let resolved_by_pair = unique
        .into_iter()
        .map(|(face, instance)| {
            (
                (face, instance),
                (
                    face.and_then(|handle| snapshot.resolve_face(handle)),
                    instance.and_then(|handle| snapshot.resolve_instance(handle)),
                ),
            )
        })
        .collect::<HashMap<_, _>>();
    let result = normalized
        .iter()
        .map(|pair| resolved_by_pair.get(pair).copied().unwrap_or((None, None)))
        .collect::<Vec<_>>();
    let rejected_count = pairs
        .iter()
        .zip(&result)
        .filter(|((face, instance), (resolved_face, resolved_instance))| {
            (face.is_some() && resolved_face.is_none())
                || (instance.is_some() && resolved_instance.is_none())
        })
        .count();
    metrics
        .resolution_rejected_pair_count
        .fetch_add(rejected_count as u64, Ordering::Relaxed);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::font::shared::force_publish_shared_font_database;
    use crate::text::font::{
        shared_font_database_snapshot, shared_font_database_test_read_guard,
        shared_font_database_test_serial_guard,
    };

    #[test]
    fn generation_change_invalidates_old_slots_without_reinterpreting_backend_ids() {
        let mut registry = FontHandleRegistry::default();
        let backend_face = FontFaceId(u64::from(u32::MAX) + 41);
        let first = registry
            .register_unique_pairs(&[(Some(backend_face), None)], 9)
            .into_iter()
            .next()
            .and_then(|(face, _)| face)
            .expect("first handle");

        assert_eq!(registry.resolve_face(first), Some(backend_face));

        let reloaded = registry
            .register_unique_pairs(&[(Some(backend_face), None)], 10)
            .into_iter()
            .next()
            .and_then(|(face, _)| face)
            .expect("reloaded handle");
        assert_eq!(registry.resolve_face(first), None);
        assert_eq!(registry.resolve_face(reloaded), Some(backend_face));
        assert_eq!(reloaded.generation, 10);
    }

    #[test]
    fn shared_database_reload_rejects_pre_reload_handle() {
        let _shared_font_database = shared_font_database_test_serial_guard();
        let (generation, database) = shared_font_database_snapshot();
        let backend_face = FontFaceId(1);
        let before_reload = register_font_face_handle(backend_face, generation)
            .expect("pre-reload face should receive a slot");
        assert_eq!(resolve_font_face_handle(before_reload), Some(backend_face));

        let reloaded_generation = force_publish_shared_font_database(&database);

        assert!(reloaded_generation > generation);
        assert_eq!(resolve_font_face_handle(before_reload), None);
        let after_reload = register_font_face_handle(backend_face, reloaded_generation)
            .expect("reloaded face should receive a new-generation slot");
        assert_eq!(resolve_font_face_handle(after_reload), Some(backend_face));
        assert_ne!(before_reload, after_reload);
    }

    #[test]
    fn stale_projection_cannot_roll_registry_generation_back() {
        let mut registry = FontHandleRegistry::default();
        let current = registry
            .register_unique_pairs(&[(Some(FontFaceId(7)), None)], 12)
            .into_iter()
            .next()
            .and_then(|(face, _)| face)
            .expect("current generation handle");

        assert_eq!(
            registry.register_unique_pairs(&[(Some(FontFaceId(9)), None)], 11),
            vec![(None, None)]
        );
        assert_eq!(registry.generation, 12);
        assert_eq!(registry.resolve_face(current), Some(FontFaceId(7)));
    }

    #[test]
    fn registry_resolution_rejects_a_generation_change_after_its_initial_probe() {
        let mut registry = FontHandleRegistry::default();
        let handle = registry
            .register_unique_pairs(&[(Some(FontFaceId(31)), None)], 7)
            .into_iter()
            .next()
            .and_then(|(face, _)| face)
            .expect("current generation handle");
        let snapshot = FontHandleRegistrySnapshot::from(&registry);

        assert!(!snapshot_matches_font_database_generation(&snapshot, 7, 8));
        assert_eq!(snapshot.resolve_face(handle), Some(FontFaceId(31)));
    }

    #[test]
    fn paired_font_handles_roundtrip_face_and_instance_together() {
        let (generation, _database) = shared_font_database_test_read_guard();
        let face = FontFaceId(23);
        let instance = InstancedFaceId(29);

        let (face_handle, instance_handle) =
            register_font_handles(Some(face), Some(instance), generation);
        let resolved = resolve_font_handles(face_handle, instance_handle);

        assert_eq!(resolved, (Some(face), Some(instance)));
    }

    #[test]
    fn font_handle_batch_projection_and_resolution_deduplicate_repeated_pairs() {
        let _shared_font_database = shared_font_database_test_serial_guard();
        let (generation, _database) = shared_font_database_snapshot();
        let repeated = (Some(FontFaceId(1_001)), Some(InstancedFaceId(2_001)));
        let distinct = (Some(FontFaceId(1_002)), Some(InstancedFaceId(2_002)));
        let pairs = vec![repeated, repeated, distinct, repeated];

        let before_registration = font_handle_registry_report();
        let registered = register_font_handle_batch(&pairs, generation);
        let after_registration = font_handle_registry_report();

        assert_eq!(registered.len(), pairs.len());
        assert_eq!(registered[0], registered[1]);
        assert_eq!(registered[0], registered[3]);
        assert_ne!(registered[0], registered[2]);
        assert_eq!(
            after_registration.registration_batch_count,
            before_registration.registration_batch_count + 1
        );
        assert_eq!(
            after_registration.registration_lock_acquire_count,
            before_registration.registration_lock_acquire_count + 1
        );
        assert_eq!(
            after_registration.registration_snapshot_publish_count,
            before_registration.registration_snapshot_publish_count + 1
        );
        assert_eq!(
            after_registration.registration_unique_pair_count,
            before_registration.registration_unique_pair_count + 2
        );

        let before_resolution = font_handle_registry_report();
        let resolved = resolve_font_handle_batch(&registered);
        let after_resolution = font_handle_registry_report();

        assert_eq!(resolved, pairs);
        assert_eq!(
            after_resolution.resolution_batch_count,
            before_resolution.resolution_batch_count + 1
        );
        assert_eq!(
            after_resolution.resolution_snapshot_acquire_count,
            before_resolution.resolution_snapshot_acquire_count + 1
        );
        assert_eq!(
            after_resolution.resolution_unique_pair_count,
            before_resolution.resolution_unique_pair_count + 2
        );
    }

    #[test]
    fn repeated_font_handle_batch_does_not_republish_an_unchanged_snapshot() {
        let _shared_font_database = shared_font_database_test_serial_guard();
        let (_, database) = shared_font_database_snapshot();
        let generation = force_publish_shared_font_database(&database);
        let pairs = [(Some(FontFaceId(7_001)), Some(InstancedFaceId(8_001)))];

        let before = font_handle_registry_report();
        let first = register_font_handle_batch(&pairs, generation);
        let after_first = font_handle_registry_report();
        let second = register_font_handle_batch(&pairs, generation);
        let after_second = font_handle_registry_report();

        assert_eq!(first, second);
        assert_eq!(
            after_first.registration_snapshot_publish_count,
            before.registration_snapshot_publish_count + 1
        );
        assert_eq!(
            after_second.registration_snapshot_publish_count,
            after_first.registration_snapshot_publish_count
        );
        assert_eq!(
            after_second.registration_lock_acquire_count,
            after_first.registration_lock_acquire_count + 1
        );
    }
}
