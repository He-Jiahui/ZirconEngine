use zircon_runtime::core::framework::render::ParticleExtract;
use zircon_runtime::core::math::Vec3;

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
    ParticleExtract { emitters, sprites }
}
