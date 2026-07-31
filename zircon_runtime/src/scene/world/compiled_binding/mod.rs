mod generation;
mod index;
mod property_path;

pub(super) use generation::SceneBindingGenerations;
pub use index::{CompiledDescendantNameEntry, CompiledDescendantNameIndex};
pub use property_path::{
    CompiledScenePropertyTarget, CompiledTransformPropertyTarget, ComponentFieldId, PathId,
};

use super::World;
use crate::scene::EntityId;

impl World {
    /// Returns the generation used to invalidate compiled scene topology bindings.
    pub fn scene_binding_generation(&self, root: EntityId) -> u64 {
        self.scene_binding_generations.for_root(root)
    }

    /// Compiles a dense, hierarchy-ordered name projection for a root's descendants.
    ///
    /// The projection belongs to the scene runtime. Consumers may retain it across
    /// frames and must recompile only when the root's
    /// [`Self::scene_binding_generation`] changes.
    pub fn compile_descendant_name_index(
        &self,
        root: EntityId,
    ) -> Option<CompiledDescendantNameIndex> {
        if !self.contains_entity(root) {
            return None;
        }

        let entries = self
            .subtree_entity_ids(root)
            .into_iter()
            .filter(|entity| *entity != root)
            .filter_map(|entity| {
                self.names.get(&entity).map(|name| {
                    CompiledDescendantNameEntry::new(entity, name.0.clone().into_boxed_str())
                })
            })
            .collect();

        Some(CompiledDescendantNameIndex::new(
            root,
            self.scene_binding_generation(root),
            entries,
        ))
    }

    pub(super) fn advance_scene_binding_generation_for_name(&mut self, entity: EntityId) {
        self.advance_scene_binding_generations(Some(entity));
    }

    pub(super) fn advance_scene_binding_generations_for_reparent(
        &mut self,
        entity: EntityId,
        previous_parent: Option<EntityId>,
        current_parent: Option<EntityId>,
    ) {
        let mut roots = self.scene_binding_ancestor_chain(Some(entity));
        roots.extend(self.scene_binding_ancestor_chain(previous_parent));
        roots.extend(self.scene_binding_ancestor_chain(current_parent));
        roots.sort_unstable();
        roots.dedup();
        self.scene_binding_generations.advance_roots(roots);
    }

    pub(super) fn advance_scene_binding_generations_for_removal(
        &mut self,
        entity: EntityId,
        previous_parent: Option<EntityId>,
    ) {
        // The removed entity no longer has a hierarchy edge, but its identifier
        // can be inserted again. Advance that tombstone root as well as the old
        // ancestor chain so retained bindings cannot cross entity lifetimes.
        let mut roots = vec![entity];
        roots.extend(self.scene_binding_ancestor_chain(previous_parent));
        roots.sort_unstable();
        roots.dedup();
        self.scene_binding_generations.advance_roots(roots);
    }

    pub(super) fn advance_scene_binding_generations_for_new_descendant(
        &mut self,
        entity: EntityId,
    ) {
        self.advance_scene_binding_generations(Some(entity));
    }

    pub(super) fn invalidate_all_scene_binding_generations(&mut self) {
        self.scene_binding_generations
            .advance_roots(self.entities.iter().copied());
    }

    fn advance_scene_binding_generations(&mut self, first_root: Option<EntityId>) {
        self.scene_binding_generations
            .advance_roots(self.scene_binding_ancestor_chain(first_root));
    }

    fn scene_binding_ancestor_chain(&self, first_root: Option<EntityId>) -> Vec<EntityId> {
        let mut roots = Vec::new();
        let mut current = first_root;
        let mut remaining = self.entities.len().saturating_add(1);
        while let Some(entity) = current {
            if remaining == 0 {
                break;
            }
            roots.push(entity);
            current = self.parent_of(entity);
            remaining -= 1;
        }
        roots
    }
}

#[cfg(test)]
mod tests;
