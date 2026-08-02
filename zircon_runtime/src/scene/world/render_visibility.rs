use std::collections::{BTreeMap, BTreeSet};

use crate::core::framework::render::{
    ParticleExtract, RenderLayerSet, RenderMeshSnapshot, RenderSpriteSnapshot, VisibilityInput,
    VisibilityRenderableInput,
};
use crate::scene::components::{Mobility, default_render_layer_mask};

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
    let renderable_entities = renderables
        .iter()
        .map(|entry| entry.entity)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let static_entities = renderables
        .iter()
        .filter(|entry| entry.mobility == Mobility::Static)
        .map(|entry| entry.entity)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let dynamic_entities = renderables
        .iter()
        .filter(|entry| entry.mobility == Mobility::Dynamic)
        .map(|entry| entry.entity)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    VisibilityInput {
        renderable_entities,
        static_entities,
        dynamic_entities,
        renderables,
    }
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
