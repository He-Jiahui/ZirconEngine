use crate::core::framework::scene::WorldHandle;

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
mod tests {
    use super::RenderWorldSnapshotHandle;

    #[test]
    fn snapshot_handle_keeps_world_identity_and_source_generation_distinct() {
        let snapshot = RenderWorldSnapshotHandle::new(7).with_generation(19);

        assert_eq!(snapshot.raw(), 7);
        assert_eq!(snapshot.generation(), 19);
    }
}
