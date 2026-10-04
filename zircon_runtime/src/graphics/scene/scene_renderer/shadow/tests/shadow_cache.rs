use std::collections::HashMap;

use super::{
    shadow_light_params_hash, static_shadow_caster_revision,
    static_shadow_caster_revision_from_meshes,
    static_shadow_caster_revision_from_meshes_with_resource_revisions, ShadowCache,
    ShadowCacheDecision, ShadowCacheInput, ShadowCacheInvalidationReason,
    ShadowStaticCasterRevisionInput,
};
use crate::core::framework::render::{
    render_mesh_stable_instance_key, GpuLightType, RenderMeshSnapshot, RenderMeshStaticState,
    RendererCommon,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Transform, Vec4};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use crate::graphics::scene::scene_renderer::shadow::atlas::ShadowSlotKey;
use crate::graphics::scene::scene_renderer::shadow::slot::GpuShadowSlot;

fn input(
    light_params_hash: u64,
    static_caster_revision: u64,
    atlas_slot_generation: u64,
) -> ShadowCacheInput {
    ShadowCacheInput::new(
        ShadowSlotKey::new(GpuLightType::Point, 42, 3),
        light_params_hash,
        static_caster_revision,
        atlas_slot_generation,
    )
}

#[test]
fn render_shadow_cache_reuses_static_depth_only_for_an_exact_three_factor_match() {
    let mut cache = ShadowCache::default();
    let cached = input(11, 22, 33);

    assert_eq!(
        cache.evaluate(cached),
        ShadowCacheDecision::RedrawStaticDepth(ShadowCacheInvalidationReason::Missing)
    );
    cache.commit_static_depth(cached);

    assert_eq!(
        cache.evaluate(cached),
        ShadowCacheDecision::ReuseStaticDepth
    );
}

#[test]
fn render_shadow_cache_invalidates_for_each_static_depth_dependency() {
    let mut cache = ShadowCache::default();
    cache.commit_static_depth(input(11, 22, 33));

    assert_eq!(
        cache.evaluate(input(12, 22, 33)),
        ShadowCacheDecision::RedrawStaticDepth(
            ShadowCacheInvalidationReason::LightParametersChanged
        )
    );
    assert_eq!(
        cache.evaluate(input(11, 23, 33)),
        ShadowCacheDecision::RedrawStaticDepth(ShadowCacheInvalidationReason::StaticCastersChanged)
    );
    assert_eq!(
        cache.evaluate(input(11, 22, 34)),
        ShadowCacheDecision::RedrawStaticDepth(ShadowCacheInvalidationReason::AtlasSlotReallocated)
    );
}

#[test]
fn render_shadow_cache_discards_unallocated_slots() {
    let mut cache = ShadowCache::default();
    cache.commit_static_depth(input(11, 22, 33));
    cache.commit_static_depth(ShadowCacheInput::new(
        ShadowSlotKey::new(GpuLightType::Point, 77, 0),
        44,
        55,
        66,
    ));

    cache.retain_slots(|slot| slot.light_id == 42);

    assert_eq!(cache.entry_count(), 1);
    assert_eq!(
        cache.evaluate(ShadowCacheInput::new(
            ShadowSlotKey::new(GpuLightType::Point, 77, 0),
            44,
            55,
            66,
        )),
        ShadowCacheDecision::RedrawStaticDepth(ShadowCacheInvalidationReason::Missing)
    );
}

#[test]
fn render_shadow_cache_static_caster_revision_is_order_independent_and_content_sensitive() {
    let first = ShadowStaticCasterRevisionInput::new(10, 1, RenderMeshStaticState::new(true, 2, 3));
    let second =
        ShadowStaticCasterRevisionInput::new(20, 2, RenderMeshStaticState::new(true, 5, 7));

    let ordered = static_shadow_caster_revision([first, second])
        .expect("authoritative static casters are cacheable");
    let reordered = static_shadow_caster_revision([second, first])
        .expect("authoritative static casters are cacheable");
    let changed = static_shadow_caster_revision([
        first,
        ShadowStaticCasterRevisionInput::new(20, 2, RenderMeshStaticState::new(true, 5, 8)),
    ])
    .expect("authoritative static casters are cacheable");

    assert_eq!(ordered, reordered);
    assert_ne!(ordered, changed);
}

#[test]
fn render_shadow_cache_static_caster_revision_fails_closed_for_dynamic_or_unversioned_input() {
    assert_eq!(
        static_shadow_caster_revision([ShadowStaticCasterRevisionInput::new(
            10,
            1,
            RenderMeshStaticState::new(false, 2, 3),
        )]),
        None
    );
    assert_eq!(
        static_shadow_caster_revision([ShadowStaticCasterRevisionInput::new(
            10,
            1,
            RenderMeshStaticState::new(true, 2, 0),
        )]),
        None
    );
}

#[test]
fn render_shadow_cache_static_mesh_revision_ignores_dynamic_overlay_casters() {
    let static_mesh = test_mesh(1, Mobility::Static, RenderMeshStaticState::new(true, 2, 3));
    let dynamic_mesh = test_mesh(2, Mobility::Dynamic, RenderMeshStaticState::default());

    assert_eq!(
        static_shadow_caster_revision_from_meshes(&[static_mesh.clone(), dynamic_mesh]),
        static_shadow_caster_revision_from_meshes(&[static_mesh])
    );
    assert_eq!(
        static_shadow_caster_revision_from_meshes(&[test_mesh(
            3,
            Mobility::Static,
            RenderMeshStaticState::new(true, 0, 3),
        )]),
        None
    );
}

#[test]
fn render_shadow_cache_static_mesh_revision_tracks_ready_resource_revisions() {
    let static_mesh = test_mesh(
        1,
        Mobility::Static,
        RenderMeshStaticState::from_transform_static(true),
    );
    let mut revisions =
        HashMap::from([(static_mesh.model.id(), 4), (static_mesh.material.id(), 8)]);

    let initial = static_shadow_caster_revision_from_meshes_with_resource_revisions(
        &[static_mesh.clone()],
        |resource| revisions.get(&resource).copied(),
    )
    .expect("ready static resources are cacheable");
    revisions.insert(static_mesh.material.id(), 9);
    let changed = static_shadow_caster_revision_from_meshes_with_resource_revisions(
        &[static_mesh],
        |resource| revisions.get(&resource).copied(),
    )
    .expect("ready static resources are cacheable");

    assert_ne!(initial, changed);
}

#[test]
fn render_shadow_cache_static_mesh_revision_tracks_instance_transform_changes() {
    let mut static_mesh = test_mesh(
        1,
        Mobility::Static,
        RenderMeshStaticState::from_transform_static(true),
    );
    let revisions = HashMap::from([(static_mesh.model.id(), 4), (static_mesh.material.id(), 8)]);
    let initial = static_shadow_caster_revision_from_meshes_with_resource_revisions(
        &[static_mesh.clone()],
        |resource| revisions.get(&resource).copied(),
    )
    .expect("ready static resources are cacheable");
    static_mesh.transform_revision = 17;
    let moved = static_shadow_caster_revision_from_meshes_with_resource_revisions(
        &[static_mesh],
        |resource| revisions.get(&resource).copied(),
    )
    .expect("ready static resources are cacheable");

    assert_ne!(initial, moved);
}

#[test]
fn render_shadow_cache_static_mesh_revision_fails_closed_for_missing_resource_revision() {
    let static_mesh = test_mesh(
        1,
        Mobility::Static,
        RenderMeshStaticState::from_transform_static(true),
    );

    assert_eq!(
        static_shadow_caster_revision_from_meshes_with_resource_revisions(&[static_mesh], |_| None,),
        None
    );
}

#[test]
fn render_shadow_cache_light_params_hash_tracks_all_shader_visible_slot_values() {
    let base = GpuShadowSlot {
        view_proj: [[1.0, 0.0, 0.0, 0.0]; 4],
        atlas_scale_bias: [0.25, 0.25, 0.0, 0.0],
        params: [0.001, 0.01, 1.0 / 1024.0, 4.0],
    };
    let mut changed_view = base;
    changed_view.view_proj[2][1] = 0.5;
    let mut changed_atlas = base;
    changed_atlas.atlas_scale_bias[2] = 0.125;
    let mut changed_params = base;
    changed_params.params[1] = 0.02;

    assert_ne!(
        shadow_light_params_hash(&base),
        shadow_light_params_hash(&changed_view)
    );
    assert_ne!(
        shadow_light_params_hash(&base),
        shadow_light_params_hash(&changed_atlas)
    );
    assert_ne!(
        shadow_light_params_hash(&base),
        shadow_light_params_hash(&changed_params)
    );
}

fn test_mesh(
    entity: u64,
    mobility: Mobility,
    static_state: RenderMeshStaticState,
) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id: entity,
        stable_instance_key: render_mesh_stable_instance_key(entity, 0),
        transform_revision: 0,
        transform: Transform::default(),
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
            "shadow-cache-test-model",
        )),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
            "shadow-cache-test-material",
        )),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility,
        static_state,
        common: RendererCommon {
            is_static: mobility == Mobility::Static,
            ..RendererCommon::default()
        },
    }
}
