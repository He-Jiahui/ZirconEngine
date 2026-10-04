use crate::core::framework::render::{
    RenderExtractContext, RenderExtractProducer, RenderFrameExtract, RenderWorldSnapshotHandle,
    SceneViewportExtractRequest,
};
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
    source_world: RenderWorldSnapshotHandle,
    change_tick: ChangeTick,
    lifecycle_visibility_revision: u64,
    active_camera: EntityId,
    viewport_size: UVec2,
}

impl RuntimeFrameExtractCacheKey {
    fn from_level(level: &LevelSystem, viewport_size: UVec2) -> Self {
        level.with_world(|world| Self {
            source_world: world.render_world_snapshot_handle(),
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

        let context = level.with_world(|world| {
            RenderExtractContext::new(
                world.render_world_snapshot_handle(),
                SceneViewportExtractRequest::default(),
            )
        });
        let extract = level
            .build_render_frame_extract(&context)
            .with_viewport_size(viewport_size);
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
#[path = "tests/extract_cache.rs"]
mod tests;
