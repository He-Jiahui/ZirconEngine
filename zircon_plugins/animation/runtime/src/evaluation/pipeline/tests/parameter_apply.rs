use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime::core::framework::animation::AnimationPlaybackSettings;
use zircon_runtime::core::resource::{
    AnimationClipMarker, AnimationSequenceMarker, AnimationSkeletonMarker, ResourceHandle,
    ResourceId,
};
use zircon_runtime::scene::components::{
    AnimationPlayerComponent, AnimationSequencePlayerComponent, AnimationSkeletonComponent,
};
use zircon_runtime::scene::{NodeKind, World};

use super::AnimationEvaluationProjection;

#[test]
fn clip_player_time_is_deferred_until_event_batch_admission() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Cube);
    world
        .set_animation_player(
            entity,
            Some(AnimationPlayerComponent {
                clip: ResourceHandle::<AnimationClipMarker>::new(ResourceId::from_stable_label(
                    "animation.clip.deferred-time",
                )),
                playback_speed: 1.0,
                time_seconds: 0.25,
                weight: 1.0,
                looping: false,
                playing: true,
            }),
        )
        .expect("animation player is installed");
    let mut projection = AnimationEvaluationProjection::default();

    let transaction = projection.scan(
        &mut world,
        &AnimationPlaybackSettings::default(),
        None,
        0.5,
        &BTreeMap::new(),
        &BTreeMap::new(),
        &BTreeMap::new(),
    );

    assert_eq!(world.animation_player(entity).unwrap().time_seconds, 0.25);
    assert_eq!(transaction.clip_player_updates.len(), 1);
    assert_eq!(transaction.clip_player_updates[0].1.time_seconds, 0.75);
    assert_eq!(transaction.scan.clip_event_samples.len(), 1);
    assert_eq!(
        transaction.scan.clip_event_samples[0].from_time_seconds,
        0.25
    );
    assert_eq!(transaction.scan.clip_event_samples[0].to_time_seconds, 0.75);
}

#[test]
fn negative_speed_looping_player_submits_a_descending_unwrapped_range() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Cube);
    world
        .set_animation_player(
            entity,
            Some(AnimationPlayerComponent {
                clip: ResourceHandle::<AnimationClipMarker>::new(ResourceId::from_stable_label(
                    "animation.clip.reverse-loop-range",
                )),
                playback_speed: -1.0,
                time_seconds: 2.25,
                weight: 1.0,
                looping: true,
                playing: true,
            }),
        )
        .expect("animation player is installed");
    let mut projection = AnimationEvaluationProjection::default();

    let transaction = projection.scan(
        &mut world,
        &AnimationPlaybackSettings::default(),
        None,
        0.5,
        &BTreeMap::new(),
        &BTreeMap::new(),
        &BTreeMap::new(),
    );

    assert_eq!(transaction.clip_player_updates.len(), 1);
    assert_eq!(transaction.clip_player_updates[0].1.time_seconds, 1.75);
    assert_eq!(transaction.scan.clip_event_samples.len(), 1);
    assert_eq!(
        transaction.scan.clip_event_samples[0].from_time_seconds,
        2.25
    );
    assert_eq!(transaction.scan.clip_event_samples[0].to_time_seconds, 1.75);
    assert!(transaction.scan.clip_event_samples[0].looping);
}

#[test]
fn deferred_entity_restores_projection_revision_for_retry() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Cube);
    world
        .set_animation_skeleton(
            entity,
            Some(AnimationSkeletonComponent {
                skeleton: ResourceHandle::<AnimationSkeletonMarker>::new(
                    ResourceId::from_stable_label("animation.skeleton.deferred-revision"),
                ),
            }),
        )
        .expect("animation skeleton is installed");
    world
        .set_animation_player(
            entity,
            Some(AnimationPlayerComponent {
                clip: ResourceHandle::<AnimationClipMarker>::new(ResourceId::from_stable_label(
                    "animation.clip.deferred-revision",
                )),
                playback_speed: 1.0,
                time_seconds: 0.25,
                weight: 1.0,
                looping: false,
                playing: false,
            }),
        )
        .expect("paused animation player is installed");
    let mut projection = AnimationEvaluationProjection::default();
    let first = projection.scan(
        &mut world,
        &AnimationPlaybackSettings::default(),
        None,
        0.5,
        &BTreeMap::new(),
        &BTreeMap::new(),
        &BTreeMap::new(),
    );
    assert_eq!(first.scan.clip_samples.len(), 1);
    projection.commit_revision_stage(first.revision_stage, &BTreeSet::from([entity]));

    let retry = projection.scan(
        &mut world,
        &AnimationPlaybackSettings::default(),
        None,
        0.5,
        &BTreeMap::new(),
        &BTreeMap::new(),
        &BTreeMap::new(),
    );

    assert_eq!(retry.scan.clip_samples.len(), 1);
    projection.commit_revision_stage(retry.revision_stage, &BTreeSet::new());

    let committed = projection.scan(
        &mut world,
        &AnimationPlaybackSettings::default(),
        None,
        0.5,
        &BTreeMap::new(),
        &BTreeMap::new(),
        &BTreeMap::new(),
    );
    assert!(committed.scan.clip_samples.is_empty());
}

#[test]
fn sequence_time_and_revision_are_staged_until_owner_admission() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Cube);
    world
        .set_animation_sequence_player(
            entity,
            Some(AnimationSequencePlayerComponent {
                sequence: ResourceHandle::<AnimationSequenceMarker>::new(
                    ResourceId::from_stable_label("animation.sequence.deferred"),
                ),
                playback_speed: 1.0,
                time_seconds: 0.25,
                looping: false,
                playing: true,
            }),
        )
        .expect("animation sequence player is installed");
    let mut projection = AnimationEvaluationProjection::default();

    let first = projection.scan(
        &mut world,
        &AnimationPlaybackSettings::default(),
        None,
        0.5,
        &BTreeMap::new(),
        &BTreeMap::new(),
        &BTreeMap::new(),
    );

    assert_eq!(
        world
            .animation_sequence_player(entity)
            .expect("sequence player remains installed")
            .time_seconds,
        0.25
    );
    assert_eq!(first.sequence_player_updates.len(), 1);
    assert_eq!(first.sequence_player_updates[0].1.time_seconds, 0.75);
    assert_eq!(first.scan.sequences.len(), 1);
    assert_eq!(first.scan.sequences[0].entity, entity);
    projection.commit_revision_stage(first.revision_stage, &BTreeSet::from([entity]));

    let retry = projection.scan(
        &mut world,
        &AnimationPlaybackSettings::default(),
        None,
        0.5,
        &BTreeMap::new(),
        &BTreeMap::new(),
        &BTreeMap::new(),
    );
    assert_eq!(retry.scan.sequences.len(), 1);
}
