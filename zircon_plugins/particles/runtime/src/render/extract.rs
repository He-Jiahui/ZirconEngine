use std::collections::HashMap;

use zircon_runtime::core::framework::render::{
    ParticleExtract, RenderParticleBoundsSnapshot, RenderParticleGpuFrameExtract,
};
use zircon_runtime::core::math::{Real, Vec3};

use crate::ParticleRuntimeSnapshot;

#[cfg(test)]
#[path = "extract/performance_tests.rs"]
mod performance_tests;

pub fn build_particle_extract(
    snapshot: &ParticleRuntimeSnapshot,
    camera_position: Option<Vec3>,
) -> ParticleExtract {
    let mut sprites = snapshot.sprites.as_ref().to_vec();
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
    let bounds = build_bounds(&sprites, snapshot.emitters.len());
    ParticleExtract {
        emitters,
        sprites,
        previous_sprites: Vec::new(),
        bounds,
        sort_camera_position: camera_position,
        gpu_frame: build_gpu_frame(snapshot),
    }
}

fn build_gpu_frame(snapshot: &ParticleRuntimeSnapshot) -> Option<RenderParticleGpuFrameExtract> {
    let mut alive_count = 0u32;
    let mut per_emitter_spawned = Vec::with_capacity(snapshot.emitters.len());
    for emitter in &snapshot.emitters {
        if emitter.backend != crate::ParticleSimulationBackend::Gpu {
            continue;
        }
        let live_particles = emitter.live_particles as u32;
        alive_count += live_particles;
        per_emitter_spawned.push(live_particles);
    }
    if per_emitter_spawned.is_empty() {
        return None;
    }
    Some(RenderParticleGpuFrameExtract {
        alive_count,
        spawned_total: alive_count,
        per_emitter_spawned,
        indirect_draw_args: [6, alive_count, 0, 0],
    })
}

fn build_bounds(
    sprites: &[zircon_runtime::core::framework::render::RenderParticleSpriteSnapshot],
    emitter_capacity_hint: usize,
) -> Vec<RenderParticleBoundsSnapshot> {
    let mut ranges: HashMap<_, (Vec3, Vec3)> =
        HashMap::with_capacity(emitter_capacity_hint.min(sprites.len()));
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

    let mut bounds = ranges
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
        .collect::<Vec<_>>();
    bounds.sort_unstable_by_key(|bounds| bounds.entity);
    bounds
}
