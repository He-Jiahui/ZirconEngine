use std::sync::{Arc, Mutex};

use zircon_runtime::core::{
    ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject, StartupMode,
};
use zircon_runtime::engine_module::{factory, qualified_name};
use zircon_runtime::graphics::{
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassStage,
};
use zircon_runtime::render_graph::QueueLane;

pub const PLUGIN_ID: &str = "particles";
pub const PARTICLES_FEATURE_NAME: &str = "particle";
pub const PARTICLES_MODULE_NAME: &str = "ParticlesModule";
pub const PARTICLES_MANAGER_NAME: &str = "ParticlesModule.Manager.ParticlesManager";

#[derive(Clone, Debug, PartialEq)]
pub struct ParticleEmitterState {
    pub emitter_id: String,
    pub live_particles: usize,
}

#[derive(Clone, Debug, Default)]
pub struct DefaultParticlesManager {
    emitters: Arc<Mutex<Vec<ParticleEmitterState>>>,
}

impl DefaultParticlesManager {
    pub fn emit(&self, emitter_id: impl Into<String>, count: usize) -> ParticleEmitterState {
        let state = ParticleEmitterState {
            emitter_id: emitter_id.into(),
            live_particles: count,
        };
        self.emitters
            .lock()
            .expect("particles emitters mutex poisoned")
            .push(state.clone());
        state
    }

    pub fn snapshot(&self) -> Vec<ParticleEmitterState> {
        self.emitters
            .lock()
            .expect("particles emitters mutex poisoned")
            .clone()
    }
}

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(PARTICLES_MODULE_NAME, "Particle simulation runtime plugin").with_manager(
        ManagerDescriptor::new(
            qualified_name(
                PARTICLES_MODULE_NAME,
                ServiceKind::Manager,
                "ParticlesManager",
            ),
            StartupMode::Lazy,
            Vec::new(),
            factory(|_| Ok(Arc::new(DefaultParticlesManager::default()) as ServiceObject)),
        ),
    )
}

#[derive(Clone, Debug)]
pub struct ParticlesRuntimePlugin {
    descriptor: zircon_runtime::RuntimePluginDescriptor,
}

impl ParticlesRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl zircon_runtime::RuntimePlugin for ParticlesRuntimePlugin {
    fn descriptor(&self) -> &zircon_runtime::RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register_runtime_extensions(
        &self,
        registry: &mut zircon_runtime::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::RuntimeExtensionRegistryError> {
        registry.register_module(module_descriptor())?;
        registry.register_render_feature(render_feature_descriptor())
    }
}

pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        PARTICLES_FEATURE_NAME,
        vec![
            "view".to_string(),
            "particles".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::Transparent,
            "particle-render",
            QueueLane::Graphics,
        )
        .with_executor_id("particle.transparent")
        .read_texture("scene-depth")
        .read_texture("scene-color")
        .write_texture("scene-color")],
    )
}

pub fn runtime_plugin_descriptor() -> zircon_runtime::RuntimePluginDescriptor {
    zircon_runtime::RuntimePluginDescriptor::new(
        PLUGIN_ID,
        "Particles",
        zircon_runtime::RuntimePluginId::Particles,
        "zircon_plugin_particles_runtime",
    )
    .with_target_modes([
        zircon_runtime::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::RuntimeTargetMode::EditorHost,
    ])
    .with_capability("runtime.plugin.particles")
}

pub fn runtime_plugin() -> ParticlesRuntimePlugin {
    ParticlesRuntimePlugin::new()
}

pub fn package_manifest() -> zircon_runtime::PluginPackageManifest {
    zircon_runtime::RuntimePlugin::package_manifest(&runtime_plugin())
}

pub fn runtime_selection() -> zircon_runtime::ProjectPluginSelection {
    zircon_runtime::RuntimePlugin::project_selection(&runtime_plugin())
}

pub fn plugin_registration() -> zircon_runtime::RuntimePluginRegistrationReport {
    zircon_runtime::RuntimePluginRegistrationReport::from_plugin(&runtime_plugin())
}

pub fn runtime_capabilities() -> &'static [&'static str] {
    &["runtime.plugin.particles"]
}

#[cfg(test)]
mod tests {
    use zircon_runtime::core::CoreRuntime;

    use super::*;

    #[test]
    fn particles_registration_contributes_runtime_module() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .modules()
            .iter()
            .any(|module| module.name == PARTICLES_MODULE_NAME));
        assert!(report
            .extensions
            .render_features()
            .iter()
            .any(|feature| feature.name == PARTICLES_FEATURE_NAME));
        assert_eq!(
            report.package_manifest.modules[0].target_modes,
            vec![
                zircon_runtime::RuntimeTargetMode::ClientRuntime,
                zircon_runtime::RuntimeTargetMode::EditorHost,
            ]
        );
    }

    #[test]
    fn particles_module_resolves_manager_and_tracks_emission() {
        let runtime = CoreRuntime::new();
        runtime.register_module(module_descriptor()).unwrap();
        runtime.activate_module(PARTICLES_MODULE_NAME).unwrap();
        let manager = runtime
            .handle()
            .resolve_manager::<DefaultParticlesManager>(PARTICLES_MANAGER_NAME)
            .unwrap();

        let emitted = manager.emit("sparks", 12);

        assert_eq!(emitted.live_particles, 12);
        assert_eq!(manager.snapshot(), vec![emitted]);
    }
}
