use std::collections::{HashMap, HashSet};

use crate::core::framework::scene::EntityId;

use super::super::RenderParticleSpriteSnapshot;
use super::ParticleExtract;

#[cfg(test)]
#[path = "particle_extract_policy/tests/hash_identity_tests.rs"]
mod hash_identity_tests;

// 粒子上一帧匹配按实体与稳定 sprite key 计数；匿名重复实体会被排除，避免把一个流的速度误配给另一个流。
// 这些计数随后进入帧统计和速度更新路径，反映匹配状态而不是 GPU 执行结果。
impl ParticleExtract {
    pub fn previous_state_sprite_count(&self) -> usize {
        self.previous_state_sprite_count_with(&self.previous_sprites)
    }

    pub fn previous_state_sprite_count_with(
        &self,
        previous_sprites: &[super::super::RenderParticlePreviousSpriteSnapshot],
    ) -> usize {
        if self.sprites.is_empty() || previous_sprites.is_empty() {
            return 0;
        }

        let ambiguous_anonymous_entities = self.anonymous_stream_ambiguity_entities();
        let mut remaining_previous_by_identity = HashMap::with_capacity(previous_sprites.len());
        for sprite in previous_sprites {
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

    pub(crate) fn anonymous_stream_ambiguity_entities(&self) -> HashSet<EntityId> {
        anonymous_sprite_count_by_entity(&self.sprites)
            .into_iter()
            .filter_map(|(entity, count)| (count > 1).then_some(entity))
            .collect()
    }
}

pub(crate) fn is_ambiguous_anonymous_identity(
    entity: EntityId,
    stable_sprite_key: u64,
    ambiguous_anonymous_entities: &HashSet<EntityId>,
) -> bool {
    stable_sprite_key == 0 && ambiguous_anonymous_entities.contains(&entity)
}

fn anonymous_sprite_count_by_entity(
    sprites: &[RenderParticleSpriteSnapshot],
) -> HashMap<EntityId, usize> {
    let mut anonymous_sprite_count_by_entity = HashMap::with_capacity(sprites.len());
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
#[path = "tests/particle_extract_policy.rs"]
mod tests;
