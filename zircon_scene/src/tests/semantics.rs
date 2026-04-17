use crate::{
    ComponentData, DefaultLevelManager, EntityIdentity, RuntimeObject, RuntimeSystem,
};

#[derive(Clone, Debug)]
struct DemoComponent;

impl ComponentData for DemoComponent {}

fn assert_component<T: ComponentData>() {}

#[test]
fn runtime_semantics_keep_ecs_roles_explicit() {
    let level = DefaultLevelManager::default().create_default_level();
    let entity = 7_u64;

    assert_component::<DemoComponent>();
    assert_eq!(entity.entity_id(), entity);
    assert_eq!(level.object_kind(), "system");
    assert_eq!(level.system_name(), "LevelSystem");
}
