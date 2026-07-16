use super::super::*;
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::framework::sound::{
    SoundGameplayEmitter, SOUND_GAMEPLAY_EMISSION_CAPACITY,
};

const WORLD_A: WorldHandle = WorldHandle::new(11);
const WORLD_B: WorldHandle = WorldHandle::new(12);

#[test]
fn spatial_source_creation_produces_bounded_gameplay_emission() {
    let sound = DefaultSoundManager::default();
    let first_cursor = sound
        .read_gameplay_emissions(WORLD_A, None)
        .unwrap()
        .next_sequence;
    let second_cursor = sound
        .read_gameplay_emissions(WORLD_A, None)
        .unwrap()
        .next_sequence;
    let clip = sound.insert_clip_for_test(test_clip("res://sound/guard-step.wav", &[0.25]));
    let mut source = SoundSourceDescriptor::clip(clip);
    source.position = [3.0, 1.0, -2.0];
    source.gain = 0.6;
    source.spatial.max_distance = 18.0;
    source.gameplay_emitter = Some(SoundGameplayEmitter {
        world: WORLD_A,
        entity: 91,
    });

    sound.create_source(source).unwrap();

    let first = sound
        .read_gameplay_emissions(WORLD_A, Some(first_cursor))
        .unwrap();
    let second = sound
        .read_gameplay_emissions(WORLD_A, Some(second_cursor))
        .unwrap();
    assert_eq!(first.events, second.events);
    assert_eq!(first.events.len(), 1);
    assert_eq!(first.events[0].world, WORLD_A);
    assert_eq!(first.events[0].source, 91);
    assert_eq!(first.events[0].position, [3.0, 1.0, -2.0]);
    assert_eq!(first.events[0].strength, 0.6);
    assert_eq!(first.events[0].max_range, Some(18.0));
    let other_world = sound
        .read_gameplay_emissions(WORLD_B, Some(first_cursor))
        .unwrap();
    assert!(other_world.events.is_empty());
    assert_eq!(other_world.next_sequence, 0);
}

#[test]
fn non_gameplay_and_muted_sources_do_not_produce_emissions() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/ui-click.wav", &[0.25]));
    sound
        .create_source(SoundSourceDescriptor::clip(clip))
        .unwrap();
    let mut muted = SoundSourceDescriptor::clip(clip);
    muted.gameplay_emitter = Some(SoundGameplayEmitter {
        world: WORLD_A,
        entity: 7,
    });
    muted.muted = true;
    sound.create_source(muted).unwrap();

    assert!(sound
        .read_gameplay_emissions(WORLD_A, Some(0))
        .unwrap()
        .events
        .is_empty());
}

#[test]
fn gameplay_emission_journal_reports_overwritten_history_without_consuming_other_readers() {
    let sound = DefaultSoundManager::default();
    let cursor = sound
        .read_gameplay_emissions(WORLD_A, None)
        .unwrap()
        .next_sequence;
    let clip = sound.insert_clip_for_test(test_clip("res://sound/patrol-step.wav", &[0.25]));
    let emission_count = SOUND_GAMEPLAY_EMISSION_CAPACITY + 2;
    for entity in 1..=emission_count as u64 {
        let mut source = SoundSourceDescriptor::clip(clip);
        source.gameplay_emitter = Some(SoundGameplayEmitter {
            world: WORLD_A,
            entity,
        });
        sound.create_source(source).unwrap();
    }

    let first_reader = sound
        .read_gameplay_emissions(WORLD_A, Some(cursor))
        .unwrap();
    let second_reader = sound
        .read_gameplay_emissions(WORLD_A, Some(cursor))
        .unwrap();

    assert_eq!(first_reader, second_reader);
    assert_eq!(first_reader.events.len(), SOUND_GAMEPLAY_EMISSION_CAPACITY);
    assert_eq!(first_reader.missed_events, 2);
    assert_eq!(first_reader.events.first().unwrap().source, 3);
    assert_eq!(
        first_reader.events.last().unwrap().source,
        emission_count as u64
    );
}

#[test]
fn gameplay_emission_capacity_and_coverage_are_isolated_per_world() {
    let sound = DefaultSoundManager::default();
    let world_a_cursor = sound
        .read_gameplay_emissions(WORLD_A, None)
        .unwrap()
        .next_sequence;
    let clip = sound.insert_clip_for_test(test_clip("res://sound/world-step.wav", &[0.25]));
    let mut world_a_source = SoundSourceDescriptor::clip(clip);
    world_a_source.gameplay_emitter = Some(SoundGameplayEmitter {
        world: WORLD_A,
        entity: 900,
    });
    sound.create_source(world_a_source).unwrap();

    for entity in 1..=(SOUND_GAMEPLAY_EMISSION_CAPACITY as u64 + 2) {
        let mut source = SoundSourceDescriptor::clip(clip);
        source.gameplay_emitter = Some(SoundGameplayEmitter {
            world: WORLD_B,
            entity,
        });
        sound.create_source(source).unwrap();
    }

    let world_a = sound
        .read_gameplay_emissions(WORLD_A, Some(world_a_cursor))
        .unwrap();
    let world_b = sound.read_gameplay_emissions(WORLD_B, Some(0)).unwrap();

    assert_eq!(world_a.missed_events, 0);
    assert_eq!(world_a.events.len(), 1);
    assert_eq!(world_a.events[0].source, 900);
    assert_eq!(world_b.missed_events, 2);
    assert_eq!(world_b.events.len(), SOUND_GAMEPLAY_EMISSION_CAPACITY);
}

#[test]
fn gameplay_emission_read_recovers_after_state_lock_poisoning() {
    let sound = DefaultSoundManager::default();
    sound.poison_state_for_test();

    let read = sound
        .read_gameplay_emissions(WORLD_A, Some(0))
        .expect("poisoned state lock is recovered");

    assert!(read.events.is_empty());
    assert_eq!(read.next_sequence, 0);
    assert_eq!(read.missed_events, 0);
}
