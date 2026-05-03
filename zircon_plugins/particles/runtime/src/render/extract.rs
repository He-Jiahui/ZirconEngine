use std::collections::BTreeMap;

use zircon_runtime::core::framework::render::{ParticleExtract, RenderParticleBoundsSnapshot};
use zircon_runtime::core::math::{Real, Vec3};

use crate::ParticleRuntimeSnapshot;

pub fn build_particle_extract(
    snapshot: &ParticleRuntimeSnapshot,
    camera_position: Option<Vec3>,
) -> ParticleExtract {
    let mut sprites = snapshot.sprites.clone();
    if let Some(camera_position) = camera_position {
        sprites.sort_by(|a, b| {
            let a_distance = (a.position - camera_position).length_squared();
            let b_distance = (b.position - camera_position).length_squared();
            b_distance.total_cmp(&a_distance)
        });
    }
    let mut emitters = snapshot
        .emitters
        .iter()
        .map(|emitter| emitter.entity)
        .collect::<Vec<_>>();
    emitters.sort_unstable();
    emitters.dedup();
    let bounds = build_bounds(&sprites);
    ParticleExtract {
        emitters,
        sprites,
        bounds,
        sort_camera_position: camera_position,
    }
}

fn build_bounds(
    sprites: &[zircon_runtime::core::framework::render::RenderParticleSpriteSnapshot],
) -> Vec<RenderParticleBoundsSnapshot> {
    let mut ranges: BTreeMap<_, (Vec3, Vec3)> = BTreeMap::new();
    for sprite in sprites {
        let half = Vec3::splat(sprite.size.max(0.0) * 0.5);
        let min = sprite.position - half;
        let max = sprite.position + half;
        ranges
            .entry(sprite.entity)
            .and_modify(|(current_min, current_max)| {
                *current_min = current_min.min(min);
                *current_max = current_max.max(max);
            })
            .or_insert((min, max));
    }

    ranges
        .into_iter()
        .map(|(entity, (min, max))| {
            let center = (min + max) * 0.5;
            let radius: Real = (max - center).length();
            RenderParticleBoundsSnapshot {
                entity,
                center,
                radius,
            }
        })
        .collect()
}
