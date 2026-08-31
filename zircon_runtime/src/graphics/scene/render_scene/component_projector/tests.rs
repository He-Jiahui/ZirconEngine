use crate::core::framework::render::{
    RenderComponentFullReprojectionReason, RenderComponentProjectionMode, RenderExtractContext,
    RenderMeshBounds, RenderWorldSnapshotHandle, SceneViewportExtractRequest,
    render_mesh_stable_instance_key,
};
use crate::core::math::{Mat4, Quat, Transform, Vec3};
use crate::scene::components::{
    ActiveInHierarchy, LocalTransform, MeshRenderer, Mobility, Name, RenderLayerMask, WorldMatrix,
};
use crate::scene::ecs::{Bundle, RemovedComponentRetention};
use crate::scene::{SystemStage, World};

use super::*;

#[derive(Default)]
struct TestGeometryResolver {
    calls: usize,
    next_issue: Option<RenderSceneGeometryResolveIssue>,
}

impl RenderSceneGeometryResolver for TestGeometryResolver {
    fn resolve_geometry(
        &mut self,
        _entity: u64,
        source: &RenderSceneMeshSource,
        _morph_weights: &[f32],
    ) -> Result<RenderSceneResolvedGeometry, RenderSceneGeometryResolveIssue> {
        self.calls += 1;
        if let Some(issue) = self.next_issue.take() {
            return Err(issue);
        }
        Ok(RenderSceneResolvedGeometry::new(
            RenderScenePrimitiveLocalBounds::new(
                RenderMeshBounds::from_min_max([-1.0; 3], [1.0; 3]),
                source
                    .lods()
                    .iter()
                    .map(|_| RenderMeshBounds::from_min_max([-1.0; 3], [1.0; 3]))
                    .collect(),
            ),
            1,
            1,
            1,
        ))
    }
}

fn mesh_bundle(name: &str) -> impl Bundle {
    (
        Name(name.to_string()),
        LocalTransform::default(),
        MeshRenderer::default(),
        WorldMatrix::default(),
        ActiveInHierarchy::default(),
        RenderLayerMask::default(),
        Mobility::default(),
    )
}

fn publish(world: &mut World) {
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
}

#[test]
fn render_scene_component_projector_applies_full_artifact_once() {
    let mut world = World::empty();
    let entity = world.spawn(mesh_bundle("mesh")).unwrap();
    let world_handle = RenderWorldSnapshotHandle::new(7);
    let frame = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        world_handle,
        SceneViewportExtractRequest::default(),
    ));
    let mut projector = RenderSceneComponentProjector::new(world_handle);
    let mut resolver = TestGeometryResolver::default();

    let journal = projector
        .project_frame(&frame, &mut resolver)
        .expect("initial component projection")
        .expect("initial projection journal");

    assert_eq!(journal.additions().len(), 1);
    assert_eq!(resolver.calls, 1);
    assert_eq!(projector.read().len(), 1);
    assert!(
        projector
            .read()
            .handle_for_stable_key(render_mesh_stable_instance_key(entity, 0))
            .is_some()
    );
    assert!(
        projector
            .project_frame(&frame, &mut resolver)
            .expect("exact replay")
            .is_none()
    );
    assert_eq!(resolver.calls, 1);
}

#[test]
fn render_scene_component_projector_rejects_a_frame_from_another_world() {
    let mut world = World::empty();
    world.spawn(mesh_bundle("mesh")).unwrap();
    let expected = RenderWorldSnapshotHandle::new(70);
    let incoming = RenderWorldSnapshotHandle::new(71);
    let frame = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        incoming,
        SceneViewportExtractRequest::default(),
    ));
    let mut projector = RenderSceneComponentProjector::new(expected);
    let mut resolver = TestGeometryResolver::default();

    let error = projector
        .project_frame(&frame, &mut resolver)
        .expect_err("cross-world frame must be rejected");

    assert_eq!(
        error,
        RenderSceneComponentProjectionError::FrameWorldMismatch { expected, incoming }
    );
    assert_eq!(resolver.calls, 0);
    assert!(projector.read().is_empty());
}

#[test]
fn render_scene_component_projector_reuses_geometry_for_transform_only_patch() {
    let mut world = World::empty();
    let entity = world.spawn(mesh_bundle("mesh")).unwrap();
    publish(&mut world);
    let mut projector = RenderSceneComponentProjector::new(RenderWorldSnapshotHandle::new(8));
    let mut resolver = TestGeometryResolver::default();
    projector
        .project(
            &world.render_component_change_artifact().unwrap(),
            &mut resolver,
        )
        .unwrap();

    world
        .get_mut::<LocalTransform>(entity)
        .unwrap()
        .transform
        .translation
        .x = 5.0;
    publish(&mut world);
    let journal = projector
        .project(
            &world.render_component_change_artifact().unwrap(),
            &mut resolver,
        )
        .expect("transform projection")
        .expect("transform journal");

    assert_eq!(resolver.calls, 1);
    assert_eq!(journal.updates().len(), 1);
    assert_eq!(
        journal.updates()[0].dirty(),
        RenderScenePrimitiveDirtyFlags::TRANSFORM | RenderScenePrimitiveDirtyFlags::BOUNDS
    );
}

#[test]
fn render_scene_component_projector_retries_same_artifact_after_pending_geometry() {
    let mut world = World::empty();
    world.spawn(mesh_bundle("mesh")).unwrap();
    publish(&mut world);
    let artifact = world.render_component_change_artifact().unwrap();
    let mut projector = RenderSceneComponentProjector::new(RenderWorldSnapshotHandle::new(9));
    let mut resolver = TestGeometryResolver {
        next_issue: Some(RenderSceneGeometryResolveIssue::Pending),
        ..TestGeometryResolver::default()
    };

    let error = projector
        .project(&artifact, &mut resolver)
        .expect_err("pending geometry must not publish");
    assert!(matches!(
        error,
        RenderSceneComponentProjectionError::GeometryResolution {
            issue: RenderSceneGeometryResolveIssue::Pending,
            ..
        }
    ));
    assert!(projector.read().is_empty());

    let journal = projector
        .project(&artifact, &mut resolver)
        .expect("same artifact retry")
        .expect("retry journal");
    assert_eq!(journal.additions().len(), 1);
    assert_eq!(resolver.calls, 2);
}

#[test]
fn render_scene_component_projector_full_resync_removes_absent_primitive() {
    let mut world = World::empty();
    let removed = world.spawn(mesh_bundle("removed")).unwrap();
    let retained = world.spawn(mesh_bundle("retained")).unwrap();
    publish(&mut world);
    let mut projector = RenderSceneComponentProjector::new(RenderWorldSnapshotHandle::new(10));
    let mut resolver = TestGeometryResolver::default();
    projector
        .project(
            &world.render_component_change_artifact().unwrap(),
            &mut resolver,
        )
        .unwrap();
    world.configure_removed_component_retention::<MeshRenderer>(RemovedComponentRetention::new(
        0, 0, 0,
    ));

    world.remove::<MeshRenderer>(removed).unwrap();
    publish(&mut world);
    let journal = projector
        .project(
            &world.render_component_change_artifact().unwrap(),
            &mut resolver,
        )
        .expect("full recovery")
        .expect("recovery journal");

    assert_eq!(journal.removals().len(), 1);
    assert_eq!(projector.read().len(), 1);
    assert!(
        projector
            .read()
            .handle_for_stable_key(render_mesh_stable_instance_key(retained, 0))
            .is_some()
    );
    assert_eq!(resolver.calls, 3);
}

#[test]
fn render_scene_component_projector_keeps_batch_atomic_while_geometry_is_pending() {
    let mut world = World::empty();
    let geometry_entity = world.spawn(mesh_bundle("geometry")).unwrap();
    let transform_entity = world.spawn(mesh_bundle("transform")).unwrap();
    publish(&mut world);
    let mut projector = RenderSceneComponentProjector::new(RenderWorldSnapshotHandle::new(11));
    let mut resolver = TestGeometryResolver::default();
    projector
        .project(
            &world.render_component_change_artifact().unwrap(),
            &mut resolver,
        )
        .unwrap();

    world
        .get_mut::<MeshRenderer>(geometry_entity)
        .unwrap()
        .morph_weights
        .push(0.5);
    world
        .get_mut::<LocalTransform>(transform_entity)
        .unwrap()
        .transform
        .translation
        .x = 7.0;
    publish(&mut world);
    let artifact = world.render_component_change_artifact().unwrap();
    resolver.next_issue = Some(RenderSceneGeometryResolveIssue::Pending);

    let error = projector
        .project(&artifact, &mut resolver)
        .expect_err("one pending primitive must reject the complete delta");
    assert!(matches!(
        error,
        RenderSceneComponentProjectionError::GeometryResolution {
            entity,
            issue: RenderSceneGeometryResolveIssue::Pending,
        } if entity == geometry_entity
    ));
    let geometry_key = render_mesh_stable_instance_key(geometry_entity, 0);
    let transform_key = render_mesh_stable_instance_key(transform_entity, 0);
    let read = projector.read();
    let geometry = read
        .get(read.handle_for_stable_key(geometry_key).unwrap())
        .unwrap();
    let transform = read
        .get(read.handle_for_stable_key(transform_key).unwrap())
        .unwrap();
    assert!(geometry.descriptor().morph_weights.is_empty());
    assert_eq!(
        transform.descriptor().world_from_local.to_cols_array_2d()[3][0],
        0.0
    );
    drop(read);

    let journal = projector
        .project(&artifact, &mut resolver)
        .expect("same generation must remain retryable")
        .expect("retry must publish one journal");
    assert_eq!(journal.updates().len(), 2);
    assert_eq!(resolver.calls, 4);
}

#[test]
fn render_scene_component_projector_rejects_incremental_generation_gap_before_resolution() {
    let mut world = World::empty();
    let entity = world.spawn(mesh_bundle("mesh")).unwrap();
    publish(&mut world);
    let mut projector = RenderSceneComponentProjector::new(RenderWorldSnapshotHandle::new(12));
    let mut resolver = TestGeometryResolver::default();
    let initial_artifact = world.render_component_change_artifact().unwrap();
    let applied_generation = initial_artifact.journal_generation();
    projector.project(&initial_artifact, &mut resolver).unwrap();

    world
        .get_mut::<LocalTransform>(entity)
        .unwrap()
        .transform
        .translation
        .x = 1.0;
    publish(&mut world);
    world
        .get_mut::<LocalTransform>(entity)
        .unwrap()
        .transform
        .translation
        .x = 2.0;
    publish(&mut world);
    let skipped_artifact = world.render_component_change_artifact().unwrap();
    let incoming_generation = skipped_artifact.journal_generation();

    let error = projector
        .project(&skipped_artifact, &mut resolver)
        .expect_err("incremental generation gap must request a full source replay");
    assert!(matches!(
        error,
        RenderSceneComponentProjectionError::JournalDiscontinuity {
            applied_generation: error_applied,
            incoming_generation: error_incoming,
        } if error_applied == applied_generation && error_incoming == incoming_generation
    ));
    assert_eq!(resolver.calls, 1);
}

#[test]
fn render_scene_component_projector_recovers_pending_generation_gap_from_source_full_reprojection()
{
    let mut world = World::empty();
    let entity = world.spawn(mesh_bundle("mesh")).unwrap();
    publish(&mut world);
    let mut projector = RenderSceneComponentProjector::new(RenderWorldSnapshotHandle::new(14));
    let mut resolver = TestGeometryResolver::default();
    projector
        .project(
            &world.render_component_change_artifact().unwrap(),
            &mut resolver,
        )
        .unwrap();

    world
        .get_mut::<MeshRenderer>(entity)
        .unwrap()
        .morph_weights
        .push(0.75);
    publish(&mut world);
    resolver.next_issue = Some(RenderSceneGeometryResolveIssue::Pending);
    projector
        .project(
            &world.render_component_change_artifact().unwrap(),
            &mut resolver,
        )
        .expect_err("pending geometry leaves the artifact generation unapplied");

    world
        .get_mut::<LocalTransform>(entity)
        .unwrap()
        .transform
        .translation
        .x = 9.0;
    publish(&mut world);
    let gap = projector
        .project(
            &world.render_component_change_artifact().unwrap(),
            &mut resolver,
        )
        .expect_err("latest sparse artifact cannot cover the pending generation");
    assert!(matches!(
        gap,
        RenderSceneComponentProjectionError::JournalDiscontinuity { .. }
    ));

    world.request_full_render_component_projection();
    publish(&mut world);
    let recovery = world.render_component_change_artifact().unwrap();
    assert_eq!(
        recovery.mode(),
        RenderComponentProjectionMode::Full(
            RenderComponentFullReprojectionReason::JournalRequested
        )
    );
    projector.project(&recovery, &mut resolver).unwrap();

    let read = projector.read();
    let primitive = read
        .get(
            read.handle_for_stable_key(render_mesh_stable_instance_key(entity, 0))
                .unwrap(),
        )
        .unwrap();
    assert_eq!(primitive.descriptor().morph_weights.as_ref(), &[0.75]);
    assert_eq!(primitive.world_bounds().center, [9.0, 0.0, 0.0]);
    assert_eq!(resolver.calls, 3);
}

#[test]
fn render_scene_component_projector_preserves_hierarchical_shear_world_matrix() {
    let mut world = World::empty();
    let parent = world
        .spawn((
            Name("parent".to_string()),
            LocalTransform {
                transform: Transform {
                    scale: Vec3::new(2.0, 1.0, 1.0),
                    ..Transform::default()
                },
            },
            WorldMatrix::default(),
            ActiveInHierarchy::default(),
            RenderLayerMask::default(),
            Mobility::default(),
        ))
        .unwrap();
    let child = world.spawn(mesh_bundle("child")).unwrap();
    world
        .get_mut::<LocalTransform>(child)
        .unwrap()
        .transform
        .rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_4);
    world.set_parent_checked(child, Some(parent)).unwrap();
    publish(&mut world);
    let expected = world.world_matrix(child).unwrap();
    let (scale, rotation, translation) = expected.to_scale_rotation_translation();
    let lossy_trs = Mat4::from_scale_rotation_translation(scale, rotation, translation);
    assert_ne!(expected.to_cols_array(), lossy_trs.to_cols_array());

    let mut projector = RenderSceneComponentProjector::new(RenderWorldSnapshotHandle::new(13));
    let mut resolver = TestGeometryResolver::default();
    projector
        .project(
            &world.render_component_change_artifact().unwrap(),
            &mut resolver,
        )
        .unwrap();
    let read = projector.read();
    let primitive = read
        .get(
            read.handle_for_stable_key(render_mesh_stable_instance_key(child, 0))
                .unwrap(),
        )
        .unwrap();

    assert_eq!(
        primitive.descriptor().world_from_local.to_cols_array(),
        expected.to_cols_array()
    );
}
