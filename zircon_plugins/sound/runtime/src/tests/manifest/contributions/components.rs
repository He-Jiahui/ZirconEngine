#[test]
fn static_plugin_manifest_keeps_component_descriptors_in_sync() {
    let runtime_manifest = crate::package_manifest();
    let mut runtime_components = runtime_manifest
        .components
        .iter()
        .map(|component| component.type_id.clone())
        .collect::<Vec<_>>();
    let mut descriptor_components = crate::components::sound_component_descriptors()
        .into_iter()
        .map(|component| component.type_id)
        .collect::<Vec<_>>();

    runtime_components.sort_unstable();
    descriptor_components.sort_unstable();

    assert_eq!(descriptor_components.len(), 3);
    assert_eq!(descriptor_components, runtime_components);
}
