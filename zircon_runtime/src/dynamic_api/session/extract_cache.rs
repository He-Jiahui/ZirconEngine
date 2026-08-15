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
        if let Some(entry) = &self.entry {
            if entry.key == key {
                return RuntimeFrameExtractCacheResult {
                    extract: entry.extract.clone(),
                    status: RuntimeFrameExtractCacheStatus::Reused,
                    diagnostics_summary: entry.diagnostics_summary,
                };
            }
        }

        let extract = level.with_world(|world| {
            world
                .to_render_frame_extract()
                .with_viewport_size(viewport_size)
        });
        let diagnostics_summary = RuntimeFrameExtractDiagnosticsSummary::from_extract(&extract);
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
