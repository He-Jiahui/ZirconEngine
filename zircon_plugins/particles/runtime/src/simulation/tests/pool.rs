use super::*;

fn initial_particle(seed: u32) -> InitialParticle {
    InitialParticle {
        position: Vec3::ZERO,
        velocity: Vec3::ZERO,
        lifetime: 1.0,
        size: 1.0,
        rotation: 0.0,
        angular_velocity: 0.0,
        color: Vec4::ONE,
        seed,
        emitter_index: 0,
    }
}

#[test]
fn live_count_is_cached_across_spawn_kill_reuse_and_clear() {
    let mut pool = CpuParticlePool::default();
    pool.spawn(initial_particle(1));
    pool.spawn(initial_particle(2));
    assert_eq!(pool.live_count(), 2);

    pool.kill(0);
    pool.kill(0);
    assert_eq!(pool.live_count(), 1);

    pool.spawn(initial_particle(3));
    assert_eq!(pool.live_count(), 2);
    assert_eq!(pool.allocated(), 2);

    pool.clear();
    assert_eq!(pool.live_count(), 0);
    assert_eq!(pool.allocated(), 0);
}

#[test]
fn live_count_query_does_not_scan_allocated_slots() {
    let source = include_str!("../pool.rs")
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or_default();
    let query = &source[source
        .find("pub(crate) fn live_count")
        .expect("particle pool must expose live_count")..];

    assert!(source.contains("live_count: usize"));
    assert!(!query.contains("self.alive.iter()"));
}
