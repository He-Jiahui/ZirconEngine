use zircon_runtime::core::framework::render::RenderParticleSpriteSnapshot;
use zircon_runtime::core::framework::scene::EntityId;
use zircon_runtime::core::math::{is_finite_vec3, is_finite_vec4, Real, Transform, Vec3};

use crate::asset::{evaluate_color_curve, evaluate_scalar_curve};
use crate::component::{ParticleEmitterHandle, ParticleSystemComponent};
use crate::{
    ParticleColorKey, ParticleCoordinateSpace, ParticleEmitterAsset, ParticleScalarKey,
    ParticleShape, ParticleSimulationBackend,
};

use super::pool::{CpuParticlePool, InitialParticle};
use super::{ParticleRng, ParticleSimulationError};

const SPAWN_ACCUMULATOR_EPSILON: Real = 1.0e-5;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ParticleEmitterRuntimeState {
    pub emitter_id: String,
    pub live_particles: usize,
    pub allocated_particles: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ParticleSystemInstance {
    pub handle: ParticleEmitterHandle,
    pub component: ParticleSystemComponent,
    pub playing: bool,
    pub age_seconds: Real,
    pub fallback_to_cpu: bool,
    physics_enabled: bool,
    emitters: Vec<ParticleEmitterInstance>,
}

impl ParticleSystemInstance {
    pub(crate) fn new(
        handle: ParticleEmitterHandle,
        component: ParticleSystemComponent,
        fallback_to_cpu: bool,
        physics_enabled: bool,
    ) -> Result<Self, ParticleSimulationError> {
        validate_component(&component)?;
        let seed = component.asset.seed ^ handle.raw();
        let emitters = component
            .asset
            .emitters
            .iter()
            .enumerate()
            .map(|(index, emitter)| {
                ParticleEmitterInstance::new(index as u32, emitter.clone(), seed, physics_enabled)
            })
            .collect();
        Ok(Self {
            handle,
            playing: component.playing,
            component,
            age_seconds: 0.0,
            fallback_to_cpu,
            physics_enabled,
            emitters,
        })
    }

    pub(crate) fn play(&mut self) {
        self.playing = true;
    }

    pub(crate) fn pause(&mut self) {
        self.playing = false;
    }

    pub(crate) fn stop(&mut self) {
        self.playing = false;
        self.reset_particles();
    }

    pub(crate) fn reset_particles(&mut self) {
        self.age_seconds = 0.0;
        for emitter in &mut self.emitters {
            emitter.reset(self.component.asset.seed ^ self.handle.raw());
        }
    }

    pub(crate) fn tick(&mut self, dt: Real) -> Result<(), ParticleSimulationError> {
        if !dt.is_finite() || dt < 0.0 {
            return Err(ParticleSimulationError::InvalidDeltaTime);
        }
        if !self.playing || dt <= Real::EPSILON {
            return Ok(());
        }
        let scaled_dt = dt * self.component.time_scale.max(0.0);
        let previous_age = self.age_seconds;
        self.age_seconds += scaled_dt;
        let transform = self.component.transform;
        let entity = self.component.entity;
        for emitter in &mut self.emitters {
            emitter.spawn_due_particles(previous_age, self.age_seconds, transform, entity);
            emitter.update_particles(scaled_dt);
        }
        Ok(())
    }

    pub(crate) fn sprites(&self) -> Vec<RenderParticleSpriteSnapshot> {
        let mut sprites = Vec::new();
        for emitter in &self.emitters {
            emitter.append_sprites(
                self.component.entity,
                self.component.transform,
                &mut sprites,
            );
        }
        sprites
    }

    pub(crate) fn emitter_states(&self) -> Vec<ParticleEmitterRuntimeState> {
        self.emitters
            .iter()
            .map(|emitter| ParticleEmitterRuntimeState {
                emitter_id: emitter.asset.id.clone(),
                live_particles: emitter.pool.live_count(),
                allocated_particles: emitter.pool.allocated(),
            })
            .collect()
    }

    pub(crate) fn backend(&self) -> ParticleSimulationBackend {
        self.component.asset.backend
    }

    pub(crate) fn entity(&self) -> EntityId {
        self.component.entity
    }

    pub(crate) fn requires_physics(&self) -> bool {
        self.emitters
            .iter()
            .any(|emitter| emitter.asset.physics.is_enabled())
    }

    pub(crate) fn set_physics_enabled(&mut self, physics_enabled: bool) {
        self.physics_enabled = physics_enabled;
        for emitter in &mut self.emitters {
            emitter.physics_enabled = physics_enabled;
        }
    }

    pub(crate) fn requires_animation(&self) -> bool {
        self.emitters
            .iter()
            .any(|emitter| !emitter.asset.animation_bindings.is_empty())
    }

    pub(crate) fn trigger_burst_now(&mut self) {
        let transform = self.component.transform;
        let entity = self.component.entity;
        for emitter in &mut self.emitters {
            emitter.spawn_explicit(1, transform, entity);
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ParticleEmitterInstance {
    emitter_index: u32,
    asset: ParticleEmitterAsset,
    pool: CpuParticlePool,
    spawn_accumulator: Real,
    next_burst_index: usize,
    rng: ParticleRng,
    physics_enabled: bool,
}

impl ParticleEmitterInstance {
    fn new(
        emitter_index: u32,
        mut asset: ParticleEmitterAsset,
        system_seed: u64,
        physics_enabled: bool,
    ) -> Self {
        asset
            .bursts
            .sort_by(|a, b| a.time_seconds.max(0.0).total_cmp(&b.time_seconds.max(0.0)));
        let rng_seed =
            system_seed ^ ((emitter_index as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        Self {
            emitter_index,
            asset,
            pool: CpuParticlePool::default(),
            spawn_accumulator: 0.0,
            next_burst_index: 0,
            rng: ParticleRng::new(rng_seed),
            physics_enabled,
        }
    }

    fn reset(&mut self, system_seed: u64) {
        self.pool.clear();
        self.spawn_accumulator = 0.0;
        self.next_burst_index = 0;
        let rng_seed =
            system_seed ^ ((self.emitter_index as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        self.rng = ParticleRng::new(rng_seed);
    }

    fn spawn_due_particles(
        &mut self,
        previous_age: Real,
        current_age: Real,
        transform: Transform,
        entity: EntityId,
    ) {
        let mut spawn_count = 0u32;
        while let Some(burst) = self.asset.bursts.get(self.next_burst_index).copied() {
            let burst_time = burst.time_seconds.max(0.0);
            if burst_time > current_age {
                break;
            }
            if burst_time >= previous_age {
                spawn_count = spawn_count.saturating_add(burst.count);
            }
            self.next_burst_index += 1;
        }

        if self.asset.spawn_rate_per_second > 0.0 {
            self.spawn_accumulator +=
                (current_age - previous_age) * self.asset.spawn_rate_per_second;
            let continuous = (self.spawn_accumulator + SPAWN_ACCUMULATOR_EPSILON).floor() as u32;
            self.spawn_accumulator = (self.spawn_accumulator - continuous as Real).max(0.0);
            spawn_count = spawn_count.saturating_add(continuous);
        }

        let live = self.pool.live_count() as u32;
        let available = self.asset.max_particles.saturating_sub(live);
        for _ in 0..spawn_count.min(available) {
            let initial = self.initial_particle(transform, entity);
            self.pool.spawn(initial);
        }
    }

    fn spawn_explicit(&mut self, count: u32, transform: Transform, entity: EntityId) {
        let live = self.pool.live_count() as u32;
        let available = self.asset.max_particles.saturating_sub(live);
        for _ in 0..count.min(available) {
            let initial = self.initial_particle(transform, entity);
            self.pool.spawn(initial);
        }
    }

    fn initial_particle(&mut self, transform: Transform, _entity: EntityId) -> InitialParticle {
        let local_position = self.asset.shape.sample_position(&mut self.rng);
        let position = match self.asset.coordinate_space {
            ParticleCoordinateSpace::Local => local_position,
            ParticleCoordinateSpace::World => transform.matrix().transform_point3(local_position),
        };
        let lifetime = self
            .asset
            .lifetime
            .normalized()
            .sample(&mut self.rng)
            .max(Real::EPSILON);
        let size = self
            .asset
            .initial_size
            .normalized()
            .sample(&mut self.rng)
            .max(0.0);
        InitialParticle {
            position,
            velocity: self.asset.initial_velocity.sample(&mut self.rng),
            lifetime,
            size,
            rotation: self.asset.initial_rotation.sample(&mut self.rng),
            angular_velocity: self.asset.initial_angular_velocity.sample(&mut self.rng),
            color: self.asset.start_color,
            seed: self.rng.next_u32(),
            emitter_index: self.emitter_index,
        }
    }

    fn update_particles(&mut self, dt: Real) {
        for index in 0..self.pool.alive.len() {
            if !self.pool.alive[index] {
                continue;
            }
            self.pool.age[index] += dt;
            if self.pool.age[index] >= self.pool.lifetime[index] {
                self.pool.kill(index);
                continue;
            }
            let normalized_age = (self.pool.age[index] / self.pool.lifetime[index]).clamp(0.0, 1.0);
            self.pool.previous_position[index] = self.pool.position[index];
            let external_force = if self.physics_enabled {
                self.asset.physics.external_force
            } else {
                Vec3::ZERO
            };
            self.pool.velocity[index] += (self.asset.gravity + external_force) * dt;
            let drag_factor = (1.0 - self.asset.drag.max(0.0) * dt).clamp(0.0, 1.0);
            let damping_factor = if self.physics_enabled && self.asset.physics.collision_enabled {
                1.0 - self.asset.physics.damping.clamp(0.0, 1.0)
            } else {
                1.0
            };
            self.pool.velocity[index] *= drag_factor * damping_factor;
            self.pool.position[index] += self.pool.velocity[index] * dt;
            self.pool.rotation[index] += self.pool.angular_velocity[index] * dt;
            self.pool.size[index] = self.pool.initial_size[index]
                * evaluate_scalar_curve(&self.asset.size_over_lifetime, normalized_age).max(0.0);
            self.pool.color[index] = self.pool.start_color[index]
                * evaluate_color_curve(&self.asset.color_over_lifetime, normalized_age);
        }
    }

    fn append_sprites(
        &self,
        entity: EntityId,
        transform: Transform,
        sprites: &mut Vec<RenderParticleSpriteSnapshot>,
    ) {
        for index in 0..self.pool.alive.len() {
            if !self.pool.alive[index] {
                continue;
            }
            let position = match self.asset.coordinate_space {
                ParticleCoordinateSpace::Local => transform
                    .matrix()
                    .transform_point3(self.pool.position[index]),
                ParticleCoordinateSpace::World => self.pool.position[index],
            };
            sprites.push(RenderParticleSpriteSnapshot {
                entity,
                position,
                size: self.pool.size[index],
                rotation: self.pool.rotation[index],
                color: self.pool.color[index],
                intensity: 1.0,
                material: self.asset.material,
                texture: self.asset.texture,
            });
        }
    }
}

fn validate_component(component: &ParticleSystemComponent) -> Result<(), ParticleSimulationError> {
    if component.asset.emitters.is_empty() {
        return Err(ParticleSimulationError::InvalidAsset(
            "particle system must contain at least one emitter".to_string(),
        ));
    }
    if !component.time_scale.is_finite() {
        return Err(ParticleSimulationError::InvalidAsset(
            "time scale must be finite".to_string(),
        ));
    }
    for emitter in &component.asset.emitters {
        if emitter.max_particles == 0 {
            continue;
        }
        if !finite_scalar_range(emitter.lifetime)
            || !finite_scalar_range(emitter.initial_size)
            || !finite_scalar_range(emitter.initial_rotation)
            || !finite_scalar_range(emitter.initial_angular_velocity)
            || !emitter.spawn_rate_per_second.is_finite()
            || !emitter.drag.is_finite()
            || !emitter.physics.bounce.is_finite()
            || !emitter.physics.damping.is_finite()
        {
            return Err(ParticleSimulationError::InvalidAsset(format!(
                "emitter {} contains non-finite scalar settings",
                emitter.id
            )));
        }
        if !finite_vec3_range(emitter.initial_velocity)
            || !is_finite_vec3(emitter.gravity)
            || !is_finite_vec3(emitter.physics.external_force)
            || !is_finite_vec4(emitter.start_color)
        {
            return Err(ParticleSimulationError::InvalidAsset(format!(
                "emitter {} contains non-finite vector settings",
                emitter.id
            )));
        }
        validate_shape(emitter.id.as_str(), emitter.shape)?;
        validate_bursts(emitter.id.as_str(), &emitter.bursts)?;
        validate_animation_bindings(emitter.id.as_str(), &emitter.animation_bindings)?;
        validate_scalar_keys(emitter.id.as_str(), &emitter.size_over_lifetime)?;
        validate_color_keys(emitter.id.as_str(), &emitter.color_over_lifetime)?;
    }
    Ok(())
}

fn finite_scalar_range(range: crate::ParticleScalarRange) -> bool {
    range.min.is_finite() && range.max.is_finite()
}

fn finite_vec3_range(range: crate::ParticleVec3Range) -> bool {
    is_finite_vec3(range.min) && is_finite_vec3(range.max)
}

fn validate_shape(emitter_id: &str, shape: ParticleShape) -> Result<(), ParticleSimulationError> {
    let valid = match shape {
        ParticleShape::Point => true,
        ParticleShape::Sphere { radius } => radius.is_finite(),
        ParticleShape::Box { half_extents } => is_finite_vec3(half_extents),
        ParticleShape::Cone { radius, height } => radius.is_finite() && height.is_finite(),
    };
    if valid {
        Ok(())
    } else {
        Err(ParticleSimulationError::InvalidAsset(format!(
            "emitter {emitter_id} contains non-finite shape settings"
        )))
    }
}

fn validate_scalar_keys(
    emitter_id: &str,
    keys: &[ParticleScalarKey],
) -> Result<(), ParticleSimulationError> {
    if keys
        .iter()
        .all(|key| key.t.is_finite() && key.value.is_finite())
    {
        Ok(())
    } else {
        Err(ParticleSimulationError::InvalidAsset(format!(
            "emitter {emitter_id} contains non-finite size curve keys"
        )))
    }
}

fn validate_bursts(
    emitter_id: &str,
    bursts: &[crate::ParticleBurst],
) -> Result<(), ParticleSimulationError> {
    if bursts.iter().all(|burst| burst.time_seconds.is_finite()) {
        Ok(())
    } else {
        Err(ParticleSimulationError::InvalidAsset(format!(
            "emitter {emitter_id} contains non-finite burst settings"
        )))
    }
}

fn validate_animation_bindings(
    emitter_id: &str,
    bindings: &[crate::ParticleAnimationBinding],
) -> Result<(), ParticleSimulationError> {
    if bindings
        .iter()
        .all(|binding| binding.normalized_progress.is_finite())
    {
        Ok(())
    } else {
        Err(ParticleSimulationError::InvalidAsset(format!(
            "emitter {emitter_id} contains non-finite animation binding settings"
        )))
    }
}

fn validate_color_keys(
    emitter_id: &str,
    keys: &[ParticleColorKey],
) -> Result<(), ParticleSimulationError> {
    if keys
        .iter()
        .all(|key| key.t.is_finite() && is_finite_vec4(key.value))
    {
        Ok(())
    } else {
        Err(ParticleSimulationError::InvalidAsset(format!(
            "emitter {emitter_id} contains non-finite color curve keys"
        )))
    }
}
