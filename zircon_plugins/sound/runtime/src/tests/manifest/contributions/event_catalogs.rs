use super::super::support::{static_sound_contributions, STATIC_SOUND_PLUGIN_MANIFEST};

#[test]
fn static_plugin_manifest_keeps_runtime_event_catalogs_in_sync() {
    let static_manifest = static_sound_contributions(STATIC_SOUND_PLUGIN_MANIFEST);
    let runtime_manifest = crate::package_manifest();
    let mut runtime_event_catalogs = runtime_manifest
        .event_catalogs
        .iter()
        .map(|catalog| (catalog.namespace.clone(), catalog.version))
        .collect::<Vec<_>>();

    runtime_event_catalogs.sort_unstable();

    assert_eq!(static_manifest.event_catalogs, runtime_event_catalogs);
}
