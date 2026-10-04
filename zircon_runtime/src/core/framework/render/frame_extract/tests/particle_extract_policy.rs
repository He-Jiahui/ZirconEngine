use crate::core::math::{Vec2, Vec3};

use super::*;
use crate::core::framework::render::RenderParticlePreviousSpriteSnapshot;

#[test]
fn particle_extract_counts_previous_state_by_entity() {
    let mut extract = ParticleExtract::default();
    extract.sprites = vec![
        RenderParticleSpriteSnapshot {
            entity: 7,
            ..RenderParticleSpriteSnapshot::default()
        },
        RenderParticleSpriteSnapshot {
            entity: 9,
            ..RenderParticleSpriteSnapshot::default()
        },
    ];
    extract.previous_sprites = vec![previous_sprite(9, 0)];

    assert_eq!(extract.previous_state_sprite_count(), 1);
    assert_eq!(extract.missing_previous_state_sprite_count(), 1);
}

#[test]
fn particle_extract_rejects_ambiguous_anonymous_previous_state() {
    let mut extract = ParticleExtract::default();
    extract.sprites = vec![
        RenderParticleSpriteSnapshot {
            entity: 9,
            ..RenderParticleSpriteSnapshot::default()
        },
        RenderParticleSpriteSnapshot {
            entity: 9,
            ..RenderParticleSpriteSnapshot::default()
        },
    ];
    extract.previous_sprites = vec![previous_sprite(9, 0)];

    assert_eq!(extract.previous_state_sprite_count(), 0);
    assert_eq!(extract.missing_previous_state_sprite_count(), 2);

    extract.previous_sprites.push(previous_sprite(9, 0));

    assert_eq!(extract.previous_state_sprite_count(), 0);
    assert_eq!(extract.missing_previous_state_sprite_count(), 2);
}

#[test]
fn particle_extract_matches_duplicate_entity_previous_state_by_stable_sprite_key() {
    let mut extract = ParticleExtract::default();
    extract.sprites = vec![
        RenderParticleSpriteSnapshot {
            entity: 9,
            stable_sprite_key: 11,
            ..RenderParticleSpriteSnapshot::default()
        },
        RenderParticleSpriteSnapshot {
            entity: 9,
            stable_sprite_key: 12,
            ..RenderParticleSpriteSnapshot::default()
        },
    ];
    extract.previous_sprites = vec![previous_sprite(9, 12)];

    assert_eq!(extract.previous_state_sprite_count(), 1);
    assert_eq!(extract.missing_previous_state_sprite_count(), 1);
}

#[test]
fn particle_extract_reports_anonymous_stream_ambiguity_for_duplicate_key_zero_sprites() {
    let mut extract = ParticleExtract::default();
    extract.sprites = vec![
        RenderParticleSpriteSnapshot {
            entity: 9,
            stable_sprite_key: 0,
            ..RenderParticleSpriteSnapshot::default()
        },
        RenderParticleSpriteSnapshot {
            entity: 9,
            stable_sprite_key: 0,
            ..RenderParticleSpriteSnapshot::default()
        },
        RenderParticleSpriteSnapshot {
            entity: 9,
            stable_sprite_key: 7,
            ..RenderParticleSpriteSnapshot::default()
        },
        RenderParticleSpriteSnapshot {
            entity: 10,
            stable_sprite_key: 0,
            ..RenderParticleSpriteSnapshot::default()
        },
    ];

    assert_eq!(extract.anonymous_stream_ambiguity_sprite_count(), 2);
    assert_eq!(
        extract.anonymous_stream_ambiguity_entities(),
        HashSet::from([9])
    );
}

fn previous_sprite(
    entity: EntityId,
    stable_sprite_key: u64,
) -> RenderParticlePreviousSpriteSnapshot {
    RenderParticlePreviousSpriteSnapshot {
        entity,
        stable_sprite_key,
        position: Vec3::new(1.0, 2.0, 3.0),
        size: 1.0,
        aspect_ratio: 1.0,
        billboard_offset: Vec2::ZERO,
        rotation: 0.0,
        billboard_basis: None,
    }
}
