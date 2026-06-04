use super::support::{static_sound_contributions, STATIC_SOUND_PLUGIN_MANIFEST};

#[test]
fn static_plugin_manifest_keeps_runtime_contribution_keys_in_sync() {
    let static_manifest = static_sound_contributions(STATIC_SOUND_PLUGIN_MANIFEST);
    let runtime_manifest = crate::package_manifest();
    let mut runtime_dependencies = runtime_manifest
        .dependencies
        .iter()
        .map(|dependency| {
            (
                dependency.id.clone(),
                dependency.required,
                dependency.capability.clone(),
            )
        })
        .collect::<Vec<_>>();
    let mut runtime_event_catalogs = runtime_manifest
        .event_catalogs
        .iter()
        .map(|catalog| (catalog.namespace.clone(), catalog.version))
        .collect::<Vec<_>>();
    let mut runtime_modules = runtime_manifest
        .modules
        .iter()
        .map(|module| {
            (
                module.name.clone(),
                module.kind,
                module.crate_name.clone(),
                module.target_modes.clone(),
                module.capabilities.clone(),
            )
        })
        .collect::<Vec<_>>();
    let mut runtime_components = runtime_manifest
        .components
        .iter()
        .map(|component| component.type_id.clone())
        .collect::<Vec<_>>();
    let mut descriptor_components = crate::components::sound_component_descriptors()
        .into_iter()
        .map(|component| component.type_id)
        .collect::<Vec<_>>();

    runtime_dependencies.sort_unstable();
    runtime_event_catalogs.sort_unstable();
    runtime_modules.sort_unstable_by_key(|module| module.0.clone());
    runtime_components.sort_unstable();
    descriptor_components.sort_unstable();

    assert_eq!(static_manifest.dependencies, runtime_dependencies);
    assert_eq!(static_manifest.event_catalogs, runtime_event_catalogs);
    let sound_event_catalog = runtime_manifest
        .event_catalogs
        .iter()
        .find(|catalog| catalog.namespace == crate::SOUND_DYNAMIC_EVENT_NAMESPACE)
        .expect("sound dynamic event catalog");
    assert_eq!(
        sound_event_catalog
            .events
            .iter()
            .map(|event| event.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "sound.dynamic_events.impact",
            "sound.dynamic_events.marker",
            "sound.dynamic_events.ambient_stinger",
        ]
    );
    assert_eq!(static_manifest.modules, runtime_modules);
    assert_eq!(descriptor_components.len(), 3);
    assert_eq!(descriptor_components, runtime_components);
}
