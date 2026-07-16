use super::{SelectionModel, WorldDomain};
use crate::scene::viewport::SceneViewportController;
use zircon_runtime_interface::math::UVec2;

#[test]
fn selection_preserves_order_primary_and_generation() {
    let mut selection = SelectionModel::default();

    assert!(selection.replace(WorldDomain::Edit, [7, 3, 7, 9], Some(3)));
    assert_eq!(ordered_items(&selection, WorldDomain::Edit), [7, 3, 9]);
    assert_eq!(selection.primary(WorldDomain::Edit), Some(3));
    assert_eq!(selection.generation(WorldDomain::Edit), 1);

    assert!(!selection.replace(WorldDomain::Edit, [7, 3, 9], Some(3)));
    assert_eq!(selection.generation(WorldDomain::Edit), 1);
}

#[test]
fn edit_and_play_selection_are_isolated() {
    let mut selection = SelectionModel::default();
    selection.select_only(WorldDomain::Edit, 11);
    selection.select_only(WorldDomain::Play, 42);

    assert_eq!(selection.active_domain(), WorldDomain::Edit);
    assert_eq!(active_items(&selection), [11]);
    let revision_before_switch = selection.revision();

    assert!(selection.set_active_domain(WorldDomain::Play));
    assert_eq!(active_items(&selection), [42]);
    assert_eq!(selection.active_primary(), Some(42));
    assert_eq!(ordered_items(&selection, WorldDomain::Edit), [11]);
    assert_eq!(selection.revision(), revision_before_switch + 1);
    assert!(!selection.set_active_domain(WorldDomain::Play));
}

#[test]
fn toggle_and_extend_keep_primary_inside_the_set() {
    let mut selection = SelectionModel::default();

    selection.select_only(WorldDomain::Edit, 10);
    selection.extend(WorldDomain::Edit, [20, 30, 20]);
    assert_eq!(ordered_items(&selection, WorldDomain::Edit), [10, 20, 30]);
    assert_eq!(selection.primary(WorldDomain::Edit), Some(30));

    assert!(selection.toggle(WorldDomain::Edit, 30));
    assert_eq!(ordered_items(&selection, WorldDomain::Edit), [10, 20]);
    assert_eq!(selection.primary(WorldDomain::Edit), Some(20));

    assert!(selection.clear(WorldDomain::Edit));
    assert!(selection.items(WorldDomain::Edit).is_empty());
    assert_eq!(selection.primary(WorldDomain::Edit), None);
    assert!(!selection.clear(WorldDomain::Edit));
}

#[test]
fn viewport_selection_uses_the_active_world_domain_model() {
    let mut viewport = SceneViewportController::new(UVec2::new(1280, 720));

    assert!(viewport.selection_mut().select_only_active(11));
    assert_eq!(viewport.selection().active_primary(), Some(11));
    assert_eq!(ordered_items(viewport.selection(), WorldDomain::Edit), [11]);

    assert!(viewport
        .selection_mut()
        .set_active_domain(WorldDomain::Play));
    assert_eq!(viewport.selection().active_primary(), None);
    assert!(viewport.selection_mut().select_only_active(42));
    assert_eq!(ordered_items(viewport.selection(), WorldDomain::Play), [42]);
    assert_eq!(ordered_items(viewport.selection(), WorldDomain::Edit), [11]);
}

fn ordered_items(selection: &SelectionModel, domain: WorldDomain) -> Vec<u64> {
    selection.items(domain).iter().copied().collect()
}

fn active_items(selection: &SelectionModel) -> Vec<u64> {
    selection.active_items().iter().copied().collect()
}
