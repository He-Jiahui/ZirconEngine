use crate::core::framework::render::RenderWorldSnapshotHandle;
use crate::core::math::Vec4;
use crate::graphics::scene::render_scene::{
    RenderScene, RenderSceneDelta, RenderSceneGeneration, RenderScenePrimitiveDirtyFlags,
};

use super::super::{
    GpuSceneJournalConsumer, GpuSceneJournalReprojectionError,
    GpuSceneJournalReprojectionPreflightError, GpuSceneJournalResidentWriteKind,
};
use super::{TestStagingError, stable_key, test_primitive, test_primitive_with, test_world};

#[test]
fn render_gpu_scene_journal_reprojection_is_full_and_slot_ordered_without_generation_reset() {
    let world = test_world();
    let mut scene = RenderScene::new(world);
    let mut consumer = GpuSceneJournalConsumer::new(world);
    let initial = scene
        .apply_delta(RenderSceneDelta::new(
            vec![test_primitive(1), test_primitive(2), test_primitive(3)],
            Vec::new(),
        ))
        .expect("initial scene delta");
    let middle_handle = initial.additions()[1].handle();
    let last_handle = initial.additions()[2].handle();
    consumer
        .apply_with_staging(&initial, |_| Ok::<_, TestStagingError>(()))
        .expect("initial consumer apply");

    let removal = scene
        .apply_delta(RenderSceneDelta::new(Vec::new(), vec![stable_key(1)]))
        .expect("remove first persistent slot");
    consumer
        .apply_with_staging(&removal, |_| Ok::<_, TestStagingError>(()))
        .expect("removal consumer apply");
    let applied_generation = consumer.applied_generation();
    let read = scene.read();

    let staging_error = consumer
        .reproject_with_staging(&read, |_| Err::<(), _>(TestStagingError))
        .expect_err("failed device reprojection remains external");
    assert_eq!(
        staging_error,
        GpuSceneJournalReprojectionError::Staging(TestStagingError)
    );
    assert_eq!(consumer.applied_generation(), applied_generation);
    assert_eq!(consumer.slot_high_water(), 3);
    assert_eq!(consumer.resident_count(), 2);

    let output = consumer
        .reproject_with_staging(&read, |plan| {
            assert_eq!(plan.world(), world);
            assert_eq!(plan.generation(), applied_generation);
            assert_eq!(plan.slot_high_water(), 3);
            assert_eq!(plan.resident_count(), 2);
            assert_eq!(plan.full_resident_write_count(), 2);
            assert_eq!(plan.instance_transform_write_count(), 2);
            assert_eq!(plan.local_bounds_write_count(), 2);
            assert_eq!(plan.direct_slot_validation_count(), 2);
            assert_eq!(plan.stable_key_lookup_count(), 0);
            assert_eq!(plan.resident_writes().len(), 2);
            assert_eq!(plan.resident_writes()[0].handle(), middle_handle);
            assert_eq!(plan.resident_writes()[1].handle(), last_handle);
            for write in plan.resident_writes() {
                assert_eq!(write.kind(), GpuSceneJournalResidentWriteKind::Full);
                assert_eq!(write.dirty(), RenderScenePrimitiveDirtyFlags::ALL);
            }
            Ok::<_, TestStagingError>("device generation 2 upload")
        })
        .expect("current scene can fully reproject");

    assert_eq!(output, "device generation 2 upload");
    assert_eq!(consumer.applied_generation(), applied_generation);
    assert_eq!(consumer.slot_high_water(), 3);
    assert_eq!(consumer.resident_count(), 2);
}

#[test]
fn render_gpu_scene_journal_reprojection_rejects_world_or_generation_drift_before_staging() {
    let world = test_world();
    let mut scene = RenderScene::new(world);
    let mut consumer = GpuSceneJournalConsumer::new(world);
    let initial = scene
        .apply_delta(RenderSceneDelta::new(vec![test_primitive(8)], Vec::new()))
        .expect("initial scene delta");
    consumer
        .apply_with_staging(&initial, |_| Ok::<_, TestStagingError>(()))
        .expect("initial consumer apply");

    let foreign_world = RenderWorldSnapshotHandle::new(92);
    let mut foreign_scene = RenderScene::new(foreign_world);
    foreign_scene
        .apply_delta(RenderSceneDelta::new(vec![test_primitive(9)], Vec::new()))
        .expect("foreign scene delta");
    let mut staging_called = false;
    let foreign_error = consumer
        .reproject_with_staging(&foreign_scene.read(), |_| {
            staging_called = true;
            Ok::<_, TestStagingError>(())
        })
        .expect_err("foreign world cannot reproject this residency");
    assert_eq!(
        foreign_error,
        GpuSceneJournalReprojectionError::Preflight(
            GpuSceneJournalReprojectionPreflightError::WorldChanged {
                expected_world: world,
                scene_world: foreign_world,
            }
        )
    );

    scene
        .apply_delta(RenderSceneDelta::new(
            vec![test_primitive_with(8, |descriptor| {
                descriptor.tint = Vec4::new(0.25, 0.5, 0.75, 1.0);
            })],
            Vec::new(),
        ))
        .expect("unconsumed scene update");
    let generation_error = consumer
        .reproject_with_staging(&scene.read(), |_| {
            staging_called = true;
            Ok::<_, TestStagingError>(())
        })
        .expect_err("unconsumed generation cannot reproject");
    assert_eq!(
        generation_error,
        GpuSceneJournalReprojectionError::Preflight(
            GpuSceneJournalReprojectionPreflightError::GenerationChanged {
                applied_generation: RenderSceneGeneration::new(1),
                scene_generation: RenderSceneGeneration::new(2),
            }
        )
    );
    assert!(!staging_called);
    assert_eq!(consumer.applied_generation(), RenderSceneGeneration::new(1));
}
