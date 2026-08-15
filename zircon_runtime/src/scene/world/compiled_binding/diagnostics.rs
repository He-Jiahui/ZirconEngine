use std::sync::atomic::{AtomicU64, Ordering};

use crate::scene::world::World;

/// Cumulative work performed by text-to-typed scene-property binding paths.
///
/// The counters distinguish import/edit-boundary work from steady-state typed
/// access so performance tests can reject an accidental return to per-frame
/// path parsing or property-entry enumeration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CompiledScenePropertyAccessStats {
    pub path_lookup_requests: u64,
    pub path_entity_visits: u64,
    pub path_ancestor_visits: u64,
    pub path_sibling_visits: u64,
    pub canonicalization_bytes: u64,
    pub target_compilations: u64,
    pub field_dispatch_compilations: u64,
    pub compiled_reader_accesses: u64,
    pub compiled_writer_accesses: u64,
    pub stale_target_rejections: u64,
    pub property_entry_visits: u64,
}

impl CompiledScenePropertyAccessStats {
    pub fn saturating_delta_since(self, baseline: Self) -> Self {
        Self {
            path_lookup_requests: self
                .path_lookup_requests
                .saturating_sub(baseline.path_lookup_requests),
            path_entity_visits: self
                .path_entity_visits
                .saturating_sub(baseline.path_entity_visits),
            path_ancestor_visits: self
                .path_ancestor_visits
                .saturating_sub(baseline.path_ancestor_visits),
            path_sibling_visits: self
                .path_sibling_visits
                .saturating_sub(baseline.path_sibling_visits),
            canonicalization_bytes: self
                .canonicalization_bytes
                .saturating_sub(baseline.canonicalization_bytes),
            target_compilations: self
                .target_compilations
                .saturating_sub(baseline.target_compilations),
            field_dispatch_compilations: self
                .field_dispatch_compilations
                .saturating_sub(baseline.field_dispatch_compilations),
            compiled_reader_accesses: self
                .compiled_reader_accesses
                .saturating_sub(baseline.compiled_reader_accesses),
            compiled_writer_accesses: self
                .compiled_writer_accesses
                .saturating_sub(baseline.compiled_writer_accesses),
            stale_target_rejections: self
                .stale_target_rejections
                .saturating_sub(baseline.stale_target_rejections),
            property_entry_visits: self
                .property_entry_visits
                .saturating_sub(baseline.property_entry_visits),
        }
    }
}

#[derive(Debug, Default)]
pub(in super::super) struct CompiledScenePropertyAccessDiagnostics {
    path_lookup_requests: AtomicU64,
    path_entity_visits: AtomicU64,
    path_ancestor_visits: AtomicU64,
    path_sibling_visits: AtomicU64,
    canonicalization_bytes: AtomicU64,
    target_compilations: AtomicU64,
    field_dispatch_compilations: AtomicU64,
    compiled_reader_accesses: AtomicU64,
    compiled_writer_accesses: AtomicU64,
    stale_target_rejections: AtomicU64,
    property_entry_visits: AtomicU64,
}

impl CompiledScenePropertyAccessDiagnostics {
    fn snapshot(&self) -> CompiledScenePropertyAccessStats {
        CompiledScenePropertyAccessStats {
            path_lookup_requests: self.path_lookup_requests.load(Ordering::Relaxed),
            path_entity_visits: self.path_entity_visits.load(Ordering::Relaxed),
            path_ancestor_visits: self.path_ancestor_visits.load(Ordering::Relaxed),
            path_sibling_visits: self.path_sibling_visits.load(Ordering::Relaxed),
            canonicalization_bytes: self.canonicalization_bytes.load(Ordering::Relaxed),
            target_compilations: self.target_compilations.load(Ordering::Relaxed),
            field_dispatch_compilations: self.field_dispatch_compilations.load(Ordering::Relaxed),
            compiled_reader_accesses: self.compiled_reader_accesses.load(Ordering::Relaxed),
            compiled_writer_accesses: self.compiled_writer_accesses.load(Ordering::Relaxed),
            stale_target_rejections: self.stale_target_rejections.load(Ordering::Relaxed),
            property_entry_visits: self.property_entry_visits.load(Ordering::Relaxed),
        }
    }

    fn reset(&self) {
        self.path_lookup_requests.store(0, Ordering::Relaxed);
        self.path_entity_visits.store(0, Ordering::Relaxed);
        self.path_ancestor_visits.store(0, Ordering::Relaxed);
        self.path_sibling_visits.store(0, Ordering::Relaxed);
        self.canonicalization_bytes.store(0, Ordering::Relaxed);
        self.target_compilations.store(0, Ordering::Relaxed);
        self.field_dispatch_compilations.store(0, Ordering::Relaxed);
        self.compiled_reader_accesses.store(0, Ordering::Relaxed);
        self.compiled_writer_accesses.store(0, Ordering::Relaxed);
        self.stale_target_rejections.store(0, Ordering::Relaxed);
        self.property_entry_visits.store(0, Ordering::Relaxed);
    }

    fn record(&self, counter: &AtomicU64, amount: u64) {
        counter.fetch_add(amount, Ordering::Relaxed);
    }
}

// Diagnostics are runtime-only observations and deliberately do not affect
// persistent-world equality.
impl PartialEq for CompiledScenePropertyAccessDiagnostics {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl World {
    pub fn compiled_scene_property_access_stats(&self) -> CompiledScenePropertyAccessStats {
        self.compiled_scene_property_access_diagnostics.snapshot()
    }

    pub fn reset_compiled_scene_property_access_stats(&self) {
        self.compiled_scene_property_access_diagnostics.reset();
    }

    pub(in crate::scene::world) fn record_scene_property_path_lookup(&self) {
        self.compiled_scene_property_access_diagnostics.record(
            &self
                .compiled_scene_property_access_diagnostics
                .path_lookup_requests,
            1,
        );
    }

    pub(in crate::scene::world) fn record_scene_property_path_entity_visit(&self) {
        self.compiled_scene_property_access_diagnostics.record(
            &self
                .compiled_scene_property_access_diagnostics
                .path_entity_visits,
            1,
        );
    }

    pub(in crate::scene::world) fn record_scene_property_path_ancestor_visit(&self) {
        self.compiled_scene_property_access_diagnostics.record(
            &self
                .compiled_scene_property_access_diagnostics
                .path_ancestor_visits,
            1,
        );
    }

    pub(in crate::scene::world) fn record_scene_property_path_sibling_visit(&self) {
        self.compiled_scene_property_access_diagnostics.record(
            &self
                .compiled_scene_property_access_diagnostics
                .path_sibling_visits,
            1,
        );
    }

    pub(in crate::scene::world) fn record_scene_property_canonicalization(&self, bytes: usize) {
        self.compiled_scene_property_access_diagnostics.record(
            &self
                .compiled_scene_property_access_diagnostics
                .canonicalization_bytes,
            bytes as u64,
        );
    }

    pub(in crate::scene::world) fn record_scene_property_target_compilation(&self) {
        self.compiled_scene_property_access_diagnostics.record(
            &self
                .compiled_scene_property_access_diagnostics
                .target_compilations,
            1,
        );
    }

    pub(in crate::scene::world) fn record_scene_property_field_dispatch_compilation(&self) {
        self.compiled_scene_property_access_diagnostics.record(
            &self
                .compiled_scene_property_access_diagnostics
                .field_dispatch_compilations,
            1,
        );
    }

    pub(in crate::scene::world) fn record_compiled_scene_property_reader_access(&self) {
        self.compiled_scene_property_access_diagnostics.record(
            &self
                .compiled_scene_property_access_diagnostics
                .compiled_reader_accesses,
            1,
        );
    }

    pub(in crate::scene::world) fn record_compiled_scene_property_writer_access(&self) {
        self.compiled_scene_property_access_diagnostics.record(
            &self
                .compiled_scene_property_access_diagnostics
                .compiled_writer_accesses,
            1,
        );
    }

    pub(in crate::scene::world) fn record_compiled_scene_property_stale_target(&self) {
        self.compiled_scene_property_access_diagnostics.record(
            &self
                .compiled_scene_property_access_diagnostics
                .stale_target_rejections,
            1,
        );
    }

    pub(in crate::scene::world) fn record_scene_property_entry_visit(&self) {
        self.compiled_scene_property_access_diagnostics.record(
            &self
                .compiled_scene_property_access_diagnostics
                .property_entry_visits,
            1,
        );
    }
}
