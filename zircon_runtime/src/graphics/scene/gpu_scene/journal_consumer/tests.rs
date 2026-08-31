use std::sync::Arc;

use crate::core::framework::render::{
    MaterialPropertyOverrideBlock, RenderMaterialAlphaMode, RenderMeshBounds,
    RenderWorldSnapshotHandle, RendererCommon, render_mesh_stable_instance_key,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Mat4, Vec3, Vec4};
use crate::core::resource::{MaterialMarker, MeshMarker, ModelMarker, ResourceHandle, ResourceId};
use crate::graphics::scene::render_scene::{
    RenderScene, RenderSceneDelta, RenderSceneGeneration, RenderSceneMeshBinding,
    RenderSceneMeshSource, RenderSceneMeshSourceLevel, RenderScenePrimitive,
    RenderScenePrimitiveDescriptor, RenderScenePrimitiveDirtyFlags,
    RenderScenePrimitiveLocalBounds, RenderScenePrimitiveRevisions,
};

use super::{
    GpuSceneJournalConsumer, GpuSceneJournalConsumerError, GpuSceneJournalResidentWriteKind,
    GpuSceneJournalTransactionCommit, GpuSceneJournalTransactionError,
};

mod reprojection;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TestStagingError;

#[test]
fn render_gpu_scene_journal_consumer_commits_direct_slots_and_same_journal_reuse() {
    let world = test_world();
    let mut scene = RenderScene::new(world);
    let mut consumer = GpuSceneJournalConsumer::new(world);
    let first = scene
        .apply_delta(RenderSceneDelta::new(
            vec![test_primitive(1), test_primitive(2)],
            Vec::new(),
        ))
        .expect("initial scene delta");
    let first_handle = first.additions()[0].handle();
    let removed_handle = first.additions()[1].handle();

    let first_plan = consumer.preflight(&first).expect("initial consumer plan");
    assert_eq!(first_plan.projected_slot_high_water(), 2);
    assert_eq!(first_plan.projected_resident_count(), 2);
    assert_eq!(first_plan.direct_slot_validation_count(), 0);
    assert_eq!(first_plan.stable_key_lookup_count(), 0);
    assert_eq!(first_plan.slot_mutations().len(), 2);
    assert_eq!(first_plan.slot_mutations()[0].slot(), 0);
    assert_eq!(first_plan.slot_mutations()[1].slot(), 1);
    assert_eq!(first_plan.resident_writes().len(), 2);
    assert_eq!(first_plan.full_resident_write_count(), 2);
    assert_eq!(first_plan.dirty_resident_write_count(), 0);
    assert_eq!(first_plan.instance_transform_write_count(), 2);
    assert_eq!(first_plan.local_bounds_write_count(), 2);
    assert_eq!(first_plan.retirement_count(), 0);
    assert!(first_plan.retirements().is_empty());
    for (addition, write) in first.additions().iter().zip(first_plan.resident_writes()) {
        assert_eq!(write.handle(), addition.handle());
        assert_eq!(write.kind(), GpuSceneJournalResidentWriteKind::Full);
        assert_eq!(write.dirty(), RenderScenePrimitiveDirtyFlags::ALL);
        assert_eq!(
            write.primitive().stable_instance_key(),
            addition.primitive().stable_instance_key()
        );
    }
    consumer
        .commit_preflighted(first_plan)
        .expect("commit initial plan");
    assert_eq!(consumer.resident_count(), 2);
    assert_eq!(
        consumer.resident_stable_key(first_handle),
        Some(stable_key(1))
    );
    assert_eq!(
        consumer.resident_stable_key(removed_handle),
        Some(stable_key(2))
    );

    let changed = test_primitive_with(1, |descriptor| {
        descriptor.tint = Vec4::new(0.25, 0.5, 0.75, 1.0);
    });
    let second = scene
        .apply_delta(RenderSceneDelta::new(
            vec![changed, test_primitive(3)],
            vec![stable_key(2)],
        ))
        .expect("mixed scene delta");
    let replacement_handle = second.additions()[0].handle();
    assert_eq!(replacement_handle.slot(), removed_handle.slot());
    assert_ne!(
        replacement_handle.slot_generation(),
        removed_handle.slot_generation()
    );

    let second_plan = consumer.preflight(&second).expect("mixed consumer plan");
    assert_eq!(second_plan.journal().updates().len(), 1);
    assert_eq!(second_plan.journal().removals().len(), 1);
    assert_eq!(second_plan.journal().additions().len(), 1);
    assert_eq!(second_plan.projected_resident_count(), 2);
    assert_eq!(second_plan.direct_slot_validation_count(), 3);
    assert_eq!(second_plan.stable_key_lookup_count(), 0);
    assert_eq!(second_plan.slot_mutations().len(), 1);
    assert_eq!(
        second_plan.slot_mutations()[0].slot(),
        removed_handle.slot()
    );
    assert_eq!(
        second_plan.slot_mutations()[0].slot_generation(),
        replacement_handle.slot_generation()
    );
    assert_eq!(
        second_plan.slot_mutations()[0].stable_instance_key(),
        Some(stable_key(3))
    );
    assert_eq!(second_plan.resident_writes().len(), 2);
    assert_eq!(second_plan.full_resident_write_count(), 1);
    assert_eq!(second_plan.dirty_resident_write_count(), 1);
    assert_eq!(second_plan.instance_transform_write_count(), 1);
    assert_eq!(second_plan.local_bounds_write_count(), 1);
    assert_eq!(second_plan.retirement_count(), 1);
    let updated_write = &second_plan.resident_writes()[0];
    assert_eq!(updated_write.handle(), first_handle);
    assert_eq!(
        updated_write.kind(),
        GpuSceneJournalResidentWriteKind::Dirty
    );
    assert_eq!(
        updated_write.dirty(),
        RenderScenePrimitiveDirtyFlags::MATERIAL
    );
    assert_eq!(
        updated_write.primitive().stable_instance_key(),
        stable_key(1)
    );
    let replacement_write = &second_plan.resident_writes()[1];
    assert_eq!(replacement_write.handle(), replacement_handle);
    assert_eq!(
        replacement_write.kind(),
        GpuSceneJournalResidentWriteKind::Full
    );
    assert_eq!(
        replacement_write.dirty(),
        RenderScenePrimitiveDirtyFlags::ALL
    );
    assert_eq!(
        replacement_write.primitive().stable_instance_key(),
        stable_key(3)
    );
    assert_eq!(second_plan.retirements().len(), 1);
    let retirement = &second_plan.retirements()[0];
    assert_eq!(retirement.handle(), removed_handle);
    assert_eq!(retirement.stable_instance_key(), stable_key(2));
    assert_eq!(retirement.primitive().stable_instance_key(), stable_key(2));
    consumer
        .commit_preflighted(second_plan)
        .expect("commit mixed plan");

    assert_eq!(consumer.resident_stable_key(removed_handle), None);
    assert_eq!(
        consumer.resident_stable_key(replacement_handle),
        Some(stable_key(3))
    );
    assert_eq!(
        consumer.resident_stable_key(first_handle),
        Some(stable_key(1))
    );
    assert_eq!(consumer.applied_generation(), RenderSceneGeneration::new(2));
    assert_eq!(consumer.resident_count(), 2);
}

#[test]
fn render_gpu_scene_journal_consumer_replay_is_a_zero_mutation_plan() {
    let world = test_world();
    let mut scene = RenderScene::new(world);
    let mut consumer = GpuSceneJournalConsumer::new(world);
    let journal = scene
        .apply_delta(RenderSceneDelta::new(vec![test_primitive(4)], Vec::new()))
        .expect("initial scene delta");

    let plan = consumer.preflight(&journal).expect("initial plan");
    consumer.commit_preflighted(plan).expect("initial commit");
    let replay = consumer.preflight(&journal).expect("replay plan");

    assert!(!replay.requires_apply());
    assert!(replay.slot_mutations().is_empty());
    assert!(replay.resident_writes().is_empty());
    assert!(replay.retirements().is_empty());
    assert_eq!(replay.full_resident_write_count(), 0);
    assert_eq!(replay.dirty_resident_write_count(), 0);
    assert_eq!(replay.instance_transform_write_count(), 0);
    assert_eq!(replay.local_bounds_write_count(), 0);
    assert_eq!(replay.retirement_count(), 0);
    assert_eq!(replay.projected_resident_count(), 1);
    assert_eq!(replay.direct_slot_validation_count(), 0);
    assert_eq!(replay.stable_key_lookup_count(), 0);
    consumer.commit_preflighted(replay).expect("replay commit");
    assert_eq!(consumer.applied_generation(), RenderSceneGeneration::new(1));
    assert_eq!(consumer.slot_high_water(), 1);
    assert_eq!(consumer.resident_count(), 1);
}

#[test]
fn render_gpu_scene_journal_transaction_commits_only_after_successful_staging() {
    let world = test_world();
    let mut scene = RenderScene::new(world);
    let mut consumer = GpuSceneJournalConsumer::new(world);
    let journal = scene
        .apply_delta(RenderSceneDelta::new(vec![test_primitive(12)], Vec::new()))
        .expect("initial scene delta");
    let expected_handle = journal.additions()[0].handle();

    let error = consumer
        .apply_with_staging(&journal, |plan| {
            assert!(plan.requires_apply());
            assert_eq!(plan.full_resident_write_count(), 1);
            Err::<&'static str, _>(TestStagingError)
        })
        .expect_err("failed staging cannot advance residency");

    assert_eq!(
        error,
        GpuSceneJournalTransactionError::Staging(TestStagingError)
    );
    assert_eq!(
        consumer.applied_generation(),
        RenderSceneGeneration::INITIAL
    );
    assert_eq!(consumer.slot_high_water(), 0);
    assert_eq!(consumer.resident_count(), 0);
    assert_eq!(consumer.resident_stable_key(expected_handle), None);

    let mut staged_apply_count = 0;
    let committed = consumer
        .apply_with_staging(&journal, |plan| {
            staged_apply_count += 1;
            assert_eq!(plan.resident_writes()[0].handle(), expected_handle);
            Ok::<_, TestStagingError>("initial upload")
        })
        .expect("successful staging commits residency");
    assert_eq!(
        committed,
        GpuSceneJournalTransactionCommit::Applied("initial upload")
    );
    assert_eq!(staged_apply_count, 1);
    assert_eq!(consumer.applied_generation(), RenderSceneGeneration::new(1));
    assert_eq!(consumer.resident_count(), 1);
    assert_eq!(
        consumer.resident_stable_key(expected_handle),
        Some(stable_key(12))
    );

    let replay = consumer
        .apply_with_staging(&journal, |_| {
            staged_apply_count += 1;
            Ok::<_, TestStagingError>("unexpected replay upload")
        })
        .expect("exact replay remains a no-op");
    assert_eq!(replay, GpuSceneJournalTransactionCommit::Replayed);
    assert_eq!(staged_apply_count, 1);
    assert_eq!(consumer.applied_generation(), RenderSceneGeneration::new(1));
}

#[test]
fn render_gpu_scene_journal_work_keeps_transform_bounds_dirty_and_removal_retirement_exact() {
    let world = test_world();
    let mut scene = RenderScene::new(world);
    let mut consumer = GpuSceneJournalConsumer::new(world);
    let initial = scene
        .apply_delta(RenderSceneDelta::new(
            vec![test_primitive(10), test_primitive(11)],
            Vec::new(),
        ))
        .expect("initial scene delta");
    let removed_handle = initial.additions()[0].handle();
    let updated_handle = initial.additions()[1].handle();
    let initial_plan = consumer.preflight(&initial).expect("initial work plan");
    consumer
        .commit_preflighted(initial_plan)
        .expect("initial work commit");

    let expected_world_from_local = Mat4::from_translation(Vec3::new(4.0, 0.0, 0.0));
    let updated = test_primitive_with(11, |descriptor| {
        descriptor.world_from_local = expected_world_from_local;
    });
    let journal = scene
        .apply_delta(RenderSceneDelta::new(vec![updated], vec![stable_key(10)]))
        .expect("transform and removal delta");
    let plan = consumer.preflight(&journal).expect("transform work plan");

    assert_eq!(plan.resident_writes().len(), 1);
    assert_eq!(plan.instance_transform_write_count(), 1);
    assert_eq!(plan.local_bounds_write_count(), 0);
    assert_eq!(plan.resident_writes()[0].handle(), updated_handle);
    assert_eq!(
        plan.resident_writes()[0].dirty(),
        RenderScenePrimitiveDirtyFlags::TRANSFORM | RenderScenePrimitiveDirtyFlags::BOUNDS
    );
    assert!(plan.resident_writes()[0].requires_instance_transform_write());
    assert!(!plan.resident_writes()[0].requires_local_bounds_write());
    assert_eq!(
        plan.resident_writes()[0].kind(),
        GpuSceneJournalResidentWriteKind::Dirty
    );
    assert_eq!(
        plan.resident_writes()[0]
            .primitive()
            .descriptor()
            .world_from_local
            .to_cols_array(),
        expected_world_from_local.to_cols_array()
    );
    assert_eq!(
        plan.resident_writes()[0].primitive().world_bounds().center,
        [4.0, 0.0, 0.0]
    );
    assert_eq!(plan.retirements().len(), 1);
    assert_eq!(plan.retirements()[0].handle(), removed_handle);
    assert_eq!(plan.retirements()[0].stable_instance_key(), stable_key(10));
    assert_eq!(plan.stable_key_lookup_count(), 0);
}

#[test]
fn render_gpu_scene_journal_consumer_rejects_valid_generation_with_wrong_resident_key() {
    let world = test_world();
    let mut first_scene = RenderScene::new(world);
    let mut second_scene = RenderScene::new(world);
    let mut consumer = GpuSceneJournalConsumer::new(world);
    let first = first_scene
        .apply_delta(RenderSceneDelta::new(vec![test_primitive(5)], Vec::new()))
        .expect("first scene initial delta");
    let second_initial = second_scene
        .apply_delta(RenderSceneDelta::new(vec![test_primitive(6)], Vec::new()))
        .expect("second scene initial delta");
    let first_plan = consumer.preflight(&first).expect("first consumer plan");
    consumer
        .commit_preflighted(first_plan)
        .expect("first consumer commit");
    let second_update = second_scene
        .apply_delta(RenderSceneDelta::new(
            vec![test_primitive_with(6, |descriptor| {
                descriptor.tint = Vec4::new(0.5, 0.25, 0.75, 1.0);
            })],
            Vec::new(),
        ))
        .expect("second scene update");

    let mut staging_called = false;
    let error = consumer
        .apply_with_staging(&second_update, |_| {
            staging_called = true;
            Ok::<(), TestStagingError>(())
        })
        .expect_err("same generation cannot alias a different scene slot owner");

    assert_eq!(
        error,
        GpuSceneJournalTransactionError::Preflight(
            GpuSceneJournalConsumerError::StableKeyMismatch {
                handle: second_initial.additions()[0].handle(),
                resident_stable_key: stable_key(5),
                journal_stable_key: stable_key(6),
            }
        )
    );
    assert!(!staging_called);
    assert_eq!(consumer.applied_generation(), RenderSceneGeneration::new(1));
    assert_eq!(consumer.slot_high_water(), 1);
}

#[test]
fn render_gpu_scene_journal_consumer_rejects_stale_plan_without_mutating_slots() {
    let world = test_world();
    let mut scene = RenderScene::new(world);
    let mut consumer = GpuSceneJournalConsumer::new(world);
    let journal = scene
        .apply_delta(RenderSceneDelta::new(vec![test_primitive(7)], Vec::new()))
        .expect("initial scene delta");
    let first = consumer.preflight(&journal).expect("first plan");
    let stale = consumer.preflight(&journal).expect("parallel plan");

    consumer.commit_preflighted(first).expect("first commit");
    let error = consumer
        .commit_preflighted(stale)
        .expect_err("stale plan cannot commit after cursor advancement");

    assert!(matches!(error, GpuSceneJournalConsumerError::Cursor(_)));
    assert_eq!(consumer.slot_high_water(), 1);
    assert_eq!(consumer.applied_generation(), RenderSceneGeneration::new(1));
}

fn test_primitive(entity: u64) -> RenderScenePrimitive {
    test_primitive_with(entity, |_| {})
}

fn test_primitive_with(
    entity: u64,
    mutate: impl FnOnce(&mut RenderScenePrimitiveDescriptor),
) -> RenderScenePrimitive {
    let mut descriptor = RenderScenePrimitiveDescriptor {
        node_id: entity,
        stable_instance_key: stable_key(entity),
        world_from_local: Mat4::IDENTITY,
        mesh_source: RenderSceneMeshSource::new(test_mesh_source_level(), Vec::new()),
        morph_weights: Arc::from([]),
        skeletal_pose: None,
        tint: Vec4::ONE,
        material_property_overrides: MaterialPropertyOverrideBlock::default(),
        material_alpha_mode: RenderMaterialAlphaMode::Opaque,
        render_queue: 2_000,
        material_queue: 2_000,
        order_in_layer: 0,
        depth_bias: 0.0,
        mobility: Mobility::Static,
        transform_static: true,
        common: RendererCommon {
            is_static: true,
            ..RendererCommon::default()
        },
    };
    mutate(&mut descriptor);
    RenderScenePrimitive::new(
        descriptor,
        RenderScenePrimitiveLocalBounds::base_only(RenderMeshBounds::from_min_max(
            [-1.0; 3], [1.0; 3],
        )),
        RenderScenePrimitiveRevisions::new(1, 1, 1, 1, 1),
    )
    .expect("valid consumer-test primitive")
}

fn test_mesh_source_level() -> RenderSceneMeshSourceLevel {
    let mesh = ResourceHandle::<MeshMarker>::new(ResourceId::from_stable_label(
        "tests/gpu-scene-journal/mesh",
    ));
    let material = ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
        "tests/gpu-scene-journal/material",
    ));
    RenderSceneMeshSourceLevel::new(
        ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
            "tests/gpu-scene-journal/model",
        )),
        Some(mesh),
        material,
        vec![RenderSceneMeshBinding { mesh, material }],
    )
}

fn stable_key(entity: u64) -> u64 {
    render_mesh_stable_instance_key(entity, 0)
}

const fn test_world() -> RenderWorldSnapshotHandle {
    RenderWorldSnapshotHandle::new(91)
}
