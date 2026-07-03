use super::animation_assets::{
    sequence_asset_for_entity, single_hand_translation_clip, two_bone_skeleton,
};
use super::runtime_helpers::{runtime_asset_manager, runtime_with_physics_animation_scene_asset};
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::manager::resolve_animation_manager;
use zircon_runtime::core::math::Vec3;
use zircon_runtime::core::resource::{
    AnimationSequenceMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use zircon_runtime::scene::components::{AnimationSequencePlayerComponent, NodeKind};

#[test]
fn clip_sampling_resolves_track_target_id_before_bone_name_fallback() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let animation = resolve_animation_manager(&runtime.handle()).unwrap();
    let skeleton = two_bone_skeleton();
    let mut clip = single_hand_translation_clip(
        &AssetUri::parse("res://animation/target-id.clip.zranim").unwrap(),
        5.0,
    );
    clip.tracks[0].bone_name = "MissingByName".to_string();
    clip.tracks[0].target_id = Some("Root/Hand".to_string());

    let pose = animation
        .sample_clip_pose(&skeleton, &clip, 0.0, false)
        .unwrap();

    let hand = pose.bones.iter().find(|bone| bone.name == "Hand").unwrap();
    assert!(hand
        .local_transform
        .translation
        .abs_diff_eq(Vec3::new(5.0, 0.0, 0.0), 1.0e-4));
}

#[test]
fn sequence_runtime_resolves_target_id_before_entity_path_fallback() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let core = runtime.handle();
    let target_entity_name = "Runtime Sequence Target";
    let sequence_uri = AssetUri::parse("res://animation/target-id.sequence.zranim").unwrap();
    let sequence_id = ResourceId::from_locator(&sequence_uri);
    let asset_manager = runtime_asset_manager(&core);
    let mut sequence = sequence_asset_for_entity("Wrong/Path");
    sequence.bindings[0].target_id = Some(target_entity_name.to_string());
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(sequence_id, ResourceKind::AnimationSequence, sequence_uri),
        sequence,
    );
    let level = runtime.create_default_level().unwrap();
    let cube = level.with_world_mut(|world| {
        let cube = world.spawn_node(NodeKind::Cube);
        world.rename_node(cube, target_entity_name).unwrap();
        world
            .set_animation_sequence_player(
                cube,
                Some(AnimationSequencePlayerComponent {
                    sequence: ResourceHandle::<AnimationSequenceMarker>::new(sequence_id),
                    playback_speed: 1.0,
                    time_seconds: 0.0,
                    looping: false,
                    playing: true,
                }),
            )
            .unwrap();
        cube
    });

    runtime.tick_level_seconds(&level, 0.5).unwrap();

    let translation =
        level.with_world(|world| world.find_node(cube).unwrap().transform.translation);
    assert_eq!(translation, Vec3::new(2.0, 0.0, 0.0));
}
