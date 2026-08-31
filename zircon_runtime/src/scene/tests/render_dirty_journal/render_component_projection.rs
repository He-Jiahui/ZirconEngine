use std::sync::Arc;

use super::*;
use crate::core::framework::render::{
    RenderComponentChangeKind, RenderComponentChangeMask, RenderComponentFullReprojectionReason,
    RenderComponentProjectionMode, RenderComponentValue, RenderExtractContext,
    RenderWorldSnapshotHandle, SceneViewportExtractRequest,
};
use crate::scene::components::{
    ActiveInHierarchy, MeshRenderer, Mobility, Name, RenderLayerMask, WorldMatrix,
};
use crate::scene::ecs::RemovedComponentRetention;

fn mesh_bundle(name: &str) -> impl crate::scene::ecs::Bundle {
    (
        Name(name.to_string()),
        MeshRenderer::default(),
        WorldMatrix::default(),
        ActiveInHierarchy::default(),
        RenderLayerMask::default(),
        Mobility::default(),
    )
}

#[test]
fn render_component_projection_full_snapshot_is_world_owned_and_stably_replayed() {
    let mut world = World::empty();
    let mesh = world.spawn(mesh_bundle("mesh")).unwrap();
    publish_render_dirty_journal(&mut world);

    let first = world.render_component_change_artifact().unwrap();
    assert!(matches!(
        first.mode(),
        RenderComponentProjectionMode::Full(_)
    ));
    assert_eq!(first.upserts().len(), 1);
    assert_eq!(first.upserts()[0].entity(), mesh);
    assert_eq!(first.upserts()[0].kind(), RenderComponentChangeKind::Added);
    assert_eq!(first.upserts()[0].mask(), RenderComponentChangeMask::ALL);
    assert_eq!(first.stats().full_scan_entities(), 1);

    publish_render_dirty_journal(&mut world);
    let stable = world.render_component_change_artifact().unwrap();
    assert!(Arc::ptr_eq(&first, &stable));
}

#[test]
fn render_component_projection_is_shared_through_the_frame_geometry_contract() {
    let mut world = World::empty();
    world.spawn(mesh_bundle("mesh")).unwrap();
    let context = RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(701),
        SceneViewportExtractRequest::default(),
    );

    let first_frame = world.build_prepared_render_frame_extract(&context);
    let first = first_frame
        .geometry
        .scene_changes
        .as_ref()
        .expect("render extract must carry the world-owned scene changes");
    let published = world.render_component_change_artifact().unwrap();
    assert!(Arc::ptr_eq(first, &published));

    let second_frame = world.build_prepared_render_frame_extract(&context);
    let second = second_frame
        .geometry
        .scene_changes
        .as_ref()
        .expect("stable render extract must retain the scene-change artifact");
    assert!(Arc::ptr_eq(first, second));
    assert_eq!(second.stats().full_scan_entities(), 1);
}

#[test]
fn render_component_projection_full_reprojection_requests_coalesce_at_source() {
    let mut world = World::empty();
    let mesh = world.spawn(mesh_bundle("mesh")).unwrap();
    publish_render_dirty_journal(&mut world);
    let initial_generation = world
        .render_component_change_artifact()
        .unwrap()
        .journal_generation();

    world.request_full_render_component_projection();
    world.request_full_render_component_projection();
    publish_render_dirty_journal(&mut world);

    let artifact = world.render_component_change_artifact().unwrap();
    assert_eq!(artifact.journal_generation(), initial_generation + 1);
    assert_eq!(
        artifact.mode(),
        RenderComponentProjectionMode::Full(
            RenderComponentFullReprojectionReason::JournalRequested
        )
    );
    assert_eq!(artifact.upserts().len(), 1);
    assert_eq!(artifact.upserts()[0].entity(), mesh);
    assert!(artifact.removals().is_empty());
    assert_eq!(artifact.stats().full_scan_entities(), 1);
}

#[test]
fn render_component_projection_classifies_only_relevant_candidate_ticks() {
    let mut world = World::empty();
    let changed = world.spawn(mesh_bundle("changed")).unwrap();
    let unrelated = world.spawn(mesh_bundle("unrelated")).unwrap();
    publish_render_dirty_journal(&mut world);

    world.get_mut::<MeshRenderer>(changed).unwrap().render_queue = 9;
    world.remove::<RenderLayerMask>(changed).unwrap();
    world.get_mut::<Name>(unrelated).unwrap().0 = "renamed".to_string();
    publish_render_dirty_journal(&mut world);

    let artifact = world.render_component_change_artifact().unwrap();
    assert_eq!(artifact.mode(), RenderComponentProjectionMode::Incremental);
    assert_eq!(artifact.upserts().len(), 1);
    let upsert = &artifact.upserts()[0];
    assert_eq!(upsert.entity(), changed);
    assert_eq!(upsert.kind(), RenderComponentChangeKind::Updated);
    assert_eq!(
        upsert.mask(),
        RenderComponentChangeMask::MESH_RENDERER | RenderComponentChangeMask::RENDER_LAYER
    );
    assert_eq!(upsert.render_layer_mask(), &RenderComponentValue::Removed);
    assert_eq!(artifact.stats().candidate_entities(), 2);
    assert_eq!(artifact.stats().component_tick_probes(), 10);
}

#[test]
fn render_component_projection_does_not_clone_mesh_payload_for_transform_only_change() {
    let mut world = World::empty();
    let mesh = world.spawn(mesh_bundle("mesh")).unwrap();
    world.insert(mesh, LocalTransform::default()).unwrap();
    publish_render_dirty_journal(&mut world);

    world
        .get_mut::<LocalTransform>(mesh)
        .unwrap()
        .transform
        .translation
        .x = 4.0;
    publish_render_dirty_journal(&mut world);

    let artifact = world.render_component_change_artifact().unwrap();
    assert_eq!(artifact.upserts().len(), 1);
    assert_eq!(
        artifact.upserts()[0].mask(),
        RenderComponentChangeMask::WORLD_TRANSFORM
    );
    assert_eq!(
        artifact.upserts()[0].mesh_renderer(),
        &RenderComponentValue::Unchanged
    );
    assert_eq!(artifact.stats().mesh_renderer_payload_clones(), 0);
}

#[test]
fn render_component_projection_consumes_mesh_removal_once() {
    let mut world = World::empty();
    let mesh = world.spawn(mesh_bundle("mesh")).unwrap();
    publish_render_dirty_journal(&mut world);

    world.remove::<MeshRenderer>(mesh).unwrap();
    publish_render_dirty_journal(&mut world);
    let removed = world.render_component_change_artifact().unwrap();
    assert_eq!(removed.removals(), &[mesh]);
    assert!(removed.upserts().is_empty());

    publish_render_dirty_journal(&mut world);
    let replay = world.render_component_change_artifact().unwrap();
    assert!(Arc::ptr_eq(&removed, &replay));
    assert_eq!(replay.stats().removal_events_read(), 1);
}

#[test]
fn render_component_projection_collapses_remove_then_readd_to_added_upsert() {
    let mut world = World::empty();
    let mesh = world.spawn(mesh_bundle("mesh")).unwrap();
    publish_render_dirty_journal(&mut world);

    world.remove::<MeshRenderer>(mesh).unwrap();
    world.insert(mesh, MeshRenderer::default()).unwrap();
    publish_render_dirty_journal(&mut world);

    let artifact = world.render_component_change_artifact().unwrap();
    assert!(artifact.removals().is_empty());
    assert_eq!(artifact.upserts().len(), 1);
    assert_eq!(
        artifact.upserts()[0].kind(),
        RenderComponentChangeKind::Added
    );
    assert_eq!(artifact.upserts()[0].mask(), RenderComponentChangeMask::ALL);
}

#[test]
fn render_component_projection_resyncs_after_removal_history_loss() {
    let mut world = World::empty();
    let removed = world.spawn(mesh_bundle("removed")).unwrap();
    let retained = world.spawn(mesh_bundle("retained")).unwrap();
    publish_render_dirty_journal(&mut world);
    world.configure_removed_component_retention::<MeshRenderer>(RemovedComponentRetention::new(
        0, 0, 0,
    ));

    world.remove::<MeshRenderer>(removed).unwrap();
    publish_render_dirty_journal(&mut world);

    let artifact = world.render_component_change_artifact().unwrap();
    assert!(matches!(
        artifact.mode(),
        RenderComponentProjectionMode::Full(_)
    ));
    assert!(artifact.removals().is_empty());
    assert_eq!(artifact.upserts().len(), 1);
    assert_eq!(artifact.upserts()[0].entity(), retained);
    assert_eq!(artifact.stats().removal_events_dropped(), 1);
}
