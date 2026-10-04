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
