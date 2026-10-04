use std::sync::{Arc, Mutex};

use crate::core::framework::render::RenderFrameTiming;
use crate::core::framework::scene::WorldHandle;
use crate::core::math::Vec3;
use crate::scene::{LevelMetadata, World};

use super::*;

fn test_level() -> LevelSystem {
    LevelSystem::new(
        WorldHandle::new(71),
        Arc::new(Mutex::new(World::empty())),
        LevelMetadata::default(),
    )
}

#[test]
fn stable_cache_reuse_shares_scene_and_keeps_submission_overlay_local() {
    let level = test_level();
    let viewport_size = UVec2::new(1280, 720);
    let mut cache = RuntimeFrameExtractCache::default();

    let initial = cache.current_extract(&level, viewport_size);
    assert_eq!(initial.status, RuntimeFrameExtractCacheStatus::Rebuilt);

    let mut reused = cache.current_extract(&level, viewport_size);
    assert_eq!(reused.status, RuntimeFrameExtractCacheStatus::Reused);
    assert!(initial.extract.shares_scene_with(&reused.extract));

    reused.extract.view.camera.transform.translation = Vec3::new(7.0, 8.0, 9.0);
    reused
        .extract
        .set_timing(RenderFrameTiming::new(41, 1.0 / 60.0));

    let cached_again = cache.current_extract(&level, viewport_size);
    assert_eq!(cached_again.status, RuntimeFrameExtractCacheStatus::Reused);
    assert!(initial.extract.shares_scene_with(&cached_again.extract));
    assert_eq!(
        cached_again.extract.view.camera.transform.translation,
        initial.extract.view.camera.transform.translation
    );
    assert_eq!(cached_again.extract.timing, RenderFrameTiming::default());
}

#[test]
fn cache_miss_extracts_the_live_world_identity_and_rebuilds_after_world_replacement() {
    let level = LevelSystem::new(
        WorldHandle::new(0),
        Arc::new(Mutex::new(World::empty())),
        LevelMetadata::default(),
    );
    let viewport_size = UVec2::new(640, 480);
    let mut cache = RuntimeFrameExtractCache::default();

    let first = cache.current_extract(&level, viewport_size);
    let first_artifact = first
        .extract
        .geometry
        .scene_changes
        .as_ref()
        .expect("level extraction publishes the source-world artifact");
    assert_eq!(first.extract.world.raw(), first_artifact.world().raw());
    assert_ne!(first.extract.world.raw(), level.world_handle().get());

    let first_world = first.extract.world.raw();
    level.replace_world_and_reset_runtime_state(World::empty());
    let second = cache.current_extract(&level, viewport_size);
    let second_artifact = second
        .extract
        .geometry
        .scene_changes
        .as_ref()
        .expect("replacement level extraction publishes its source-world artifact");

    assert_eq!(second.status, RuntimeFrameExtractCacheStatus::Rebuilt);
    assert_ne!(second.extract.world.raw(), first_world);
    assert_eq!(second.extract.world.raw(), second_artifact.world().raw());
}

#[test]
fn every_cache_key_component_independently_requires_rebuild() {
    let baseline = RuntimeFrameExtractCacheKey {
        source_world: RenderWorldSnapshotHandle::new(10),
        change_tick: ChangeTick::new(11),
        lifecycle_visibility_revision: 12,
        active_camera: 13,
        viewport_size: UVec2::new(1280, 720),
    };

    assert_eq!(
        cache_status_for_key(Some(baseline), baseline),
        RuntimeFrameExtractCacheStatus::Reused
    );
    assert_eq!(
        cache_status_for_key(None, baseline),
        RuntimeFrameExtractCacheStatus::Rebuilt
    );

    for changed in [
        RuntimeFrameExtractCacheKey {
            source_world: RenderWorldSnapshotHandle::new(11),
            ..baseline
        },
        RuntimeFrameExtractCacheKey {
            change_tick: baseline.change_tick.next(),
            ..baseline
        },
        RuntimeFrameExtractCacheKey {
            lifecycle_visibility_revision: baseline.lifecycle_visibility_revision + 1,
            ..baseline
        },
        RuntimeFrameExtractCacheKey {
            active_camera: baseline.active_camera + 1,
            ..baseline
        },
        RuntimeFrameExtractCacheKey {
            viewport_size: UVec2::new(1920, 1080),
            ..baseline
        },
    ] {
        assert_eq!(
            cache_status_for_key(Some(baseline), changed),
            RuntimeFrameExtractCacheStatus::Rebuilt
        );
    }
}
