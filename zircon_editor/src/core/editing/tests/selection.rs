use super::{SceneSelection, SelectionSnapshot};

#[test]
fn typed_scene_selection_handles_share_their_payload_at_scale() {
    for count in [1_u64, 100, 10_000] {
        let model = SceneSelection::new((1..=count).collect(), Some(count));
        let snapshot = SelectionSnapshot::scene(count, model.clone());
        let cloned = snapshot.clone();
        let restored = cloned.scene_selection().unwrap();

        assert_eq!(restored.items().len(), count as usize);
        assert_eq!(restored.primary(), Some(count));
        assert!(snapshot.shares_payload_with(&cloned));
        assert!(model.shares_items_with(&restored));
    }
}
