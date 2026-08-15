use std::collections::BTreeMap;

use crate::scene::ecs::Component;
use crate::scene::{EntityId, World};

use super::component_row::PendingComponentRow;
use super::dynamic_component_presence::DynamicComponentPresence;
use super::fixed_components::{
    PersistentAnimationRuntimeComponentSnapshot, PersistentEntityCoreComponentSnapshot,
    PersistentLightingComponentSnapshot, PersistentPhysicsComponentSnapshot,
    PersistentRender2dComponentSnapshot, PersistentSceneRenderComponentSnapshot,
    RuntimeOnlyPostProcessComponentSnapshot,
};

fn stage_values<T>(
    world: &mut World,
    rows: &mut BTreeMap<EntityId, PendingComponentRow>,
    values: impl IntoIterator<Item = (EntityId, T)>,
) where
    T: Component,
{
    for (entity, component) in values {
        let row = rows
            .get_mut(&entity)
            .expect("component projection value must belong to a registered world entity");
        world.stage_component_row_value(row, component);
    }
}

impl World {
    pub(super) fn rebuild_component_storage_projection(&mut self) {
        let persistent_entity_core = self.persistent_entity_core_component_snapshot();
        let persistent_scene_render = self.persistent_scene_render_component_snapshot();
        let runtime_only_post_process = self.runtime_only_post_process_component_snapshot();
        let persistent_physics = self.persistent_physics_component_snapshot();
        let persistent_lighting = self.persistent_lighting_component_snapshot();
        let persistent_render_2d = self.persistent_render_2d_component_snapshot();
        let persistent_animation_runtime = self.persistent_animation_runtime_component_snapshot();
        self.rebuild_component_storage_projection_with_owned_components(
            persistent_entity_core,
            persistent_scene_render,
            runtime_only_post_process,
            persistent_physics,
            persistent_lighting,
            persistent_render_2d,
            persistent_animation_runtime,
        );
    }

    pub(in super::super) fn rebuild_component_storage_projection_with_owned_components(
        &mut self,
        persistent_entity_core: PersistentEntityCoreComponentSnapshot,
        persistent_scene_render: PersistentSceneRenderComponentSnapshot,
        runtime_only_post_process: RuntimeOnlyPostProcessComponentSnapshot,
        persistent_physics: PersistentPhysicsComponentSnapshot,
        persistent_lighting: PersistentLightingComponentSnapshot,
        persistent_render_2d: PersistentRender2dComponentSnapshot,
        persistent_animation_runtime: PersistentAnimationRuntimeComponentSnapshot,
    ) {
        let entities = self.entities.clone();
        let dynamic_component_type_ids = entities
            .iter()
            .copied()
            .map(|entity| {
                let type_ids = self
                    .dynamic_components
                    .get(&entity)
                    .map(|components| components.keys().cloned().collect::<Vec<_>>())
                    .unwrap_or_default();
                (entity, type_ids)
            })
            .collect::<Vec<_>>();

        self.component_storage = Default::default();
        let mut rows = BTreeMap::new();
        for entity in entities.iter().copied() {
            rows.insert(entity, self.begin_empty_component_row());
        }

        for (entity, type_ids) in dynamic_component_type_ids {
            let row = rows
                .get_mut(&entity)
                .expect("dynamic projection value must belong to a registered world entity");
            for type_id in type_ids {
                let component_id = self.component_registry.dynamic_component_id(&type_id);
                self.stage_component_row_value_with_id(row, component_id, DynamicComponentPresence);
            }
        }

        stage_values(self, &mut rows, persistent_entity_core.names);
        stage_values(self, &mut rows, persistent_entity_core.hierarchy);
        stage_values(self, &mut rows, persistent_entity_core.local_transforms);
        stage_values(self, &mut rows, persistent_entity_core.active_self);
        stage_values(self, &mut rows, persistent_scene_render.render_layer_masks);
        stage_values(self, &mut rows, persistent_scene_render.cameras);
        stage_values(self, &mut rows, persistent_scene_render.mesh_renderers);
        stage_values(self, &mut rows, persistent_scene_render.mobility);
        stage_values(self, &mut rows, runtime_only_post_process.settings);
        stage_values(self, &mut rows, runtime_only_post_process.volumes);
        stage_values(self, &mut rows, persistent_physics.rigid_bodies);
        stage_values(self, &mut rows, persistent_physics.colliders);
        stage_values(self, &mut rows, persistent_physics.joints);
        stage_values(self, &mut rows, persistent_lighting.ambient_lights);
        stage_values(self, &mut rows, persistent_lighting.directional_lights);
        stage_values(self, &mut rows, persistent_lighting.point_lights);
        stage_values(self, &mut rows, persistent_lighting.rect_lights);
        stage_values(self, &mut rows, persistent_lighting.spot_lights);
        stage_values(self, &mut rows, persistent_render_2d.sprite_2d);
        stage_values(self, &mut rows, persistent_render_2d.mesh_2d);
        stage_values(self, &mut rows, persistent_animation_runtime.skeletons);
        stage_values(self, &mut rows, persistent_animation_runtime.players);
        stage_values(
            self,
            &mut rows,
            persistent_animation_runtime.sequence_players,
        );
        stage_values(self, &mut rows, persistent_animation_runtime.graph_players);
        stage_values(
            self,
            &mut rows,
            persistent_animation_runtime.state_machine_players,
        );

        self.reset_archetype_index_for_projection();
        for entity in entities {
            let row = rows
                .remove(&entity)
                .expect("every rebuilt entity must own one complete pending row");
            self.commit_rebuilt_component_row(entity, row);
        }
        debug_assert!(rows.is_empty());
        self.rebuild_hierarchy_mutation_index();
        self.bump_lifecycle_visibility_revision();
        self.mark_derived_state_dirty();
    }
}
