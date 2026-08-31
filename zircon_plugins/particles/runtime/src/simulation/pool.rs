use zircon_runtime::core::math::{Real, Vec3, Vec4};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct InitialParticle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub lifetime: Real,
    pub size: Real,
    pub rotation: Real,
    pub angular_velocity: Real,
    pub color: Vec4,
    pub seed: u32,
    pub emitter_index: u32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct CpuParticlePool {
    pub alive: Vec<bool>,
    pub age: Vec<Real>,
    pub lifetime: Vec<Real>,
    pub position: Vec<Vec3>,
    pub previous_position: Vec<Vec3>,
    pub velocity: Vec<Vec3>,
    pub size: Vec<Real>,
    pub initial_size: Vec<Real>,
    pub color: Vec<Vec4>,
    pub start_color: Vec<Vec4>,
    pub rotation: Vec<Real>,
    pub angular_velocity: Vec<Real>,
    pub seed: Vec<u32>,
    pub emitter_index: Vec<u32>,
    pub free: Vec<usize>,
    live_count: usize,
}

impl CpuParticlePool {
    pub(crate) fn clear(&mut self) {
        self.alive.clear();
        self.age.clear();
        self.lifetime.clear();
        self.position.clear();
        self.previous_position.clear();
        self.velocity.clear();
        self.size.clear();
        self.initial_size.clear();
        self.color.clear();
        self.start_color.clear();
        self.rotation.clear();
        self.angular_velocity.clear();
        self.seed.clear();
        self.emitter_index.clear();
        self.free.clear();
        self.live_count = 0;
    }

    pub(crate) fn allocated(&self) -> usize {
        self.alive.len()
    }

    pub(crate) fn live_count(&self) -> usize {
        self.live_count
    }

    pub(crate) fn spawn(&mut self, initial: InitialParticle) {
        if let Some(index) = self.free.pop() {
            self.write(index, initial);
        } else {
            self.alive.push(true);
            self.age.push(0.0);
            self.lifetime.push(initial.lifetime);
            self.position.push(initial.position);
            self.previous_position.push(initial.position);
            self.velocity.push(initial.velocity);
            self.size.push(initial.size);
            self.initial_size.push(initial.size);
            self.color.push(initial.color);
            self.start_color.push(initial.color);
            self.rotation.push(initial.rotation);
            self.angular_velocity.push(initial.angular_velocity);
            self.seed.push(initial.seed);
            self.emitter_index.push(initial.emitter_index);
        }
        self.live_count += 1;
    }

    pub(crate) fn kill(&mut self, index: usize) {
        if self.alive.get(index).copied().unwrap_or(false) {
            self.alive[index] = false;
            self.free.push(index);
            self.live_count -= 1;
        }
    }

    fn write(&mut self, index: usize, initial: InitialParticle) {
        self.alive[index] = true;
        self.age[index] = 0.0;
        self.lifetime[index] = initial.lifetime;
        self.position[index] = initial.position;
        self.previous_position[index] = initial.position;
        self.velocity[index] = initial.velocity;
        self.size[index] = initial.size;
        self.initial_size[index] = initial.size;
        self.color[index] = initial.color;
        self.start_color[index] = initial.color;
        self.rotation[index] = initial.rotation;
        self.angular_velocity[index] = initial.angular_velocity;
        self.seed[index] = initial.seed;
        self.emitter_index[index] = initial.emitter_index;
    }
}

#[cfg(test)]
mod tests {
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
        let source = include_str!("pool.rs")
            .split("\n#[cfg(test)]")
            .next()
            .unwrap_or_default();
        let query = &source[source
            .find("pub(crate) fn live_count")
            .expect("particle pool must expose live_count")..];

        assert!(source.contains("live_count: usize"));
        assert!(!query.contains("self.alive.iter()"));
    }
}
