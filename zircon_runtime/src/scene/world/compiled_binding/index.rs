use crate::scene::{EntityId, World};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledDescendantNameEntry {
    entity: EntityId,
    name: Box<str>,
}

impl CompiledDescendantNameEntry {
    pub(super) fn new(entity: EntityId, name: Box<str>) -> Self {
        Self { entity, name }
    }

    pub const fn entity(&self) -> EntityId {
        self.entity
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledDescendantNameIndex {
    root: EntityId,
    generation: u64,
    entries: Vec<CompiledDescendantNameEntry>,
}

impl CompiledDescendantNameIndex {
    pub(super) fn new(
        root: EntityId,
        generation: u64,
        entries: Vec<CompiledDescendantNameEntry>,
    ) -> Self {
        Self {
            root,
            generation,
            entries,
        }
    }

    pub const fn root(&self) -> EntityId {
        self.root
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub fn entries(&self) -> &[CompiledDescendantNameEntry] {
        &self.entries
    }

    pub fn is_current_for(&self, world: &World) -> bool {
        world.contains_entity(self.root)
            && self.generation == world.scene_binding_generation(self.root)
    }
}
