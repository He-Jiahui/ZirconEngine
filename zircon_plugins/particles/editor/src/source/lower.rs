use zircon_plugin_particles_runtime::{
    ParticleAnimationBinding, ParticleBurst, ParticleColorKey, ParticleCoordinateSpace,
    ParticleEmitterAsset, ParticlePhysicsOptions, ParticleScalarKey, ParticleScalarRange,
    ParticleShape, ParticleSimulationBackend, ParticleSystemAsset, ParticleVec3Range,
};
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::math::{Real, Vec3, Vec4};
use zircon_runtime::core::resource::{MaterialMarker, ResourceHandle, TextureMarker};

use super::{
    ParticleSourceBackend, ParticleSourceCoordinateSpace, ParticleSourceError, ParticleSourceShape,
    ParticleSystemSource,
};

impl ParticleSystemSource {
    /// A real CPU candidate for resources such as the existing starter template.
    /// A resource reference without a ready handle is an error, never silently discarded.
    pub fn to_cpu_asset(&self) -> Result<ParticleSystemAsset, ParticleSourceError> {
        if self.system.backend != ParticleSourceBackend::Cpu {
            return Err(ParticleSourceError::invalid(
                "system.backend",
                "CPU lowering requires Cpu",
            ));
        }
        self.to_runtime_asset(|_| None, |_| None)
    }

    /// The existing resource owner supplies ready handles. Locators alone do not imply readiness.
    pub fn to_runtime_asset(
        &self,
        mut material: impl FnMut(&AssetUri) -> Option<ResourceHandle<MaterialMarker>>,
        mut texture: impl FnMut(&AssetUri) -> Option<ResourceHandle<TextureMarker>>,
    ) -> Result<ParticleSystemAsset, ParticleSourceError> {
        self.validate()?;
        let mut emitters = Vec::with_capacity(self.emitters.len());
        for source in &self.emitters {
            let path = format!(
                "emitters[{}]",
                source.key.as_deref().expect("validated key")
            );
            let material = source
                .material
                .as_ref()
                .map(|locator| {
                    let uri = AssetUri::parse(locator).map_err(|e| {
                        ParticleSourceError::invalid(format!("{path}.material"), e.to_string())
                    })?;
                    material(&uri).ok_or_else(|| {
                        ParticleSourceError::invalid(
                            format!("{path}.material"),
                            format!("resource is not ready: {locator}"),
                        )
                    })
                })
                .transpose()?;
            let texture = source
                .texture
                .as_ref()
                .map(|locator| {
                    let uri = AssetUri::parse(locator).map_err(|e| {
                        ParticleSourceError::invalid(format!("{path}.texture"), e.to_string())
                    })?;
                    texture(&uri).ok_or_else(|| {
                        ParticleSourceError::invalid(
                            format!("{path}.texture"),
                            format!("resource is not ready: {locator}"),
                        )
                    })
                })
                .transpose()?;
            let scalar =
                |v: super::ParticleSourceScalarRange| ParticleScalarRange::new(v.min, v.max);
            emitters.push(ParticleEmitterAsset {
                id: source.id.clone(),
                max_particles: source.max_particles,
                spawn_rate_per_second: source.spawn_rate_per_second,
                bursts: source
                    .bursts
                    .iter()
                    .map(|b| ParticleBurst::new(b.time_seconds, b.count))
                    .collect(),
                lifetime: scalar(source.lifetime),
                initial_size: scalar(source.initial_size),
                initial_velocity: ParticleVec3Range::new(
                    vec3(source.initial_velocity.min),
                    vec3(source.initial_velocity.max),
                ),
                initial_rotation: scalar(source.initial_rotation),
                initial_angular_velocity: scalar(source.initial_angular_velocity),
                start_color: vec4(source.start_color),
                gravity: vec3(source.gravity),
                drag: source.drag,
                shape: match source.shape {
                    ParticleSourceShape::Point => ParticleShape::Point,
                    ParticleSourceShape::Sphere { radius } => ParticleShape::Sphere { radius },
                    ParticleSourceShape::Box { half_extents } => ParticleShape::Box {
                        half_extents: vec3(half_extents),
                    },
                    ParticleSourceShape::Cone { radius, height } => {
                        ParticleShape::Cone { radius, height }
                    }
                },
                coordinate_space: match source.coordinate_space {
                    ParticleSourceCoordinateSpace::Local => ParticleCoordinateSpace::Local,
                    ParticleSourceCoordinateSpace::World => ParticleCoordinateSpace::World,
                },
                material,
                texture,
                physics: ParticlePhysicsOptions {
                    external_force: vec3(source.physics.external_force),
                    collision_enabled: source.physics.collision_enabled,
                    bounce: source.physics.bounce,
                    damping: source.physics.damping,
                },
                animation_bindings: source
                    .animation_bindings
                    .iter()
                    .map(|b| ParticleAnimationBinding {
                        parameter: b.parameter.clone(),
                        curve_path: b.curve_path.clone(),
                        normalized_progress: b.normalized_progress,
                    })
                    .collect(),
                color_over_lifetime: source
                    .color_over_lifetime
                    .iter()
                    .map(|k| ParticleColorKey::new(k.t, vec4(k.value)))
                    .collect(),
                size_over_lifetime: source
                    .size_over_lifetime
                    .iter()
                    .map(|k| ParticleScalarKey::new(k.t, k.value))
                    .collect(),
            });
        }
        Ok(ParticleSystemAsset {
            id: self.system.id.clone(),
            backend: match self.system.backend {
                ParticleSourceBackend::Cpu => ParticleSimulationBackend::Cpu,
                ParticleSourceBackend::Gpu => ParticleSimulationBackend::Gpu,
            },
            seed: self.system.seed,
            looped: self.system.looped,
            emitters,
        })
    }
}

fn vec3(v: [Real; 3]) -> Vec3 {
    Vec3::new(v[0], v[1], v[2])
}
fn vec4(v: [Real; 4]) -> Vec4 {
    Vec4::new(v[0], v[1], v[2], v[3])
}
