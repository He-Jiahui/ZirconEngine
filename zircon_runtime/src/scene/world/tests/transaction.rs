use crate::core::framework::scene::{ComponentPropertyPath, EntityPath};
use crate::scene::NodeKind;

use super::World;

#[test]
fn staged_world_commit_stales_compiled_binding_when_entity_ids_are_reused() {
    let mut current = World::empty();
    let root = current
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let hero = current
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
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
    let staged_root = staged
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let staged_hero = staged
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
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
