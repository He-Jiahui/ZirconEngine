use zircon_runtime::core::CoreRuntime;

use crate::{
    module_descriptor, ParticleSystemComponent, ParticlesManager, PARTICLES_MANAGER_NAME,
    PARTICLES_MODULE_NAME,
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
