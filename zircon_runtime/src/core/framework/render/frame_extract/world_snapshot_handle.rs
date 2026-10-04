use crate::core::framework::scene::WorldHandle;

/// 帧提取的源世界身份与世界代，供渲染端拒绝跨世界或超前的增量变更。
/// `raw` 标识世界，`generation` 标识该快照；两者不得互作比较。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderWorldSnapshotHandle {
    world: u64,
    generation: u64,
}

impl RenderWorldSnapshotHandle {
    pub const fn new(raw: u64) -> Self {
        Self {
            world: raw,
            generation: 0,
        }
    }

    pub const fn raw(self) -> u64 {
        self.world
    }

    pub const fn generation(self) -> u64 {
        self.generation
    }

    pub const fn with_generation(self, generation: u64) -> Self {
        Self { generation, ..self }
    }
}

impl From<WorldHandle> for RenderWorldSnapshotHandle {
    fn from(value: WorldHandle) -> Self {
        Self::new(value.get())
    }
}

#[cfg(test)]
#[path = "tests/world_snapshot_handle.rs"]
mod tests;
