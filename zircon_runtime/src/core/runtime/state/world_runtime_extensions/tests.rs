use super::*;
use crate::scene::ecs::{RuntimeSceneSystemContext, SystemStage};

#[test]
fn default_world_extension_set_is_finalized_before_first_apply() {
    let extensions = WorldRuntimeExtensionSet::default();
    let mut world = World::empty();

    assert!(extensions.registry.is_finalized());
    extensions
        .apply_to_world(&mut world)
        .expect("default extension set should be ready for runtime reads");
}

#[test]
fn failed_install_preserves_the_previous_finalized_registry() {
    let mut extensions = WorldRuntimeExtensionSet::default();
    let initial = registry_with_systems("weather.runtime", &["weather.tick"]);
    extensions.install(&initial).expect("initial install");

    let conflicting = registry_with_systems("storm.runtime", &["storm.prepare", "weather.tick"]);
    assert!(extensions.install(&conflicting).is_err());

    assert!(extensions.registry.is_finalized());
    assert_eq!(
        extensions
            .registry
            .plugin_runtime_systems()
            .map(|(_, system)| system.id.as_str())
            .collect::<Vec<_>>(),
        vec!["weather.tick"]
    );
}

fn registry_with_systems(module_name: &str, system_ids: &[&str]) -> RuntimeExtensionRegistry {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module(module_name)
        .expect("valid plugin module owner");
    for system_id in system_ids {
        registry
            .register_runtime_scene_system(
                owner,
                *system_id,
                SystemStage::Update,
                |_context: RuntimeSceneSystemContext<'_>| Ok(()),
            )
            .register()
            .expect("valid runtime scene system");
    }
    registry
}
