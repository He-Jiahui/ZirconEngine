use serde::{Deserialize, Serialize};
use zircon_plugin_particles_runtime::{
    ParticleCoordinateSpace, ParticleEmitterAsset, ParticleShape, ParticleSystemAsset,
};
use zircon_runtime::core::math::Real;

/// Editable source values, independent of process-local handles and preview clocks.
/// The document host remains responsible for transactions, history and durable writes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParticleSystemSource {
    #[serde(default = "source_version")]
    pub format_version: u32,
    pub system: ParticleSourceSystem,
    #[serde(default)]
    pub emitters: Vec<ParticleSourceEmitter>,
}

fn source_version() -> u32 {
    1
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParticleSourceBackend {
    #[default]
    Cpu,
    Gpu,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParticleSourceCoordinateSpace {
    #[default]
    Local,
    World,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ParticleSourceSystem {
    pub id: String,
    pub backend: ParticleSourceBackend,
    #[serde(with = "super::seed")]
    pub seed: u64,
    pub looped: bool,
}

impl Default for ParticleSourceSystem {
    fn default() -> Self {
        let source = ParticleSystemAsset::default();
        Self {
            id: source.id,
            backend: ParticleSourceBackend::Cpu,
            seed: source.seed,
            looped: source.looped,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParticleSourceScalarRange {
    pub min: Real,
    pub max: Real,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParticleSourceVec3Range {
    pub min: [Real; 3],
    pub max: [Real; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum ParticleSourceShape {
    Point,
    Sphere { radius: Real },
    Box { half_extents: [Real; 3] },
    Cone { radius: Real, height: Real },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParticleSourceBurst {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    pub time_seconds: Real,
    pub count: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParticleSourceScalarKey {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    pub t: Real,
    pub value: Real,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParticleSourceColorKey {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    pub t: Real,
    pub value: [Real; 4],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ParticleSourcePhysics {
    pub external_force: [Real; 3],
    pub collision_enabled: bool,
    pub bounce: Real,
    pub damping: Real,
}

impl Default for ParticleSourcePhysics {
    fn default() -> Self {
        Self {
            external_force: [0.0; 3],
            collision_enabled: false,
            bounce: 0.0,
            damping: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParticleSourceAnimationBinding {
    pub parameter: String,
    pub curve_path: String,
    pub normalized_progress: Real,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ParticleSourceEmitter {
    /// Stable authoring identity. Legacy files receive it once on decode; it is saved with the item.
    pub key: Option<String>,
    pub id: String,
    /// Renaming a label does not change the runtime id or the persistent authoring key.
    pub display_name: Option<String>,
    pub max_particles: u32,
    pub spawn_rate_per_second: Real,
    pub bursts: Vec<ParticleSourceBurst>,
    pub lifetime: ParticleSourceScalarRange,
    pub initial_size: ParticleSourceScalarRange,
    pub initial_velocity: ParticleSourceVec3Range,
    pub initial_rotation: ParticleSourceScalarRange,
    pub initial_angular_velocity: ParticleSourceScalarRange,
    pub start_color: [Real; 4],
    pub gravity: [Real; 3],
    pub drag: Real,
    pub shape: ParticleSourceShape,
    pub coordinate_space: ParticleSourceCoordinateSpace,
    /// Persistent locators, resolved by the existing resource owner before runtime installation.
    pub material: Option<String>,
    pub texture: Option<String>,
    pub physics: ParticleSourcePhysics,
    pub animation_bindings: Vec<ParticleSourceAnimationBinding>,
    pub color_over_lifetime: Vec<ParticleSourceColorKey>,
    pub size_over_lifetime: Vec<ParticleSourceScalarKey>,
}

impl Default for ParticleSourceEmitter {
    fn default() -> Self {
        let emitter = ParticleEmitterAsset::default();
        let scalar =
            |r: zircon_plugin_particles_runtime::ParticleScalarRange| ParticleSourceScalarRange {
                min: r.min,
                max: r.max,
            };
        Self {
            key: None,
            id: emitter.id,
            display_name: None,
            max_particles: emitter.max_particles,
            spawn_rate_per_second: emitter.spawn_rate_per_second,
            bursts: Vec::new(),
            lifetime: scalar(emitter.lifetime),
            initial_size: scalar(emitter.initial_size),
            initial_velocity: ParticleSourceVec3Range {
                min: [
                    emitter.initial_velocity.min.x,
                    emitter.initial_velocity.min.y,
                    emitter.initial_velocity.min.z,
                ],
                max: [
                    emitter.initial_velocity.max.x,
                    emitter.initial_velocity.max.y,
                    emitter.initial_velocity.max.z,
                ],
            },
            initial_rotation: scalar(emitter.initial_rotation),
            initial_angular_velocity: scalar(emitter.initial_angular_velocity),
            start_color: [
                emitter.start_color.x,
                emitter.start_color.y,
                emitter.start_color.z,
                emitter.start_color.w,
            ],
            gravity: [emitter.gravity.x, emitter.gravity.y, emitter.gravity.z],
            drag: emitter.drag,
            shape: match emitter.shape {
                ParticleShape::Point => ParticleSourceShape::Point,
                ParticleShape::Sphere { radius } => ParticleSourceShape::Sphere { radius },
                ParticleShape::Box { half_extents } => ParticleSourceShape::Box {
                    half_extents: [half_extents.x, half_extents.y, half_extents.z],
                },
                ParticleShape::Cone { radius, height } => {
                    ParticleSourceShape::Cone { radius, height }
                }
            },
            coordinate_space: match emitter.coordinate_space {
                ParticleCoordinateSpace::Local => ParticleSourceCoordinateSpace::Local,
                ParticleCoordinateSpace::World => ParticleSourceCoordinateSpace::World,
            },
            material: None,
            texture: None,
            physics: ParticleSourcePhysics::default(),
            animation_bindings: Vec::new(),
            color_over_lifetime: emitter
                .color_over_lifetime
                .into_iter()
                .map(|k| ParticleSourceColorKey {
                    key: None,
                    t: k.t,
                    value: [k.value.x, k.value.y, k.value.z, k.value.w],
                })
                .collect(),
            size_over_lifetime: emitter
                .size_over_lifetime
                .into_iter()
                .map(|k| ParticleSourceScalarKey {
                    key: None,
                    t: k.t,
                    value: k.value,
                })
                .collect(),
        }
    }
}
