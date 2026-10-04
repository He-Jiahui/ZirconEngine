use zircon_runtime::scene::components::NodeKind;
use zircon_runtime_interface::math::{Transform, Vec3};

use super::*;

#[test]
fn applied_batch_validation_rejects_world_changes_before_commit() {
    let mut scene = Scene::empty();
    let node_id = scene.spawn_node(NodeKind::Cube).unwrap();
    let before = NodeEditState::capture(&scene, node_id).unwrap();
    let mut after = before.clone();
    after.transform = Transform::from_translation(Vec3::new(2.0, 0.0, 0.0));
    scene.update_transform(node_id, after.transform).unwrap();
    let expected_world_generation = scene.world_generation();
    let targets = vec![BatchTransformTarget::new(node_id, before, after).unwrap()];

    validate_applied_targets(&scene, &targets, expected_world_generation).unwrap();
    scene.rename_node(node_id, "Externally changed").unwrap();
    assert!(
        validate_applied_targets(&scene, &targets, expected_world_generation).is_err(),
        "an externally advanced world must not commit stale already-applied snapshots"
    );
}
