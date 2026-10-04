use std::sync::Arc;

use zircon_runtime::core::framework::render::{
    RenderMeshBounds, RenderMeshSnapshot, RenderMeshStaticState, RendererCommon,
};
use zircon_runtime::core::framework::scene::Mobility;
use zircon_runtime::core::math::{Transform, Vec3, Vec4};
use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};

use super::RuntimePrepareMeshProjectionCache;
use crate::hybrid_gi::scene_representation::HybridGiGlobalSdfClipmapBounds;

#[test]
fn cache_requires_authoritative_static_revisions() {
    assert!(RenderMeshStaticState::new(true, 7, 11).has_authoritative_revisions());
    assert!(!RenderMeshStaticState::new(false, 7, 11).has_authoritative_revisions());
    assert!(!RenderMeshStaticState::new(true, 0, 11).has_authoritative_revisions());
    assert!(!RenderMeshStaticState::new(true, 7, 0).has_authoritative_revisions());
}

#[test]
fn cache_reuses_only_the_same_page_aligned_clipmap_snapshot() {
    let initial = [HybridGiGlobalSdfClipmapBounds::new(0, Vec3::ZERO, 16.0)];
    let shifted = [HybridGiGlobalSdfClipmapBounds::new(0, Vec3::X * 4.0, 16.0)];
    let mut cache = RuntimePrepareMeshProjectionCache::default();

    assert!(!cache.can_reuse(&[], &initial));
    cache.capture(&[], &initial, Arc::from([]));
    assert!(cache.can_reuse(&[], &initial));
    assert!(!cache.can_reuse(&[], &shifted));
}

#[test]
fn cache_never_reuses_a_dynamic_mesh_snapshot() {
    let clipmaps = [HybridGiGlobalSdfClipmapBounds::new(0, Vec3::ZERO, 16.0)];
    let meshes = [RenderMeshSnapshot {
        node_id: 1,
        stable_instance_key: 1,
        transform_revision: 1,
        transform: Transform::from_translation(Vec3::ZERO),
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
            "res://models/cache-test.model.toml",
        )),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
            "res://materials/cache-test.zmaterial",
        )),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Dynamic,
        static_state: RenderMeshStaticState::new(false, 7, 11),
        common: RendererCommon::default(),
    }];
    let mut cache = RuntimePrepareMeshProjectionCache::default();

    cache.capture(&meshes, &clipmaps, Arc::from([]));

    assert!(!cache.can_reuse(&meshes, &clipmaps));
}

#[test]
fn cache_retries_static_meshes_until_every_geometry_projection_is_ready() {
    let clipmaps = [HybridGiGlobalSdfClipmapBounds::new(0, Vec3::ZERO, 16.0)];
    let meshes = [RenderMeshSnapshot {
        node_id: 1,
        stable_instance_key: 1,
        transform_revision: 1,
        transform: Transform::from_translation(Vec3::ZERO),
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
            "res://models/cache-test.model.toml",
        )),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
            "res://materials/cache-test.zmaterial",
        )),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Static,
        static_state: RenderMeshStaticState::new(true, 7, 11),
        common: RendererCommon::default(),
    }];
    let mut cache = RuntimePrepareMeshProjectionCache::default();

    cache.capture(&meshes, &clipmaps, Arc::from([]));
    assert!(!cache.can_reuse(&meshes, &clipmaps));

    let bounds = RenderMeshBounds::from_min_max([-1.0; 3], [1.0; 3]);
    cache.capture(&meshes, &clipmaps, Arc::from([(1, bounds)]));
    assert!(cache.can_reuse(&meshes, &clipmaps));
}
