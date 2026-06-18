use crate::core::framework::render::RenderFrameExtract;
use crate::core::math::UVec2;
use crate::scene::ecs::ChangeTick;
use crate::scene::{EntityId, LevelSystem};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RuntimeFrameExtractCacheStatus {
    Rebuilt,
    Reused,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RuntimeFrameExtractCacheKey {
    change_tick: ChangeTick,
    query_cache_revision: u64,
    active_camera: EntityId,
    viewport_size: UVec2,
}

impl RuntimeFrameExtractCacheKey {
    fn from_level(level: &LevelSystem, viewport_size: UVec2) -> Self {
        level.with_world(|world| Self {
            change_tick: world.read_change_tick(),
            query_cache_revision: world.query_cache_revision(),
            active_camera: world.active_camera(),
            viewport_size,
        })
    }
}

#[derive(Clone, Debug)]
struct RuntimeFrameExtractCacheEntry {
    key: RuntimeFrameExtractCacheKey,
    extract: RenderFrameExtract,
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
    ) -> (RenderFrameExtract, RuntimeFrameExtractCacheStatus) {
        let key = RuntimeFrameExtractCacheKey::from_level(level, viewport_size);
        if let Some(entry) = &self.entry {
            if entry.key == key {
                return (
                    entry.extract.clone(),
                    RuntimeFrameExtractCacheStatus::Reused,
                );
            }
        }

        let extract = level.with_world(|world| {
            world
                .to_render_frame_extract()
                .with_viewport_size(viewport_size)
        });
        self.entry = Some(RuntimeFrameExtractCacheEntry {
            key,
            extract: extract.clone(),
        });
        (extract, RuntimeFrameExtractCacheStatus::Rebuilt)
    }

    pub(super) fn invalidate(&mut self) {
        self.entry = None;
    }
}
