use std::collections::BTreeMap;

use crate::core::framework::render::{
    ParticleExtract, RenderLayerSet, RenderMeshSnapshot, RenderSpriteSnapshot, VisibilityInput,
    VisibilityRenderableInput,
};
use crate::scene::{
    EntityId,
    components::{Mobility, default_render_layer_mask},
};

pub(super) fn build_visibility_input(
    meshes: &[RenderMeshSnapshot],
    sprites: &[RenderSpriteSnapshot],
    particles: &ParticleExtract,
) -> VisibilityInput {
    let particle_render_layer_masks = particle_emitter_render_layer_masks(particles);
    let mut renderables = meshes
        .iter()
        .map(|mesh| VisibilityRenderableInput {
            entity: mesh.node_id,
            stable_instance_key: mesh.stable_instance_key,
            mobility: mesh.mobility,
            render_layer_mask: mesh.common.layer_mask.clone(),
        })
        .chain(sprites.iter().map(|sprite| VisibilityRenderableInput {
            entity: sprite.entity,
            stable_instance_key: sprite.entity,
            mobility: if sprite.common.is_static {
                Mobility::Static
            } else {
                Mobility::Dynamic
            },
            render_layer_mask: sprite.common.layer_mask.clone(),
        }))
        .chain(particles.emitters.iter().map(|entity| {
            VisibilityRenderableInput {
                entity: *entity,
                stable_instance_key: *entity,
                mobility: Mobility::Dynamic,
                render_layer_mask: particle_render_layer_masks
                    .get(entity)
                    .cloned()
                    .unwrap_or_else(|| {
                        RenderLayerSet::from_scene_schema_v1_mask(default_render_layer_mask())
                    }),
            }
        }))
        .collect::<Vec<_>>();
    renderables.sort_by_key(|entry| (entry.stable_instance_key, entry.entity));
    let (renderable_entities, static_entities, dynamic_entities) =
        project_visibility_entity_sets(&renderables);

    VisibilityInput {
        renderable_entities,
        static_entities,
        dynamic_entities,
        renderables,
    }
}

fn project_visibility_entity_sets(
    renderables: &[VisibilityRenderableInput],
) -> (Vec<EntityId>, Vec<EntityId>, Vec<EntityId>) {
    let mut renderable_entities = Vec::with_capacity(renderables.len());
    let mut static_entities = Vec::with_capacity(renderables.len());
    let mut dynamic_entities = Vec::with_capacity(renderables.len());
    for entry in renderables {
        renderable_entities.push(entry.entity);
        match entry.mobility {
            Mobility::Static => static_entities.push(entry.entity),
            Mobility::Dynamic => dynamic_entities.push(entry.entity),
        }
    }
    sort_and_dedup_entities(&mut renderable_entities);
    sort_and_dedup_entities(&mut static_entities);
    sort_and_dedup_entities(&mut dynamic_entities);
    (renderable_entities, static_entities, dynamic_entities)
}

fn sort_and_dedup_entities(entities: &mut Vec<EntityId>) {
    entities.sort_unstable();
    entities.dedup();
}

fn particle_emitter_render_layer_masks(
    particles: &ParticleExtract,
) -> BTreeMap<crate::scene::EntityId, RenderLayerSet> {
    let mut layer_masks: BTreeMap<crate::scene::EntityId, RenderLayerSet> = BTreeMap::new();
    for sprite in &particles.sprites {
        let sprite_render_layer_mask = sprite.render_layer_mask.clone();
        layer_masks
            .entry(sprite.entity)
            .and_modify(|layer_mask| {
                *layer_mask = RenderLayerSet::union(&*layer_mask, &sprite_render_layer_mask);
            })
            .or_insert(sprite_render_layer_mask);
    }
    layer_masks
}

pub(super) fn empty_visibility_input() -> VisibilityInput {
    VisibilityInput {
        renderable_entities: Vec::new(),
        static_entities: Vec::new(),
        dynamic_entities: Vec::new(),
        renderables: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_entity_projection_preserves_sorted_unique_mobility_sets() {
        let renderables = vec![
            renderable(30, 1, Mobility::Dynamic),
            renderable(10, 2, Mobility::Static),
            renderable(30, 3, Mobility::Static),
            renderable(20, 4, Mobility::Dynamic),
            renderable(10, 5, Mobility::Static),
        ];

        let (all, static_entities, dynamic_entities) = project_visibility_entity_sets(&renderables);

        assert_eq!(all, vec![10, 20, 30]);
        assert_eq!(static_entities, vec![10, 30]);
        assert_eq!(dynamic_entities, vec![20, 30]);
    }

    fn renderable(
        entity: EntityId,
        stable_instance_key: u64,
        mobility: Mobility,
    ) -> VisibilityRenderableInput {
        VisibilityRenderableInput {
            entity,
            stable_instance_key,
            mobility,
            render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(1),
        }
    }
}
