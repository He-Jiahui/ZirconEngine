use std::collections::HashMap;

use crate::scene::EntityId;

use super::property_path::{ComponentFieldId, PathId};

#[derive(Clone, Debug)]
pub(in crate::scene::world) struct SceneBindingGenerations {
    next: u64,
    catalog_generation: u64,
    by_root: HashMap<EntityId, u64>,
    next_path_id: u64,
    path_ids: HashMap<Box<str>, PathId>,
    next_component_field_id: u64,
    component_field_ids: HashMap<Box<str>, ComponentFieldId>,
}

impl Default for SceneBindingGenerations {
    fn default() -> Self {
        Self {
            next: 0,
            catalog_generation: 0,
            by_root: HashMap::new(),
            next_path_id: 1,
            path_ids: HashMap::new(),
            next_component_field_id: 1,
            component_field_ids: HashMap::new(),
        }
    }
}

impl SceneBindingGenerations {
    pub(super) fn for_root(&self, root: EntityId) -> u64 {
        self.by_root.get(&root).copied().unwrap_or_default()
    }

    pub(super) const fn catalog_generation(&self) -> u64 {
        self.catalog_generation
    }

    pub(super) fn advance_roots<I>(&mut self, roots: I)
    where
        I: IntoIterator<Item = EntityId>,
    {
        let roots = roots.into_iter().collect::<Vec<_>>();
        if roots.is_empty() {
            return;
        }

        self.next = self.next.saturating_add(1);
        self.catalog_generation = self
            .catalog_generation
            .checked_add(1)
            .expect("scene binding catalog generation must not exhaust u64");
        for root in roots {
            self.by_root.insert(root, self.next);
        }
    }

    /// Invalidates replacement-world bindings past every generation published
    /// by the retired world, including when entity identifiers are reused.
    pub(super) fn advance_roots_after<I>(&mut self, previous: &Self, roots: I)
    where
        I: IntoIterator<Item = EntityId>,
    {
        let roots = roots.into_iter().collect::<Vec<_>>();
        if roots.is_empty() {
            return;
        }

        self.next = self
            .next
            .max(previous.next)
            .checked_add(1)
            .expect("scene binding generations must not exhaust u64");
        self.catalog_generation = self
            .catalog_generation
            .max(previous.catalog_generation)
            .checked_add(1)
            .expect("scene binding catalog generation must not exhaust u64");
        for root in roots {
            self.by_root.insert(root, self.next);
        }
    }

    pub(super) fn intern_path(&mut self, path: &str) -> PathId {
        if let Some(id) = self.path_ids.get(path).copied() {
            return id;
        }

        let id = PathId(self.next_path_id);
        self.next_path_id = self
            .next_path_id
            .checked_add(1)
            .expect("scene path identifiers must not exhaust u64");
        self.path_ids.insert(path.into(), id);
        id
    }

    pub(super) fn intern_component_field(&mut self, field: &str) -> ComponentFieldId {
        if let Some(id) = self.component_field_ids.get(field).copied() {
            return id;
        }

        let id = ComponentFieldId(self.next_component_field_id);
        self.next_component_field_id = self
            .next_component_field_id
            .checked_add(1)
            .expect("scene component field identifiers must not exhaust u64");
        self.component_field_ids.insert(field.into(), id);
        id
    }
}

// Runtime-only cache generations do not participate in persistent world equality.
impl PartialEq for SceneBindingGenerations {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
