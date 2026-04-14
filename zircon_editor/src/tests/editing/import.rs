use zircon_scene::NodeKind;
use zircon_resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};

use crate::EditorIntent;

use super::support::test_state;

#[test]
fn imported_mesh_can_be_undone() {
    let mut state = test_state();
    let initial_count = state.world.snapshot().nodes().len();

    assert!(state
        .import_mesh_asset(
            ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("res://models/test.obj")),
            ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "res://materials/default.material.toml",
            )),
            "res://models/test.obj",
        )
        .unwrap());
    let imported = state.world.snapshot();
    assert_eq!(imported.nodes().len(), initial_count + 1);
    assert!(matches!(
        imported.nodes().last().map(|node| &node.kind),
        Some(NodeKind::Mesh)
    ));

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert_eq!(state.world.snapshot().nodes().len(), initial_count);
}
