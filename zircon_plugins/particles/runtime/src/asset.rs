use zircon_runtime::core::math::{Real, Vec3, Vec4};
use zircon_runtime::core::resource::{MaterialMarker, ResourceHandle, TextureMarker};

use crate::interop::{ParticleAnimationBinding, ParticlePhysicsOptions};
use crate::simulation::ParticleRng;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ParticleSimulationBackend {
    #[default]
    Cpu,
    Gpu,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ParticleCoordinateSpace {
    #[default]
    Local,
    World,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleScalarRange {
    pub min: Real,
    pub max: Real,
}

impl ParticleScalarRange {
    pub const fn constant(value: Real) -> Self {
        Self {
            min: value,
            max: value,
        }
    }

    pub const fn new(min: Real, max: Real) -> Self {
        Self { min, max }
    }

    pub(crate) fn sample(self, rng: &mut ParticleRng) -> Real {
        self.min + (self.max - self.min) * rng.next_unit()
    }

    pub fn normalized(self) -> Self {
        if self.min <= self.max {
            self
        } else {
            Self {
                min: self.max,
                max: self.min,
            }
        }
    }
}

impl Default for ParticleScalarRange {
    fn default() -> Self {
        Self::constant(1.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleVec3Range {
    pub min: Vec3,
    pub max: Vec3,
}

impl ParticleVec3Range {
    pub const fn constant(value: Vec3) -> Self {
        Self {
            min: value,
            max: value,
        }
    }

    pub const fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub(crate) fn sample(self, rng: &mut ParticleRng) -> Vec3 {
        Vec3::new(
            self.min.x + (self.max.x - self.min.x) * rng.next_unit(),
            self.min.y + (self.max.y - self.min.y) * rng.next_unit(),
            self.min.z + (self.max.z - self.min.z) * rng.next_unit(),
        )
    }
}

impl Default for ParticleVec3Range {
    fn default() -> Self {
        Self::constant(Vec3::ZERO)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleScalarKey {
    pub t: Real,
    pub value: Real,
}

impl ParticleScalarKey {
    pub const fn new(t: Real, value: Real) -> Self {
        Self { t, value }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleColorKey {
    pub t: Real,
    pub value: Vec4,
}

impl ParticleColorKey {
    pub const fn new(t: Real, value: Vec4) -> Self {
        Self { t, value }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleBurst {
    pub time_seconds: Real,
    pub count: u32,
}

impl ParticleBurst {
    pub const fn new(time_seconds: Real, count: u32) -> Self {
        Self {
            time_seconds,
            count,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ParticleShape {
    Point,
    Sphere { radius: Real },
    Box { half_extents: Vec3 },
    Cone { radius: Real, height: Real },
}

impl ParticleShape {
    pub(crate) fn sample_position(self, rng: &mut ParticleRng) -> Vec3 {
        match self {
            Self::Point => Vec3::ZERO,
            Self::Sphere { radius } => {
                let direction = rng.next_unit_vector();
                let distance = radius.max(0.0) * rng.next_unit().cbrt();
                direction * distance
            }
            Self::Box { half_extents } => Vec3::new(
                rng.next_signed() * half_extents.x.abs(),
                rng.next_signed() * half_extents.y.abs(),
                rng.next_signed() * half_extents.z.abs(),
            ),
            Self::Cone { radius, height } => {
                let y = rng.next_unit() * height.max(0.0);
                let taper = if height.abs() <= Real::EPSILON {
                    1.0
                } else {
                    1.0 - (y / height.max(Real::EPSILON)).clamp(0.0, 1.0)
                };
                let angle = rng.next_unit() * std::f32::consts::TAU;
                let r = radius.max(0.0) * taper * rng.next_unit().sqrt();
                Vec3::new(angle.cos() * r, y, angle.sin() * r)
            }
        }
    }
}

impl Default for ParticleShape {
    fn default() -> Self {
        Self::Point
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParticleEmitterAsset {
    pub id: String,
    pub max_particles: u32,
    pub spawn_rate_per_second: Real,
    pub bursts: Vec<ParticleBurst>,
    pub lifetime: ParticleScalarRange,
    pub initial_size: ParticleScalarRange,
    pub initial_velocity: ParticleVec3Range,
    pub initial_rotation: ParticleScalarRange,
    pub initial_angular_velocity: ParticleScalarRange,
    pub start_color: Vec4,
    pub gravity: Vec3,
    pub drag: Real,
    pub shape: ParticleShape,
    pub coordinate_space: ParticleCoordinateSpace,
    pub material: Option<ResourceHandle<MaterialMarker>>,
    pub texture: Option<ResourceHandle<TextureMarker>>,
    pub physics: ParticlePhysicsOptions,
    pub animation_bindings: Vec<ParticleAnimationBinding>,
    pub color_over_lifetime: Vec<ParticleColorKey>,
    pub size_over_lifetime: Vec<ParticleScalarKey>,
}

impl ParticleEmitterAsset {
    pub fn sprite(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            ..Self::default()
        }
    }

    pub fn with_max_particles(mut self, max_particles: u32) -> Self {
        self.max_particles = max_particles;
        self
    }

    pub fn with_spawn_rate(mut self, spawn_rate_per_second: Real) -> Self {
        self.spawn_rate_per_second = spawn_rate_per_second;
        self
    }

    pub fn with_burst(mut self, burst: ParticleBurst) -> Self {
        self.bursts.push(burst);
        self
    }

    pub fn with_lifetime(mut self, lifetime: ParticleScalarRange) -> Self {
        self.lifetime = lifetime.normalized();
        self
    }

    pub fn with_shape(mut self, shape: ParticleShape) -> Self {
        self.shape = shape;
        self
    }

    pub fn with_initial_velocity(mut self, velocity: ParticleVec3Range) -> Self {
        self.initial_velocity = velocity;
        self
    }

    pub fn with_initial_rotation(mut self, rotation: ParticleScalarRange) -> Self {
        self.initial_rotation = rotation.normalized();
        self
    }

    pub fn with_initial_angular_velocity(mut self, angular_velocity: ParticleScalarRange) -> Self {
        self.initial_angular_velocity = angular_velocity.normalized();
        self
    }

    pub fn with_gravity(mut self, gravity: Vec3) -> Self {
        self.gravity = gravity;
        self
    }

    pub fn with_drag(mut self, drag: Real) -> Self {
        self.drag = drag.max(0.0);
        self
    }

    pub fn with_coordinate_space(mut self, coordinate_space: ParticleCoordinateSpace) -> Self {
        self.coordinate_space = coordinate_space;
        self
    }

    pub fn with_material(mut self, material: ResourceHandle<MaterialMarker>) -> Self {
        self.material = Some(material);
        self
    }

    pub fn with_texture(mut self, texture: ResourceHandle<TextureMarker>) -> Self {
        self.texture = Some(texture);
        self
    }

    pub fn with_physics(mut self, physics: ParticlePhysicsOptions) -> Self {
        self.physics = physics;
        self
    }

    pub fn with_animation_binding(mut self, binding: ParticleAnimationBinding) -> Self {
        self.animation_bindings.push(binding);
        self
    }

    pub fn with_color_over_lifetime(mut self, keys: Vec<ParticleColorKey>) -> Self {
        self.color_over_lifetime = normalize_color_keys(keys);
        self
    }

    pub fn with_size_over_lifetime(mut self, keys: Vec<ParticleScalarKey>) -> Self {
        self.size_over_lifetime = normalize_scalar_keys(keys);
        self
    }
}

impl Default for ParticleEmitterAsset {
    fn default() -> Self {
        Self {
            id: "emitter".to_string(),
            max_particles: 128,
            spawn_rate_per_second: 16.0,
            bursts: Vec::new(),
            lifetime: ParticleScalarRange::new(1.0, 1.0),
            initial_size: ParticleScalarRange::constant(0.25),
            initial_velocity: ParticleVec3Range::default(),
            initial_rotation: ParticleScalarRange::constant(0.0),
            initial_angular_velocity: ParticleScalarRange::constant(0.0),
            start_color: Vec4::ONE,
            gravity: Vec3::ZERO,
            drag: 0.0,
            shape: ParticleShape::Point,
            coordinate_space: ParticleCoordinateSpace::Local,
            material: None,
            texture: None,
            physics: ParticlePhysicsOptions::disabled(),
            animation_bindings: Vec::new(),
            color_over_lifetime: vec![
                ParticleColorKey::new(0.0, Vec4::ONE),
                ParticleColorKey::new(1.0, Vec4::ONE),
            ],
            size_over_lifetime: vec![
                ParticleScalarKey::new(0.0, 1.0),
                ParticleScalarKey::new(1.0, 1.0),
            ],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParticleSystemAsset {
    pub id: String,
    pub backend: ParticleSimulationBackend,
    pub seed: u64,
    pub looped: bool,
    pub emitters: Vec<ParticleEmitterAsset>,
}

impl ParticleSystemAsset {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            ..Self::default()
        }
    }

    pub fn with_backend(mut self, backend: ParticleSimulationBackend) -> Self {
        self.backend = backend;
        self
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    pub fn with_looped(mut self, looped: bool) -> Self {
        self.looped = looped;
        self
    }

    pub fn with_emitter(mut self, emitter: ParticleEmitterAsset) -> Self {
        self.emitters.push(emitter);
        self
    }

    pub fn with_emitters(mut self, emitters: Vec<ParticleEmitterAsset>) -> Self {
        self.emitters = emitters;
        self
    }
}

impl Default for ParticleSystemAsset {
    fn default() -> Self {
        Self {
            id: "particle_system".to_string(),
            backend: ParticleSimulationBackend::Cpu,
            seed: 0xC0FFEE,
            looped: true,
            emitters: vec![ParticleEmitterAsset::default()],
        }
    }
}

pub(crate) fn evaluate_scalar_curve(keys: &[ParticleScalarKey], t: Real) -> Real {
    if keys.is_empty() {
        return 1.0;
    }
    let t = t.clamp(0.0, 1.0);
    for window in keys.windows(2) {
        let a = window[0];
        let b = window[1];
        if t >= a.t && t <= b.t {
            let span = (b.t - a.t).max(Real::EPSILON);
            let local = ((t - a.t) / span).clamp(0.0, 1.0);
            return a.value + (b.value - a.value) * local;
        }
    }
    if t < keys[0].t {
        keys[0].value
    } else {
        keys[keys.len() - 1].value
    }
}

pub(crate) fn evaluate_color_curve(keys: &[ParticleColorKey], t: Real) -> Vec4 {
    if keys.is_empty() {
        return Vec4::ONE;
    }
    let t = t.clamp(0.0, 1.0);
    for window in keys.windows(2) {
        let a = window[0];
        let b = window[1];
        if t >= a.t && t <= b.t {
            let span = (b.t - a.t).max(Real::EPSILON);
            let local = ((t - a.t) / span).clamp(0.0, 1.0);
            return a.value.lerp(b.value, local);
        }
    }
    if t < keys[0].t {
        keys[0].value
    } else {
        keys[keys.len() - 1].value
    }
}

fn normalize_scalar_keys(mut keys: Vec<ParticleScalarKey>) -> Vec<ParticleScalarKey> {
    if keys.is_empty() {
        return vec![
            ParticleScalarKey::new(0.0, 1.0),
            ParticleScalarKey::new(1.0, 1.0),
        ];
    }
    keys.sort_by(|a, b| a.t.total_cmp(&b.t));
    keys
}

fn normalize_color_keys(mut keys: Vec<ParticleColorKey>) -> Vec<ParticleColorKey> {
    if keys.is_empty() {
        return vec![
            ParticleColorKey::new(0.0, Vec4::ONE),
            ParticleColorKey::new(1.0, Vec4::ONE),
        ];
    }
    keys.sort_by(|a, b| a.t.total_cmp(&b.t));
    keys
}
