pub(super) struct ParticlesFeatureRow {
    pub id_suffix: &'static str,
    pub display_name: &'static str,
    pub capability: &'static str,
    pub extra_dependencies: &'static [ParticlesFeatureDependencyRow],
}

pub(super) struct ParticlesFeatureDependencyRow {
    pub provider_plugin_id: &'static str,
    pub capability: &'static str,
}

const PHYSICS_DEPENDENCIES: &[ParticlesFeatureDependencyRow] = &[ParticlesFeatureDependencyRow {
    provider_plugin_id: "physics",
    capability: "runtime.plugin.physics",
}];

const ANIMATION_CONTROL_DEPENDENCIES: &[ParticlesFeatureDependencyRow] =
    &[ParticlesFeatureDependencyRow {
        provider_plugin_id: "animation",
        capability: "runtime.plugin.animation",
    }];

const GPU_SIMULATION_DEPENDENCIES: &[ParticlesFeatureDependencyRow] =
    &[ParticlesFeatureDependencyRow {
        provider_plugin_id: "render_graph",
        capability: "runtime.module.render_graph",
    }];

pub(super) const PARTICLES_FEATURE_ROWS: &[ParticlesFeatureRow] = &[
    ParticlesFeatureRow {
        id_suffix: "physics",
        display_name: "Physical Particles",
        capability: "runtime.feature.particles.physics",
        extra_dependencies: PHYSICS_DEPENDENCIES,
    },
    ParticlesFeatureRow {
        id_suffix: "animation_control",
        display_name: "Animation Controlled Particles",
        capability: "runtime.feature.particles.animation_control",
        extra_dependencies: ANIMATION_CONTROL_DEPENDENCIES,
    },
    ParticlesFeatureRow {
        id_suffix: "gpu_simulation",
        display_name: "GPU Particle Simulation",
        capability: "runtime.feature.particles.gpu_simulation",
        extra_dependencies: GPU_SIMULATION_DEPENDENCIES,
    },
];
