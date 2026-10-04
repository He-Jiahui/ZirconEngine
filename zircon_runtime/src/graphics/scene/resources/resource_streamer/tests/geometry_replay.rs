#[path = "../geometry_replay/tests/bounded_target_selection.rs"]
mod bounded_target_selection;

use std::collections::HashSet;

use super::*;
use crate::core::framework::render::{RenderMeshBounds, RenderWorldSnapshotHandle};
use crate::core::resource::{
    MeshMarker, ModelMarker, ResourceHandle, ResourceId, ResourceLocator, ResourceManager,
    ResourceMutationBatch, ResourceRecord,
};
use crate::graphics::scene::render_scene::{
    RenderSceneComponentProjectionError, RenderSceneComponentProjectionTransactionError,
    RenderSceneComponentProjector, RenderSceneGeometryResolveIssue, RenderSceneGeometryResolver,
    RenderScenePrimitiveLocalBounds, RenderSceneResolvedGeometry,
};
use crate::scene::components::{ActiveSelf, LocalTransform, MeshRenderer, Name, RenderLayerMask};
use crate::scene::ecs::Bundle;
use crate::scene::{SystemStage, World};

struct TestResolver {
    calls: usize,
    revision: u64,
    bounds_extent: f32,
    issue: Option<RenderSceneGeometryResolveIssue>,
    dependencies: Vec<UntypedResourceHandle>,
}

impl TestResolver {
    fn new(revision: u64, bounds_extent: f32) -> Self {
        Self {
            calls: 0,
            revision,
            bounds_extent,
            issue: None,
            dependencies: Vec::new(),
        }
    }
}

impl RenderSceneGeometryResolver for TestResolver {
    fn supplemental_geometry_dependencies(
        &self,
        _source: &crate::graphics::scene::render_scene::RenderSceneMeshSource,
    ) -> Vec<UntypedResourceHandle> {
        self.dependencies.clone()
    }

    fn resolve_geometry(
        &mut self,
        _entity: u64,
        source: &crate::graphics::scene::render_scene::RenderSceneMeshSource,
        _morph_weights: &[f32],
    ) -> Result<RenderSceneResolvedGeometry, RenderSceneGeometryResolveIssue> {
        self.calls += 1;
        if let Some(issue) = self.issue.take() {
            return Err(issue);
        }
        let bounds =
            RenderMeshBounds::from_min_max([-self.bounds_extent; 3], [self.bounds_extent; 3]);
        Ok(RenderSceneResolvedGeometry::new(
            RenderScenePrimitiveLocalBounds::new(
                bounds,
                source.lods().iter().map(|_| bounds).collect(),
            ),
            self.revision,
            self.revision,
            self.revision,
        ))
    }
}

fn bundle(name: &str, renderer: MeshRenderer) -> impl Bundle {
    (
        Name(name.to_string()),
        renderer,
        ActiveSelf::default(),
        RenderLayerMask::default(),
    )
}

fn projector_with_renderers(
    renderers: impl IntoIterator<Item = MeshRenderer>,
) -> (
    RenderSceneComponentProjector,
    std::sync::Arc<crate::core::framework::render::RenderComponentChangeArtifact>,
) {
    let mut world = World::empty();
    for (index, renderer) in renderers.into_iter().enumerate() {
        world
            .spawn(bundle(&format!("mesh-{index}"), renderer))
            .unwrap();
    }
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    let artifact = world.render_component_change_artifact().unwrap();
    let mut projector =
        RenderSceneComponentProjector::new(RenderWorldSnapshotHandle::new(artifact.world().raw()));
    projector
        .project(&artifact, &mut TestResolver::new(1, 1.0))
        .unwrap()
        .expect("initial projection");
    (projector, artifact)
}

fn event(label: &str, kind: ResourceKind, revision: u64) -> ResourceEvent {
    ResourceEvent {
        kind: ResourceEventKind::Updated,
        resource_kind: kind,
        id: ResourceId::from_stable_label(label),
        locator: None,
        previous_locator: None,
        revision,
    }
}

#[test]
fn render_scene_resource_geometry_replay_filters_unrelated_assets() {
    let mut state = RenderSceneGeometryReplayState::new(None);
    state.record_event(event("texture", ResourceKind::Texture, 2));
    state.record_event(event("mesh", ResourceKind::Mesh, 2));

    let batch = state.drain(8, |_| true);

    assert_eq!(batch.resources.len(), 1);
    assert_eq!(batch.resources[0].0.kind(), ResourceKind::Mesh);
}

#[test]
fn render_scene_resource_geometry_replay_zero_budget_does_not_consume_events() {
    let manager = ResourceManager::new();
    let receiver = manager.subscribe();
    let mut state = RenderSceneGeometryReplayState::new(Some(receiver));
    let record = ResourceRecord::new(
        ResourceId::from_stable_label("replay/zero-budget"),
        ResourceKind::Mesh,
        ResourceLocator::parse("res://meshes/zero-budget.mesh")
            .expect("fixture locator must be valid"),
    );
    manager
        .commit(ResourceMutationBatch::new().upsert_lazy(record))
        .expect("fixture publication must succeed");

    assert!(state.drain(0, |_| true).resources.is_empty());
    assert_eq!(state.receiver.as_ref().expect("receiver").len(), 1);
    assert_eq!(state.drain(1, |_| true).resources.len(), 1);
}

#[test]
fn render_scene_resource_geometry_replay_disconnect_requires_resync_and_reconnect() {
    let manager = ResourceManager::new();
    let mut state = RenderSceneGeometryReplayState::new(Some(manager.subscribe()));
    drop(manager);

    let batch = state.drain(1, |_| true);
    assert!(batch.resync);
    assert!(state.receiver_missing());

    state.commit(true, true);
    assert!(state.drain(1, |_| true).resync);

    let replacement_manager = ResourceManager::new();
    state.install_receiver(replacement_manager.subscribe());
    state.commit(true, true);
    assert!(!state.drain(1, |_| true).resync);
    assert!(!state.receiver_missing());
}

#[test]
fn render_scene_resource_geometry_replay_same_revision_is_noop_after_commit() {
    let mut state = RenderSceneGeometryReplayState::new(None);
    state.record_event(event("model", ResourceKind::Model, 7));
    state.commit(true, true);
    state.record_event(event("model", ResourceKind::Model, 7));

    assert!(state.drain(8, |_| true).resources.is_empty());
}

#[test]
fn render_scene_resource_geometry_replay_failure_keeps_pending_revision() {
    let mut state = RenderSceneGeometryReplayState::new(None);
    state.record_event(event("mesh", ResourceKind::Mesh, 3));

    assert_eq!(state.drain(8, |_| true).resources.len(), 1);
    assert_eq!(state.drain(8, |_| true).resources.len(), 1);
    state.commit(true, true);
    assert!(state.drain(8, |_| true).resources.is_empty());
}

#[test]
fn render_scene_resource_geometry_replay_keeps_latest_revision_until_final_chunk_commits() {
    let mut state = RenderSceneGeometryReplayState::new(None);
    let mesh = UntypedResourceHandle::new(
        ResourceId::from_stable_label("replay/repeated-update"),
        ResourceKind::Mesh,
    );
    state.record_event(event("replay/repeated-update", ResourceKind::Mesh, 1));
    assert_eq!(state.drain(8, |_| true).resources, vec![(mesh, 1)]);

    state.commit(true, false);
    state.record_event(event("replay/repeated-update", ResourceKind::Mesh, 2));
    state.record_event(event("replay/repeated-update", ResourceKind::Mesh, 1));
    assert_eq!(state.drain(8, |_| true).resources, vec![(mesh, 2)]);

    state.commit(true, true);
    assert!(state.drain(8, |_| true).resources.is_empty());
}

#[test]
fn render_scene_resource_geometry_replay_gap_requires_resync_until_complete() {
    let mut state = RenderSceneGeometryReplayState::new(None);
    state.record_event(event("mesh", ResourceKind::Mesh, 4));
    state.require_resync();

    assert!(state.drain(8, |_| true).resync);
    state.commit(false, true);
    assert!(state.drain(8, |_| true).resync);
    state.commit(true, true);
    let batch = state.drain(8, |_| true);
    assert!(!batch.resync);
    assert!(batch.resources.is_empty());
}

#[test]
fn render_scene_resource_geometry_replay_history_is_bounded_by_scene_dependencies() {
    let mut state = RenderSceneGeometryReplayState::new(None);
    for index in 0..4_096 {
        let mut added = event(&format!("unused/{index}"), ResourceKind::Mesh, 1);
        added.kind = ResourceEventKind::Added;
        state.record_event(added);
        let mut removed = event(&format!("unused/{index}"), ResourceKind::Mesh, 2);
        removed.kind = ResourceEventKind::Removed;
        state.record_event(removed);
    }
    state.retain_dependencies(|_| false);
    assert_eq!(state.tracked_revision_count(), 0);

    let retained = UntypedResourceHandle::new(
        ResourceId::from_stable_label("used/failing-mesh"),
        ResourceKind::Mesh,
    );
    let dependencies = HashSet::from([retained]);
    for revision in 1..=4_096 {
        state.record_event(event("used/failing-mesh", ResourceKind::Mesh, revision));
    }
    state.retain_dependencies(|resource| dependencies.contains(&resource));
    assert_eq!(state.tracked_revision_count(), 1);
    assert_eq!(state.drain(8, |_| true).resources, vec![(retained, 4_096)]);
}

#[test]
fn render_scene_resource_geometry_replay_updates_shared_model_without_component_change() {
    let renderer = MeshRenderer::default();
    let model = UntypedResourceHandle::new(renderer.model.id(), ResourceKind::Model);
    let (mut projector, unchanged_artifact) =
        projector_with_renderers([renderer.clone(), renderer]);
    let mut resolver = TestResolver::new(2, 2.0);

    assert!(projector
        .project(&unchanged_artifact, &mut resolver)
        .unwrap()
        .is_none());
    let commit = projector
        .replay_resource_geometry_with_staging(&[model], false, usize::MAX, &mut resolver, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap()
        .expect("shared model replay");
    let crate::graphics::scene::render_scene::RenderSceneComponentProjectionCommit::Applied {
        journal,
        ..
    } = commit
    else {
        panic!("resource replay must publish a journal")
    };

    assert_eq!(resolver.calls, 2);
    assert_eq!(journal.updates().len(), 2);
    assert_eq!(journal.stats().dirty_domain_counts().geometry_count(), 2);
    assert_eq!(journal.stats().dirty_domain_counts().bounds_count(), 2);
}

#[test]
fn render_scene_resource_geometry_replay_keeps_same_frame_declared_and_supplemental_dependency_events(
) {
    let model_id = ResourceId::from_stable_label("replay/same-frame-dependency");
    let mut renderer = MeshRenderer::default();
    renderer.model = ResourceHandle::<ModelMarker>::new(model_id);
    let model = UntypedResourceHandle::new(model_id, ResourceKind::Model);
    let mut world = World::empty();
    world
        .spawn(bundle("same-frame-model", renderer))
        .expect("same-frame model entity must spawn");
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    let artifact = world
        .render_component_change_artifact()
        .expect("same-frame geometry artifact");

    let declared = RenderSceneComponentProjector::artifact_geometry_resources(&artifact);
    assert!(declared.contains(&model));
    assert!(RenderSceneComponentProjector::artifact_may_reference_supplemental_geometry(&artifact));
    let child_mesh = UntypedResourceHandle::new(
        ResourceId::from_stable_label("replay/same-frame-child-mesh"),
        ResourceKind::Mesh,
    );
    assert!(!declared.contains(&child_mesh));

    let manager = ResourceManager::new();
    let mut dropped_state = RenderSceneGeometryReplayState::new(Some(manager.subscribe()));
    let mut retained_state = RenderSceneGeometryReplayState::new(Some(manager.subscribe()));
    let locator = ResourceLocator::parse("res://models/same-frame-dependency.model")
        .expect("same-frame model locator must be valid");
    manager
        .commit(
            ResourceMutationBatch::new()
                .upsert_lazy(ResourceRecord::new(model_id, ResourceKind::Model, locator))
                .upsert_lazy(ResourceRecord::new(
                    child_mesh.id(),
                    ResourceKind::Mesh,
                    ResourceLocator::parse("res://meshes/same-frame-child-mesh.mesh")
                        .expect("same-frame child mesh locator must be valid"),
                )),
        )
        .expect("same-frame model publication");

    assert!(dropped_state.drain(2, |_| false).resources.is_empty());
    let batch = retained_state.drain(2, |resource| {
        declared.contains(&resource)
            || (RenderSceneComponentProjector::artifact_may_reference_supplemental_geometry(
                &artifact,
            ) && matches!(resource.kind(), ResourceKind::Mesh | ResourceKind::Model))
    });
    assert_eq!(batch.resources.len(), 2);
    assert!(batch
        .resources
        .iter()
        .any(|(resource, revision)| { *resource == model && *revision > 0 }));
    assert!(batch
        .resources
        .iter()
        .any(|(resource, revision)| { *resource == child_mesh && *revision > 0 }));
    let mut projector =
        RenderSceneComponentProjector::new(RenderWorldSnapshotHandle::new(artifact.world().raw()));
    let mut resolver = TestResolver::new(1, 1.0);
    resolver.dependencies.push(child_mesh);
    projector
        .project(&artifact, &mut resolver)
        .expect("same-frame artifact projection")
        .expect("same-frame artifact must publish a journal");
    retained_state.retain_dependencies(|resource| projector.has_geometry_dependency(resource));
    assert_eq!(retained_state.tracked_revision_count(), 2);

    let mut direct_renderer = MeshRenderer::default();
    direct_renderer.mesh = Some(ResourceHandle::<MeshMarker>::new(
        ResourceId::from_stable_label("replay/same-frame-direct-mesh"),
    ));
    let mut direct_world = World::empty();
    direct_world
        .spawn(bundle("same-frame-direct-mesh", direct_renderer))
        .expect("same-frame direct mesh entity must spawn");
    direct_world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    let direct_artifact = direct_world
        .render_component_change_artifact()
        .expect("same-frame direct mesh artifact");
    assert!(
        !RenderSceneComponentProjector::artifact_may_reference_supplemental_geometry(
            &direct_artifact,
        )
    );
}

#[test]
fn render_scene_resource_geometry_replay_targets_mesh_and_ignores_unrelated_mesh() {
    let mesh = ResourceHandle::<MeshMarker>::new(ResourceId::from_stable_label("replay/mesh"));
    let unrelated = UntypedResourceHandle::new(
        ResourceId::from_stable_label("replay/unrelated-mesh"),
        ResourceKind::Mesh,
    );
    let mut renderer = MeshRenderer::default();
    renderer.mesh = Some(mesh);
    let (mut projector, _) = projector_with_renderers([renderer]);
    let mut resolver = TestResolver::new(2, 3.0);

    assert!(projector
        .replay_resource_geometry_with_staging(
            &[unrelated],
            false,
            usize::MAX,
            &mut resolver,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .is_none());
    assert_eq!(resolver.calls, 0);

    let resource = UntypedResourceHandle::new(mesh.id(), ResourceKind::Mesh);
    let commit = projector
        .replay_resource_geometry_with_staging(
            &[resource],
            false,
            usize::MAX,
            &mut resolver,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap();
    assert!(commit.is_some());
    assert_eq!(resolver.calls, 1);
}

#[test]
fn render_scene_resource_geometry_replay_indexes_model_internal_mesh_dependencies() {
    let renderer = MeshRenderer::default();
    let model = UntypedResourceHandle::new(renderer.model.id(), ResourceKind::Model);
    let mesh = UntypedResourceHandle::new(
        ResourceId::from_stable_label("replay/internal-mesh"),
        ResourceKind::Mesh,
    );
    let (mut projector, _) = projector_with_renderers([renderer]);
    let mut resolver = TestResolver::new(2, 2.0);
    resolver.dependencies.push(mesh);
    projector
        .replay_resource_geometry_with_staging(&[model], false, usize::MAX, &mut resolver, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap();
    assert!(projector.has_geometry_dependency(mesh));
    let selection = projector.prepare_geometry_replay(&[mesh], false, 256, None);
    assert!(selection.required_resources().contains(&mesh));
    let mut changed_mesh = TestResolver::new(3, 3.0);
    projector
        .replay_resource_geometry_with_staging(
            &[mesh],
            false,
            usize::MAX,
            &mut changed_mesh,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .expect("model must react to its internal mesh revision");
    assert_eq!(changed_mesh.calls, 1);
    assert!(!projector.has_geometry_dependency(mesh));
}

#[test]
fn render_scene_resource_geometry_replay_component_and_asset_resolve_once() {
    let mut world = World::empty();
    let renderer = MeshRenderer::default();
    let model = UntypedResourceHandle::new(renderer.model.id(), ResourceKind::Model);
    let entity = world.spawn(bundle("changed", renderer)).unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    let artifact = world.render_component_change_artifact().unwrap();
    let mut projector =
        RenderSceneComponentProjector::new(RenderWorldSnapshotHandle::new(artifact.world().raw()));
    projector
        .project(&artifact, &mut TestResolver::new(1, 1.0))
        .unwrap();
    world.get_mut::<MeshRenderer>(entity).unwrap().morph_weights = vec![0.5];
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    let mut resolver = TestResolver::new(2, 2.0);
    projector
        .project_with_resource_geometry_staging(
            &world.render_component_change_artifact().unwrap(),
            &[model],
            false,
            usize::MAX,
            &mut resolver,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap();
    assert_eq!(resolver.calls, 1);
}

#[test]
fn render_scene_resource_geometry_replay_resolve_failure_preserves_scene_state() {
    let renderer = MeshRenderer::default();
    let model = UntypedResourceHandle::new(renderer.model.id(), ResourceKind::Model);
    let (mut projector, _) = projector_with_renderers([renderer]);
    let before_generation = projector.read().generation();
    let before_bounds = projector.read().iter().next().unwrap().1.local_bounds();
    let mut resolver = TestResolver::new(2, 4.0);
    resolver.issue = Some(RenderSceneGeometryResolveIssue::Pending);

    let error = projector
        .replay_resource_geometry_with_staging(&[model], false, usize::MAX, &mut resolver, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .expect_err("pending replacement geometry must fail transactionally");

    assert!(matches!(
        error,
        RenderSceneComponentProjectionTransactionError::Projection(
            RenderSceneComponentProjectionError::GeometryResolution {
                issue: RenderSceneGeometryResolveIssue::Pending,
                ..
            }
        )
    ));
    assert_eq!(projector.read().generation(), before_generation);
    assert_eq!(
        projector.read().iter().next().unwrap().1.local_bounds(),
        before_bounds
    );
}

#[test]
fn render_scene_resource_geometry_replay_merges_with_continuous_component_changes_atomically() {
    let mesh = ResourceHandle::<MeshMarker>::new(ResourceId::from_stable_label(
        "replay/continuously-dirty-mesh",
    ));
    let mut stable_renderer = MeshRenderer::default();
    stable_renderer.mesh = Some(mesh);
    let mut world = World::empty();
    let stable = world.spawn(bundle("stable", stable_renderer)).unwrap();
    let changing = world
        .spawn(bundle("changing", MeshRenderer::default()))
        .unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    let mut projector = RenderSceneComponentProjector::new(RenderWorldSnapshotHandle::new(
        world
            .render_component_change_artifact()
            .unwrap()
            .world()
            .raw(),
    ));
    projector
        .project(
            &world.render_component_change_artifact().unwrap(),
            &mut TestResolver::new(1, 1.0),
        )
        .unwrap();

    let mut transform = world.get::<LocalTransform>(changing).unwrap().transform;
    transform.translation.x = 2.0;
    world.update_transform(changing, transform).unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    let changed_artifact = world.render_component_change_artifact().unwrap();
    let resource = UntypedResourceHandle::new(mesh.id(), ResourceKind::Mesh);
    let before_generation = projector.read().generation();
    let mut failing = TestResolver::new(2, 3.0);
    failing.issue = Some(RenderSceneGeometryResolveIssue::Pending);

    projector
        .project_with_resource_geometry_staging(
            &changed_artifact,
            &[resource],
            false,
            usize::MAX,
            &mut failing,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .expect_err("geometry failure must reject the component update in the same transaction");
    assert_eq!(projector.read().generation(), before_generation);

    let staging_error = projector
        .project_with_resource_geometry_staging(
            &changed_artifact,
            &[resource],
            false,
            usize::MAX,
            &mut TestResolver::new(2, 3.0),
            |_| Err::<(), _>("residency rejected"),
        )
        .expect_err("residency failure must preserve the merged scene delta");
    assert!(matches!(
        staging_error,
        RenderSceneComponentProjectionTransactionError::Staging("residency rejected")
    ));
    assert_eq!(projector.read().generation(), before_generation);

    let commit = projector
        .project_with_resource_geometry_staging(
            &changed_artifact,
            &[resource],
            false,
            usize::MAX,
            &mut TestResolver::new(2, 3.0),
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .expect("component and geometry retry");
    let crate::graphics::scene::render_scene::RenderSceneComponentProjectionCommit::Applied {
        journal,
        ..
    } = commit
    else {
        panic!("combined replay must publish a journal")
    };
    assert_eq!(journal.updates().len(), 2);
    assert_eq!(journal.stats().dirty_domain_counts().geometry_count(), 1);
    let stable_primitive = projector
        .read()
        .iter()
        .find(|(_, primitive)| primitive.descriptor().node_id == stable)
        .map(|(_, primitive)| primitive.clone())
        .expect("stable primitive");
    assert_eq!(stable_primitive.local_bounds().max, [3.0; 3]);
}

#[test]
fn render_scene_resource_geometry_replay_resync_is_bounded_and_continues() {
    let renderer = MeshRenderer::default();
    let (mut projector, _) =
        projector_with_renderers([renderer.clone(), renderer.clone(), renderer]);
    let mut resolver = TestResolver::new(2, 2.0);

    let first = projector
        .replay_resource_geometry_with_staging(&[], true, 2, &mut resolver, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap()
        .expect("first bounded resync batch");
    assert!(matches!(
        first,
        crate::graphics::scene::render_scene::RenderSceneComponentProjectionCommit::Applied { .. }
    ));
    assert_eq!(resolver.calls, 2);
    assert!(!projector.geometry_resync_complete());

    projector
        .replay_resource_geometry_with_staging(&[], true, 2, &mut resolver, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap()
        .expect("second bounded resync batch");
    assert_eq!(resolver.calls, 3);
    assert!(projector.geometry_resync_complete());
}

#[test]
fn render_scene_resource_geometry_replay_empty_and_removed_gap_commits() {
    let mut world = World::empty();
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    let artifact = world.render_component_change_artifact().unwrap();
    let mut projector =
        RenderSceneComponentProjector::new(RenderWorldSnapshotHandle::new(artifact.world().raw()));
    let mut resolver = TestResolver::new(1, 1.0);
    projector
        .project_with_resource_geometry_staging(&artifact, &[], true, 2, &mut resolver, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap();
    assert!(projector.geometry_resync_complete());
    assert_eq!(resolver.calls, 0);

    let entity = world
        .spawn(bundle("removed", MeshRenderer::default()))
        .unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    projector
        .project(
            &world.render_component_change_artifact().unwrap(),
            &mut resolver,
        )
        .unwrap();
    world.remove::<MeshRenderer>(entity).unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    let removed = world.render_component_change_artifact().unwrap();
    let selection = projector.prepare_geometry_replay(&[], true, 2, Some(&removed));
    assert!(selection.required_resources().is_empty());
    let before_calls = resolver.calls;
    projector
        .project_with_resource_geometry_staging(&removed, &[], true, 2, &mut resolver, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap();
    assert!(projector.geometry_resync_complete());
    assert_eq!(projector.read().iter().count(), 0);
    assert_eq!(resolver.calls, before_calls);
}

#[test]
fn render_scene_resource_geometry_replay_resync_selection_materializes_only_budget() {
    let (mut projector, _) = projector_with_renderers((0..129).map(|_| MeshRenderer::default()));
    let mut resolver = TestResolver::new(2, 2.0);
    let mut selected = 0;
    loop {
        projector.take_geometry_selection_visits();
        let selection = projector.prepare_geometry_replay(&[], true, 7, None);
        assert!(selection.selected_primitive_count() <= 7);
        assert_eq!(
            projector.take_geometry_selection_visits(),
            selection.selected_primitive_count()
        );
        selected += selection.selected_primitive_count();
        projector
            .replay_resource_geometry_with_staging(&[], true, 7, &mut resolver, |_| {
                Ok::<(), std::convert::Infallible>(())
            })
            .unwrap();
        if projector.geometry_resync_complete() {
            break;
        }
    }
    assert_eq!(selected, 129);
    assert_eq!(resolver.calls, 129);
}

#[test]
fn render_scene_resource_geometry_replay_continues_dependent_targets_after_budget() {
    let renderer = MeshRenderer::default();
    let model = UntypedResourceHandle::new(renderer.model.id(), ResourceKind::Model);
    let (mut projector, _) = projector_with_renderers((0..300).map(|_| renderer.clone()));
    let mut resolver = TestResolver::new(2, 2.0);

    let first = projector.prepare_geometry_replay(&[model], false, 256, None);
    assert_eq!(first.selected_primitive_count(), 256);
    assert!(first.truncated());
    assert!(first.target_keys().windows(2).all(|keys| keys[0] < keys[1]));
    let mut failing = TestResolver::new(2, 2.0);
    failing.issue = Some(RenderSceneGeometryResolveIssue::Pending);
    assert!(projector
        .replay_resource_geometry_with_staging(&[model], false, 256, &mut failing, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .is_err());
    let retry = projector.prepare_geometry_replay(&[model], false, 256, None);
    assert_eq!(retry.selected_primitive_count(), 256);
    assert!(retry.truncated());

    projector
        .replay_resource_geometry_with_staging(&[model], false, 256, &mut resolver, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap();
    projector.finish_geometry_replay_selection(&first);

    let second = projector.prepare_geometry_replay(&[model], false, 256, None);
    assert_eq!(second.selected_primitive_count(), 44);
    assert!(!second.truncated());
    projector
        .replay_resource_geometry_with_staging(&[model], false, 256, &mut resolver, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap();
    projector.finish_geometry_replay_selection(&second);

    assert_eq!(resolver.calls, 300);
}

#[test]
fn render_scene_resource_geometry_replay_restarts_continuation_for_new_revision() {
    let model_id = ResourceId::from_stable_label("replay/revision-aware");
    let model_handle = ResourceHandle::<ModelMarker>::new(model_id);
    let mut renderer = MeshRenderer::default();
    renderer.model = model_handle;
    let model = UntypedResourceHandle::new(model_id, ResourceKind::Model);
    let (mut projector, _) = projector_with_renderers((0..300).map(|_| renderer.clone()));

    let mut replay_state = RenderSceneGeometryReplayState::new(None);
    replay_state.record_event(event("replay/revision-aware", ResourceKind::Model, 1));
    let first_batch = replay_state.drain(1, |_| true);
    let first_resources = first_batch.resources;
    let first =
        projector.prepare_geometry_replay_with_revisions(&first_resources, false, 256, None);
    assert_eq!(first.selected_primitive_count(), 256);
    assert!(first.truncated());
    projector.finish_geometry_replay_selection(&first);
    replay_state.commit(true, !first.truncated());
    assert_eq!(replay_state.drain(0, |_| true).resources, vec![(model, 1)]);

    // A newer event arriving between chunks invalidates the old stable-key cursor. The
    // first 256 dependents must be replayed again against the newer prepared revision.
    replay_state.record_event(event("replay/revision-aware", ResourceKind::Model, 2));
    let second_batch = replay_state.drain(1, |_| true);
    let second_resources = second_batch.resources;
    assert_eq!(second_resources, vec![(model, 2)]);
    let second =
        projector.prepare_geometry_replay_with_revisions(&second_resources, false, 256, None);
    assert_eq!(second.selected_primitive_count(), 256);
    assert!(second.truncated());
    assert_eq!(second.target_keys(), first.target_keys());
    projector.finish_geometry_replay_selection(&second);
    replay_state.commit(true, !second.truncated());
    assert_eq!(replay_state.drain(0, |_| true).resources, vec![(model, 2)]);

    let final_chunk =
        projector.prepare_geometry_replay_with_revisions(&second_resources, false, 256, None);
    assert_eq!(final_chunk.selected_primitive_count(), 44);
    assert!(!final_chunk.truncated());
    projector.finish_geometry_replay_selection(&final_chunk);
    replay_state.commit(true, !final_chunk.truncated());
    assert!(replay_state.drain(0, |_| true).resources.is_empty());
}

#[test]
fn render_scene_resource_geometry_replay_component_commit_restarts_partial_selection() {
    let renderer = MeshRenderer::default();
    let model = UntypedResourceHandle::new(renderer.model.id(), ResourceKind::Model);
    let mut world = World::empty();
    let mut changed = None;
    for index in 0..300 {
        let entity = world
            .spawn(bundle(&format!("mesh-{index}"), renderer.clone()))
            .unwrap();
        if index == 0 {
            changed = Some(entity);
        }
    }
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    let initial = world.render_component_change_artifact().unwrap();
    let mut projector =
        RenderSceneComponentProjector::new(RenderWorldSnapshotHandle::new(initial.world().raw()));
    projector
        .project(&initial, &mut TestResolver::new(1, 1.0))
        .unwrap();

    let first = projector.prepare_geometry_replay(&[model], false, 256, None);
    assert!(first.truncated());
    projector.finish_geometry_replay_selection(&first);
    assert_eq!(
        projector
            .prepare_geometry_replay(&[model], false, 256, None)
            .selected_primitive_count(),
        44
    );

    world
        .get_mut::<MeshRenderer>(changed.expect("changed entity"))
        .unwrap()
        .morph_weights = vec![0.5];
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    let changed_artifact = world.render_component_change_artifact().unwrap();
    let mut resolver = TestResolver::new(2, 2.0);
    projector
        .project_with_resource_geometry_staging(
            &changed_artifact,
            &[model],
            false,
            256,
            &mut resolver,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap();
    assert_eq!(
        projector
            .prepare_geometry_replay(&[model], false, 256, None)
            .selected_primitive_count(),
        256
    );
}

#[test]
fn render_scene_resource_geometry_replay_stable_frame_without_invalidations_resolves_nothing() {
    let renderer = MeshRenderer::default();
    let (mut projector, artifact) = projector_with_renderers([renderer]);
    let mut resolver = TestResolver::new(2, 2.0);

    let commit = projector
        .project_with_resource_geometry_staging(&artifact, &[], false, 256, &mut resolver, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap();

    assert!(matches!(
        commit,
        crate::graphics::scene::render_scene::RenderSceneComponentProjectionCommit::Replayed
    ));
    assert_eq!(resolver.calls, 0);
}
