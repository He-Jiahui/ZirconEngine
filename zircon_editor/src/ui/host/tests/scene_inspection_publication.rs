use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::{EntityId, Scene};

use super::SceneInspectionPublication;

#[test]
fn stable_large_selection_rename_reuses_the_published_selection_snapshot() {
    const SELECTED_ENTITY_COUNT: usize = 10_000;
    const SELECTION_REVISION: u64 = 42;

    let mut scene = Scene::new();
    let selected_entities = (0..SELECTED_ENTITY_COUNT)
        .map(|_| {
            scene
                .spawn_node(NodeKind::Empty)
                .expect("test scene spawn should succeed")
        })
        .collect::<Vec<_>>();
    let renamed_entity = selected_entities[0];
    let mut publication = SceneInspectionPublication::default();
    publication.reset(
        &scene,
        Some(renamed_entity),
        SELECTION_REVISION,
        selected_entities.into_iter(),
    );
    scene
        .rename_node(renamed_entity, "Renamed selected scene item")
        .expect("selected entity should remain available for rename");

    let message = publication
        .observe(
            &scene,
            Some(renamed_entity),
            SELECTION_REVISION,
            std::iter::from_fn(|| -> Option<EntityId> {
                panic!("stable selection must reuse the published Arc instead of collecting")
            }),
        )
        .expect("renaming a selected node should publish a sparse hierarchy patch");

    assert_eq!(message.changed_anchors().len(), 1);
    assert_eq!(
        message.selection().previous_revision(),
        Some(SELECTION_REVISION)
    );
    assert_eq!(message.selection().revision(), SELECTION_REVISION);
    assert!(message.selection().added_entities().is_empty());
    assert!(message.selection().removed_entities().is_empty());
}
