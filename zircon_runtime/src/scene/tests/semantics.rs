use crate::scene::semantics::{ComponentData, EntityIdentity};

#[derive(Clone, Debug)]
struct DemoComponent;

impl ComponentData for DemoComponent {}

fn assert_component<T: ComponentData>() {}

#[test]
fn entity_and_component_semantics_keep_ecs_roles_explicit() {
    let entity = 7_u64;

    assert_component::<DemoComponent>();
    assert_eq!(entity.entity_id(), entity);
}
