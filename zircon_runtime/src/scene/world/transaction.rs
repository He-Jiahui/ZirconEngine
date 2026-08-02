use crate::scene::ecs::ResourceStore;

use super::World;

pub(in crate::scene) struct RetiredWorldTransactionState {
    _world: World,
    _replaced_resources: ResourceStore,
}

impl World {
    /// Creates an isolated World that can validate a dynamic-scene mutation.
    ///
    /// The target's entity rows, component storage, resources, callbacks, and
    /// runtime queues deliberately stay out of this projection. Callers stage
    /// only the rows and resources their mutation owns before applying it here.
    pub(in crate::scene) fn dynamic_scene_preflight_world(&self) -> Self {
        let mut preflight = Self::empty();
        preflight.component_types = self.component_types.clone();
        preflight.type_registry = self.type_registry.clone();
        preflight.vm_catalog_type_paths = self.vm_catalog_type_paths.clone();
        preflight.vm_dynamic_type_paths = self.vm_dynamic_type_paths.clone();
        preflight
    }

    pub(in crate::scene) fn commit_staged_scene_state(
        &mut self,
        mut staged: World,
    ) -> RetiredWorldTransactionState {
        staged.advance_dynamic_component_generations_after(self);
        staged.advance_scene_binding_generations_after(self);
        staged.advance_world_generation_after(self.world_generation());
        staged.record_staged_lifecycle_events = false;
        let staged_lifecycle_events = std::mem::take(&mut staged.staged_lifecycle_events);
        let mut live_resources = std::mem::take(&mut self.resources);
        let replaced_resources =
            live_resources.merge_overrides_from(std::mem::take(&mut staged.resources));
        staged.resources = live_resources;

        // These containers carry live callbacks, queued work, and runtime-only state.
        // Scene staging intentionally uses their empty/clone projections and must not
        // replace the authoritative instances at commit.
        staged.schedule = std::mem::take(&mut self.schedule);
        staged.removed_component_events = std::mem::take(&mut self.removed_component_events);
        staged.events = std::mem::take(&mut self.events);
        staged.event_mirrors = std::mem::take(&mut self.event_mirrors);
        staged.messages = std::mem::take(&mut self.messages);
        staged.observers = std::mem::take(&mut self.observers);
        staged.command_queue = std::mem::take(&mut self.command_queue);
        staged.deferred_command_errors = std::mem::take(&mut self.deferred_command_errors);
        staged.ecs_frame_performance_diagnostics =
            std::mem::take(&mut self.ecs_frame_performance_diagnostics);

        let retired = RetiredWorldTransactionState {
            _world: std::mem::replace(self, staged),
            _replaced_resources: replaced_resources,
        };
        for event in staged_lifecycle_events {
            self.dispatch_component_lifecycle(event);
        }
        retired
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::scene::{ComponentPropertyPath, EntityPath};
    use crate::scene::NodeKind;

    use super::World;

    #[test]
    fn staged_world_commit_stales_compiled_binding_when_entity_ids_are_reused() {
        let mut current = World::empty();
        let root = current.spawn_node(NodeKind::Empty);
        let hero = current.spawn_node(NodeKind::Mesh);
        current.rename_node(root, "Root").unwrap();
        current.rename_node(hero, "Hero").unwrap();
        current.set_parent_checked(hero, Some(root)).unwrap();
        let writer = current
            .compile_scene_property_writer(
                &EntityPath::parse("Root/Hero").unwrap(),
                &ComponentPropertyPath::parse("Transform.translation").unwrap(),
            )
            .unwrap()
            .unwrap();

        let mut staged = World::empty();
        let staged_root = staged.spawn_node(NodeKind::Empty);
        let staged_hero = staged.spawn_node(NodeKind::Mesh);
        assert_eq!(root, staged_root);
        assert_eq!(hero, staged_hero);
        staged.rename_node(staged_root, "Root").unwrap();
        staged.rename_node(staged_hero, "Hero").unwrap();
        staged
            .set_parent_checked(staged_hero, Some(staged_root))
            .unwrap();

        let _retired = current.commit_staged_scene_state(staged);

        assert!(!writer.is_current_for(&current));
    }
}
