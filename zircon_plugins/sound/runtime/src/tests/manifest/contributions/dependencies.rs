use super::super::support::{static_sound_contributions, STATIC_SOUND_PLUGIN_MANIFEST};

#[test]
fn static_plugin_manifest_keeps_runtime_dependencies_in_sync() {
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

    runtime_dependencies.sort_unstable();

    assert_eq!(static_manifest.dependencies, runtime_dependencies);
}
