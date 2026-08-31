use crate::core::framework::render::RenderFrameExtract;
use crate::core::math::UVec2;
use crate::scene::ecs::ChangeTick;
use crate::scene::{EntityId, LevelSystem};

use super::extract_stats::RuntimeFrameExtractDiagnosticsSummary;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RuntimeFrameExtractCacheStatus {
    Rebuilt,
    Reused,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RuntimeFrameExtractCacheKey {
    change_tick: ChangeTick,
    lifecycle_visibility_revision: u64,
    active_camera: EntityId,
    viewport_size: UVec2,
}

impl RuntimeFrameExtractCacheKey {
    fn from_level(level: &LevelSystem, viewport_size: UVec2) -> Self {
        level.with_world(|world| Self {
            change_tick: world.read_change_tick(),
            lifecycle_visibility_revision: world.lifecycle_visibility_revision(),
            active_camera: world.active_camera(),
            viewport_size,
        })
    }
}

fn cache_status_for_key(
    cached_key: Option<RuntimeFrameExtractCacheKey>,
    current_key: RuntimeFrameExtractCacheKey,
) -> RuntimeFrameExtractCacheStatus {
    if cached_key == Some(current_key) {
        RuntimeFrameExtractCacheStatus::Reused
    } else {
        RuntimeFrameExtractCacheStatus::Rebuilt
    }
}

#[derive(Clone, Debug)]
struct RuntimeFrameExtractCacheEntry {
    key: RuntimeFrameExtractCacheKey,
    extract: RenderFrameExtract,
    diagnostics_summary: RuntimeFrameExtractDiagnosticsSummary,
}

pub(super) struct RuntimeFrameExtractCacheResult {
    pub(super) extract: RenderFrameExtract,
    pub(super) status: RuntimeFrameExtractCacheStatus,
    pub(super) diagnostics_summary: RuntimeFrameExtractDiagnosticsSummary,
}

#[derive(Clone, Debug, Default)]
pub(super) struct RuntimeFrameExtractCache {
    entry: Option<RuntimeFrameExtractCacheEntry>,
}

impl RuntimeFrameExtractCache {
    pub(super) fn current_extract(
        &mut self,
        level: &LevelSystem,
        viewport_size: UVec2,
    ) -> RuntimeFrameExtractCacheResult {
        let key = RuntimeFrameExtractCacheKey::from_level(level, viewport_size);
        let status = cache_status_for_key(self.entry.as_ref().map(|entry| entry.key), key);
        if status == RuntimeFrameExtractCacheStatus::Reused {
            let entry = self
                .entry
                .as_ref()
                .expect("reused extract status requires a cache entry");
            // `RenderFrameExtract::clone` copies only the compact submission
            // overlay and shared scene-domain handles.
            return RuntimeFrameExtractCacheResult {
                extract: entry.extract.clone(),
                status,
                diagnostics_summary: entry.diagnostics_summary,
            };
        }

        let extract = level.with_world(|world| {
            world
                .to_render_frame_extract()
                .with_viewport_size(viewport_size)
        });
        let diagnostics_summary = RuntimeFrameExtractDiagnosticsSummary::from_extract(&extract);
        // Retain the same immutable scene generation; no scene vector is copied.
        self.entry = Some(RuntimeFrameExtractCacheEntry {
            key,
            extract: extract.clone(),
            diagnostics_summary,
        });
        RuntimeFrameExtractCacheResult {
            extract,
            status: RuntimeFrameExtractCacheStatus::Rebuilt,
            diagnostics_summary,
        }
    }

    pub(super) fn invalidate(&mut self) {
        self.entry = None;
    }
}

#[cfg(test)]
mod tests {
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
    fn every_cache_key_component_independently_requires_rebuild() {
        let baseline = RuntimeFrameExtractCacheKey {
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
}
