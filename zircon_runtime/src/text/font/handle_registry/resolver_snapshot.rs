use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::text::font::shared::{FontCollectionService, FontCollectionSnapshot};

use super::{
    BackendFontHandlePair, FontHandleRegistrySnapshot, TextFontHandlePair, duration_to_nanos,
    project_resolved_pairs, unique_current_text_pairs,
};

#[derive(Clone)]
pub(crate) struct FontHandleResolverSnapshot {
    font_collection: Arc<FontCollectionService>,
    registry: Arc<FontHandleRegistrySnapshot>,
}

impl std::fmt::Debug for FontHandleResolverSnapshot {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FontHandleResolverSnapshot")
            .field("collection", &self.registry.collection)
            .field("generation", &self.registry.generation)
            .field("face_count", &self.registry.faces.len())
            .field("instance_count", &self.registry.instances.len())
            .finish()
    }
}

impl FontHandleResolverSnapshot {
    pub(crate) fn matches_font_collection(&self, font_collection: &FontCollectionSnapshot) -> bool {
        self.registry.collection == font_collection.collection_id()
            && self.registry.generation == font_collection.generation()
    }
}

pub(crate) fn font_handle_resolver_snapshot(
    font_collection: &FontCollectionSnapshot,
) -> FontHandleResolverSnapshot {
    let registry_service = font_collection.service().handle_registry();
    registry_service
        .metrics
        .resolution_snapshot_acquire_count
        .fetch_add(1, Ordering::Relaxed);
    let (registry, wait, hold) = registry_service.current_snapshot();
    registry_service
        .metrics
        .resolution_snapshot_wait_nanos
        .fetch_add(duration_to_nanos(wait), Ordering::Relaxed);
    registry_service
        .metrics
        .resolution_snapshot_hold_nanos
        .fetch_add(duration_to_nanos(hold), Ordering::Relaxed);
    FontHandleResolverSnapshot {
        font_collection: font_collection.service_handle(),
        registry,
    }
}

/// Resolves through an immutable registry publication retained by an in-flight artifact.
///
/// Unlike the current-generation path, this deliberately does not probe the collection's latest
/// generation. The paired font database snapshot keeps the exact old generation alive until the
/// consumer releases both leases.
pub(crate) fn resolve_font_handle_batch_from_snapshot(
    resolver: &FontHandleResolverSnapshot,
    pairs: &[TextFontHandlePair],
) -> Vec<BackendFontHandlePair> {
    if pairs.is_empty() {
        return Vec::new();
    }
    let metrics = &resolver.font_collection.handle_registry().metrics;
    metrics
        .resolution_batch_count
        .fetch_add(1, Ordering::Relaxed);
    let collection = resolver.registry.collection;
    let generation = resolver.registry.generation;
    let unique = unique_current_text_pairs(pairs, collection, generation);
    metrics
        .resolution_unique_pair_count
        .fetch_add(unique.len() as u64, Ordering::Relaxed);
    let resolved_by_pair = unique
        .into_iter()
        .map(|(face, instance)| {
            (
                (face, instance),
                (
                    face.and_then(|handle| resolver.registry.resolve_face(handle)),
                    instance.and_then(|handle| resolver.registry.resolve_instance(handle)),
                ),
            )
        })
        .collect::<HashMap<_, _>>();
    let (result, rejected_count) =
        project_resolved_pairs(pairs, collection, generation, &resolved_by_pair);
    metrics
        .resolution_rejected_pair_count
        .fetch_add(rejected_count as u64, Ordering::Relaxed);
    result
}
