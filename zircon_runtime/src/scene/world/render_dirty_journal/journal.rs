use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::scene::ecs::ChangeTick;
use crate::scene::EntityId;

static NEXT_RENDER_DIRTY_WORLD_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct RenderDirtyWorldId(u64);

impl RenderDirtyWorldId {
    pub(super) fn allocate() -> Self {
        let id = NEXT_RENDER_DIRTY_WORLD_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
                next.checked_add(1)
            })
            .expect("render-dirty world identity space exhausted");
        Self(id)
    }

    pub(in crate::scene::world) const fn raw(self) -> u64 {
        self.0
    }
}

/// 发布给渲染提取阶段的不可变脏实体快照；Arc 让读取方跨帧持有而不影响 World 写入。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RenderDirtyEntityJournal {
    world: RenderDirtyWorldId,
    generation: u64,
    source_world_generation: u64,
    source_change_tick: ChangeTick,
    all_entities: bool,
    entities: Arc<[EntityId]>,
}

impl RenderDirtyEntityJournal {
    pub(super) fn empty(world: RenderDirtyWorldId) -> Self {
        Self {
            world,
            generation: 0,
            source_world_generation: 0,
            source_change_tick: ChangeTick::default(),
            all_entities: false,
            entities: Arc::from([]),
        }
    }

    pub(super) fn new(
        world: RenderDirtyWorldId,
        generation: u64,
        source_world_generation: u64,
        source_change_tick: ChangeTick,
        all_entities: bool,
        entities: Vec<EntityId>,
    ) -> Self {
        Self {
            world,
            generation,
            source_world_generation,
            source_change_tick,
            all_entities,
            entities: entities.into(),
        }
    }

    pub(crate) const fn world(&self) -> RenderDirtyWorldId {
        self.world
    }

    pub(crate) const fn generation(&self) -> u64 {
        self.generation
    }

    pub(crate) const fn source_world_generation(&self) -> u64 {
        self.source_world_generation
    }

    pub(crate) const fn source_change_tick(&self) -> ChangeTick {
        self.source_change_tick
    }

    pub(crate) const fn all_entities(&self) -> bool {
        self.all_entities
    }

    pub(crate) fn entities(&self) -> &[EntityId] {
        &self.entities
    }
}
