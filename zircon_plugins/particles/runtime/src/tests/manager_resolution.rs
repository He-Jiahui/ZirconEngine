use zircon_runtime::core::CoreRuntime;
use zircon_runtime::plugin::RuntimePlugin;

use crate::{
    module_descriptor, runtime_plugin, ParticleSimulationBackend, ParticleSystemComponent,
    ParticlesManager, PARTICLES_MANAGER_NAME, PARTICLES_MODULE_NAME,
};

use super::support::spawn_rate_asset;

#[test]
fn particles_module_resolves_manager_and_ticks_cpu_spawn_rate() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(PARTICLES_MODULE_NAME).unwrap();
    let manager = runtime
        .handle()
        .resolve_manager::<ParticlesManager>(PARTICLES_MANAGER_NAME)
        .unwrap();

    let handle = manager
        .instantiate(ParticleSystemComponent::new(7, spawn_rate_asset(4.0, 8)))
        .unwrap();
    manager.tick(0.25).unwrap();

    let snapshot = manager.snapshot();
    assert_eq!(snapshot.emitters[0].handle, handle);
    assert_eq!(snapshot.emitters[0].entity, 7);
    assert_eq!(snapshot.emitters[0].live_particles, 1);
    assert_eq!(snapshot.sprites.len(), 1);
}

#[test]
fn particles_runtime_plugin_module_and_runtime_prepare_share_manager() {
    let plugin = runtime_plugin();
    let manager = plugin.manager();
    let runtime = CoreRuntime::new();
    let mut registry = zircon_runtime::plugin::RuntimeExtensionRegistry::default();

    plugin.register(&mut registry).unwrap();
    for module in registry.modules() {
        runtime.register_module(module.clone()).unwrap();
    }
    runtime.activate_module(PARTICLES_MODULE_NAME).unwrap();

    let resolved = runtime
        .handle()
        .resolve_manager::<ParticlesManager>(PARTICLES_MANAGER_NAME)
        .unwrap();
    let handle = resolved
        .instantiate(ParticleSystemComponent::new(
            9,
            spawn_rate_asset(0.0, 8).with_backend(ParticleSimulationBackend::Gpu),
        ))
        .unwrap();

    assert_eq!(
        manager.gpu_runtime_instances()[0].handle,
        handle,
        "plugin runtime-prepare collector and module service must observe the same manager state"
    );
}
