use std::collections::BTreeMap;

use crate::core::resource::{
    AnimationClipMarker, AnimationGraphMarker, AnimationSequenceMarker, AnimationSkeletonMarker,
    AnimationStateMachineMarker, ResourceHandle, ResourceId,
};
use crate::scene::components::{
    AnimationGraphPlayerComponent, AnimationPlayerComponent, AnimationSequencePlayerComponent,
    AnimationSkeletonComponent, AnimationStateMachinePlayerComponent,
};
use crate::scene::World;

#[test]
fn persistent_animation_runtime_uses_generic_storage_across_clone_serde_and_records() {
    let mut world = World::empty();
    let entity = world
        .spawn(())
        .expect("default node should publish before animation runtime components");
    let skeleton = AnimationSkeletonComponent {
        skeleton: ResourceHandle::<AnimationSkeletonMarker>::new(ResourceId::from_stable_label(
            "res://animation/runtime08.skeleton.zranim",
        )),
    };
    let player = AnimationPlayerComponent {
        clip: ResourceHandle::<AnimationClipMarker>::new(ResourceId::from_stable_label(
            "res://animation/runtime08.clip.zranim",
        )),
        playback_speed: 1.0,
        time_seconds: 0.0,
        weight: 1.0,
        looping: true,
        playing: true,
    };
    let sequence_player = AnimationSequencePlayerComponent {
        sequence: ResourceHandle::<AnimationSequenceMarker>::new(ResourceId::from_stable_label(
            "res://animation/runtime08.sequence.zranim",
        )),
        playback_speed: 1.0,
        time_seconds: 0.0,
        looping: true,
        playing: true,
    };
    let graph_player = AnimationGraphPlayerComponent {
        graph: ResourceHandle::<AnimationGraphMarker>::new(ResourceId::from_stable_label(
            "res://animation/runtime08.graph.zranim",
        )),
        parameters: BTreeMap::new(),
        playing: true,
    };
    let state_machine_player = AnimationStateMachinePlayerComponent {
        state_machine: ResourceHandle::<AnimationStateMachineMarker>::new(
            ResourceId::from_stable_label("res://animation/runtime08.state-machine.zranim"),
        ),
        parameters: BTreeMap::new(),
        active_state: Some("idle".to_string()),
        playing: true,
    };

    world
        .insert(entity, skeleton.clone())
        .expect("animation skeleton should use component storage");
    world
        .insert(entity, player.clone())
        .expect("animation player should use component storage");
    world
        .insert(entity, sequence_player.clone())
        .expect("animation sequence player should use component storage");
    world
        .insert(entity, graph_player.clone())
        .expect("animation graph player should use component storage");
    world
        .insert(entity, state_machine_player.clone())
        .expect("animation state-machine player should use component storage");

    let cloned = world.clone();
    let decoded: World = serde_json::from_str(
        &serde_json::to_string(&world)
            .expect("world persistence must serialize animation runtime storage"),
    )
    .expect("world persistence must restore animation runtime storage");
    for restored in [&cloned, &decoded] {
        assert_eq!(
            restored.get::<AnimationSkeletonComponent>(entity),
            Some(&skeleton)
        );
        assert_eq!(
            restored.get::<AnimationPlayerComponent>(entity),
            Some(&player)
        );
        assert_eq!(
            restored.get::<AnimationSequencePlayerComponent>(entity),
            Some(&sequence_player)
        );
        assert_eq!(
            restored.get::<AnimationGraphPlayerComponent>(entity),
            Some(&graph_player)
        );
        assert_eq!(
            restored.get::<AnimationStateMachinePlayerComponent>(entity),
            Some(&state_machine_player)
        );
    }

    let record = world
        .node_record(entity)
        .expect("animation runtime components must project to a node record");
    world.remove_entity(entity).unwrap();
    world
        .insert_node_record(record)
        .expect("record restore should stage animation runtime before final publish");

    type AnimationData<'query> = (
        crate::scene::EntityId,
        &'query AnimationSkeletonComponent,
        &'query AnimationPlayerComponent,
        &'query AnimationSequencePlayerComponent,
        &'query AnimationGraphPlayerComponent,
        &'query AnimationStateMachinePlayerComponent,
    );
    let rows = world
        .query::<AnimationData<'static>>()
        .iter(&world)
        .map(|(entity, _, _, _, _, _)| entity)
        .collect::<Vec<_>>();
    assert_eq!(rows, vec![entity]);
}

#[test]
fn persistent_animation_runtime_does_not_retain_world_map_owners() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let world_source = std::fs::read_to_string(
        manifest_root
            .join("src")
            .join("scene")
            .join("world")
            .join("world.rs"),
    )
    .expect("read World storage owner source");
    let fixed_components_source = std::fs::read_to_string(
        manifest_root
            .join("src")
            .join("scene")
            .join("world")
            .join("typed_api")
            .join("fixed_components.rs"),
    )
    .expect("read fixed component adapter source");

    for retired_owner in [
        "pub(super) animation_skeletons:",
        "pub(super) animation_players:",
        "pub(super) animation_sequence_players:",
        "pub(super) animation_graph_players:",
        "pub(super) animation_state_machine_players:",
    ] {
        assert!(
            !world_source.contains(retired_owner),
            "World must not retain the retired animation component map owner: {retired_owner}"
        );
    }
    for retired_adapter in [
        "fixed_component_map!(AnimationPlayerComponent, animation_players)",
        "fixed_component_map!(AnimationSequencePlayerComponent, animation_sequence_players)",
        "fixed_component_map!(AnimationGraphPlayerComponent, animation_graph_players)",
        "fixed_component_map!(AnimationStateMachinePlayerComponent, animation_state_machine_players)",
    ] {
        assert!(
            !fixed_components_source.contains(retired_adapter),
            "fixed-component adapter must not reintroduce an animation map owner: {retired_adapter}"
        );
    }
    assert!(
        world_source.contains("persistent_animation_runtime_component_snapshot"),
        "World persistence must project animation runtime values from generic component storage"
    );
}
