use zircon_runtime::core::framework::animation::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationInterpolationAsset, AnimationSequenceAsset, AnimationSequenceBindingAsset,
    AnimationSequenceTrackAsset,
};
use zircon_runtime::core::framework::scene::{ComponentPropertyPath, EntityPath};
use zircon_runtime::scene::components::{MeshRenderer, NodeKind};
use zircon_runtime::scene::World;

use super::apply_sequence_to_world;

#[test]
fn sequence_applies_mesh_renderer_morph_weight_track() {
    let mut world = World::new();
    let hero = world.spawn_node(NodeKind::Mesh);
    world.rename_node(hero, "Hero").unwrap();
    let track_path = ComponentPropertyPath::parse("MeshRenderer.morph_weights.1").unwrap();
    let sequence = AnimationSequenceAsset {
        name: Some("Blink".to_string()),
        duration_seconds: 1.0,
        frames_per_second: 30.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Hero").unwrap(),
            target_id: None,
            tracks: vec![AnimationSequenceTrackAsset {
                property_path: track_path.clone(),
                channel: AnimationChannelAsset {
                    interpolation: AnimationInterpolationAsset::Step,
                    keys: vec![AnimationChannelKeyAsset {
                        time_seconds: 0.0,
                        value: AnimationChannelValueAsset::Scalar(0.7),
                        in_tangent: None,
                        out_tangent: None,
                    }],
                },
            }],
        }],
    };

    let report = apply_sequence_to_world(&mut world, &sequence, 0.0, false).unwrap();

    assert_eq!(report.applied_tracks.len(), 1);
    assert!(report.missing_tracks.is_empty());
    assert_eq!(
        world
            .get::<MeshRenderer>(hero)
            .unwrap()
            .morph_weights
            .as_slice(),
        &[0.0, 0.7]
    );
}
