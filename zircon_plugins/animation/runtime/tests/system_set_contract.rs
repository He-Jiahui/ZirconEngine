use zircon_plugin_animation_runtime::{
    package_manifest, plugin_registration, ANIMATION_SYSTEM_SET, PLUGIN_RUNTIME_MODULE_NAME,
};

const ANIMATION_MAIN_SYSTEM_SET: &str = "animation.main";

#[test]
fn animation_manifest_declares_required_main_system_set() {
    assert_eq!(ANIMATION_SYSTEM_SET, ANIMATION_MAIN_SYSTEM_SET);

    let manifest = package_manifest();
    let runtime_module = manifest
        .modules
        .iter()
        .find(|module| module.name == PLUGIN_RUNTIME_MODULE_NAME)
        .expect("animation runtime module should be declared");

    assert_eq!(
        runtime_module.system_sets,
        vec![ANIMATION_MAIN_SYSTEM_SET.to_string()]
    );
}

#[test]
fn animation_runtime_systems_join_main_system_set() {
    let mut report = plugin_registration();
    let main_set = report
        .extensions
        .intern_system_set(ANIMATION_MAIN_SYSTEM_SET)
        .expect("animation.main should be a valid system set");
    let runtime_systems = report
        .extensions
        .plugin_runtime_systems()
        .filter(|(owner, _)| {
            report.extensions.plugin_module_name(*owner) == Some(PLUGIN_RUNTIME_MODULE_NAME)
        })
        .map(|(_, system)| system)
        .collect::<Vec<_>>();

    assert!(!runtime_systems.is_empty());
    for system in runtime_systems {
        assert_eq!(
            system.sets,
            vec![main_set],
            "{} must join animation.main",
            system.id
        );
    }
}
