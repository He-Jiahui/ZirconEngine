use crate::core::framework::animation::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationInterpolationAsset, AnimationSequenceAsset, AnimationSequenceBindingAsset,
    AnimationSequenceTrackAsset,
};
use crate::core::framework::scene::{ComponentPropertyPath, EntityPath};
use crate::core::resource::{AnimationClipMarker, ResourceHandle, ResourceId};
use crate::scene::World;
use crate::scene::components::{AnimationPlayerComponent, MeshRenderer, NodeKind};

use super::{
    apply_compiled_sequence_to_world, apply_sequence_to_world, compile_sequence_for_world,
};

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

#[test]
fn compiled_sequence_resolves_numeric_target_once_and_writes_through_compiled_property() {
    let mut world = World::empty();
    let hero = world.spawn_node(NodeKind::Mesh);
    world
        .set_animation_player(
            hero,
            Some(AnimationPlayerComponent {
                clip: ResourceHandle::<AnimationClipMarker>::new(ResourceId::from_stable_label(
                    "res://animation/hero.clip.zranim",
                )),
                playback_speed: 1.0,
                time_seconds: 0.0,
                weight: 0.25,
                looping: true,
                playing: true,
            }),
        )
        .unwrap();
    let sequence = AnimationSequenceAsset {
        name: Some("Numeric target".to_string()),
        duration_seconds: 1.0,
        frames_per_second: 30.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Unused fallback").unwrap(),
            target_id: Some(hero.to_string()),
            tracks: vec![AnimationSequenceTrackAsset {
                property_path: ComponentPropertyPath::parse("AnimationPlayer.weight").unwrap(),
                channel: AnimationChannelAsset {
                    interpolation: AnimationInterpolationAsset::Step,
                    keys: vec![AnimationChannelKeyAsset {
                        time_seconds: 0.0,
                        value: AnimationChannelValueAsset::Scalar(2.0),
                        in_tangent: None,
                        out_tangent: None,
                    }],
                },
            }],
        }],
    };

    let compiled = compile_sequence_for_world(&mut world, &sequence).unwrap();
    let report =
        apply_compiled_sequence_to_world(&mut world, &sequence, &compiled, 0.0, false).unwrap();

    assert_eq!(report.applied_tracks, 1);
    assert_eq!(report.missing_tracks, 0);
    assert_eq!(
        world.get::<AnimationPlayerComponent>(hero).unwrap().weight,
        2.0
    );
}

#[test]
fn compiled_sequence_reports_stale_target_without_re_resolving_raw_path() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty);
    let hero = world.spawn_node(NodeKind::Mesh);
    world.rename_node(root, "Root").unwrap();
    world.rename_node(hero, "Hero").unwrap();
    world.set_parent_checked(hero, Some(root)).unwrap();
    world
        .set_animation_player(
            hero,
            Some(AnimationPlayerComponent {
                clip: ResourceHandle::<AnimationClipMarker>::new(ResourceId::from_stable_label(
                    "res://animation/hero.clip.zranim",
                )),
                playback_speed: 1.0,
                time_seconds: 0.0,
                weight: 0.25,
                looping: true,
                playing: true,
            }),
        )
        .unwrap();
    let sequence = AnimationSequenceAsset {
        name: Some("Stale target".to_string()),
        duration_seconds: 1.0,
        frames_per_second: 30.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Root/Hero").unwrap(),
            target_id: None,
            tracks: vec![AnimationSequenceTrackAsset {
                property_path: ComponentPropertyPath::parse("AnimationPlayer.weight").unwrap(),
                channel: AnimationChannelAsset {
                    interpolation: AnimationInterpolationAsset::Step,
                    keys: vec![AnimationChannelKeyAsset {
                        time_seconds: 0.0,
                        value: AnimationChannelValueAsset::Scalar(2.0),
                        in_tangent: None,
                        out_tangent: None,
                    }],
                },
            }],
        }],
    };
    let compiled = compile_sequence_for_world(&mut world, &sequence).unwrap();
    assert!(compiled.is_current_for(&world));
    world.rename_node(hero, "Renamed hero").unwrap();
    assert!(!compiled.is_current_for(&world));

    let report =
        apply_compiled_sequence_to_world(&mut world, &sequence, &compiled, 0.0, false).unwrap();

    assert_eq!(report.applied_tracks, 0);
    assert_eq!(report.missing_tracks, 1);
    assert_eq!(
        world.get::<AnimationPlayerComponent>(hero).unwrap().weight,
        0.25
    );
}

#[test]
fn compiled_sequence_retries_missing_target_only_after_topology_catalog_changes() {
    let mut world = World::empty();
    let sequence = AnimationSequenceAsset {
        name: Some("Missing target".to_string()),
        duration_seconds: 1.0,
        frames_per_second: 30.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Root/Hero").unwrap(),
            target_id: None,
            tracks: vec![AnimationSequenceTrackAsset {
                property_path: ComponentPropertyPath::parse("Transform.translation.x").unwrap(),
                channel: AnimationChannelAsset {
                    interpolation: AnimationInterpolationAsset::Step,
                    keys: vec![AnimationChannelKeyAsset {
                        time_seconds: 0.0,
                        value: AnimationChannelValueAsset::Scalar(2.0),
                        in_tangent: None,
                        out_tangent: None,
                    }],
                },
            }],
        }],
    };

    let compiled = compile_sequence_for_world(&mut world, &sequence).unwrap();
    assert_eq!(compiled.missing_tracks().len(), 1);
    assert!(compiled.is_current_for(&world));

    world.spawn_node(NodeKind::Empty);

    assert!(!compiled.is_current_for(&world));
}
