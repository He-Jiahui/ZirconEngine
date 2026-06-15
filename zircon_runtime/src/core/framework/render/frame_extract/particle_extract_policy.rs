use std::collections::{BTreeMap, BTreeSet};

use crate::core::framework::scene::EntityId;

use super::{ParticleExtract, RenderParticleSpriteSnapshot};

impl ParticleExtract {
    pub fn previous_state_sprite_count(&self) -> usize {
        if self.sprites.is_empty() || self.previous_sprites.is_empty() {
            return 0;
        }

        let ambiguous_anonymous_entities = self.anonymous_stream_ambiguity_entities();
        let mut remaining_previous_by_identity = BTreeMap::new();
        for sprite in &self.previous_sprites {
            if is_ambiguous_anonymous_identity(
                sprite.entity,
                sprite.stable_sprite_key,
                &ambiguous_anonymous_entities,
            ) {
                continue;
            }
            *remaining_previous_by_identity
                .entry(sprite.identity())
                .or_insert(0usize) += 1;
        }

        let mut matched = 0;
        for sprite in &self.sprites {
            if is_ambiguous_anonymous_identity(
                sprite.entity,
                sprite.stable_sprite_key,
                &ambiguous_anonymous_entities,
            ) {
                continue;
            }
            if let Some(remaining) = remaining_previous_by_identity.get_mut(&sprite.identity()) {
                if *remaining > 0 {
                    *remaining -= 1;
                    matched += 1;
                }
            }
        }
        matched
    }

    pub fn missing_previous_state_sprite_count(&self) -> usize {
        self.sprites
            .len()
            .saturating_sub(self.previous_state_sprite_count())
    }

    pub fn anonymous_stream_ambiguity_sprite_count(&self) -> usize {
        anonymous_sprite_count_by_entity(&self.sprites)
            .into_values()
            .filter(|count| *count > 1)
            .sum()
    }

    pub(crate) fn anonymous_stream_ambiguity_entities(&self) -> BTreeSet<EntityId> {
        anonymous_sprite_count_by_entity(&self.sprites)
            .into_iter()
            .filter_map(|(entity, count)| (count > 1).then_some(entity))
            .collect()
    }
}

pub(crate) fn is_ambiguous_anonymous_identity(
    entity: EntityId,
    stable_sprite_key: u64,
    ambiguous_anonymous_entities: &BTreeSet<EntityId>,
) -> bool {
    stable_sprite_key == 0 && ambiguous_anonymous_entities.contains(&entity)
}

fn anonymous_sprite_count_by_entity(
    sprites: &[RenderParticleSpriteSnapshot],
) -> BTreeMap<EntityId, usize> {
    let mut anonymous_sprite_count_by_entity = BTreeMap::new();
    for sprite in sprites {
        if sprite.stable_sprite_key == 0 {
            *anonymous_sprite_count_by_entity
                .entry(sprite.entity)
                .or_insert(0usize) += 1;
        }
    }
    anonymous_sprite_count_by_entity
}

#[cfg(test)]
mod tests {
    use crate::core::math::{Vec2, Vec3};

    use super::{super::RenderParticlePreviousSpriteSnapshot, *};

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
            BTreeSet::from([9])
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
}
