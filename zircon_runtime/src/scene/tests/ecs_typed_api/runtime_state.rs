use super::*;

#[test]
fn runtime_only_typed_ecs_state_is_not_serialized() {
    let mut world = World::empty();
    let entity = world
        .spawn((Name("Serialized Entity".to_string()), Health(42)))
        .unwrap();
    world.insert_resource(FrameCounter(3));

    let saved = serde_json::to_string(&world).unwrap();
    let mut loaded: World = serde_json::from_str(&saved).unwrap();

    assert!(!saved.contains("FrameCounter"));
    assert_eq!(loaded.get::<Health>(entity), None);
    assert_eq!(loaded.get_resource::<FrameCounter>(), None);
    assert_eq!(
        loaded.get::<Name>(entity),
        Some(&Name("Serialized Entity".to_string()))
    );
    let name_component_id = loaded.component_id::<Name>();
    let render_layer_mask_component_id = loaded.component_id::<RenderLayerMask>();

    assert!(loaded.contains_component_id(entity, name_component_id));
    assert!(loaded.contains_component_id(entity, render_layer_mask_component_id));
}
