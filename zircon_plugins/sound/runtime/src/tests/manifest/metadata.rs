use super::support::{static_plugin_metadata, STATIC_SOUND_PLUGIN_MANIFEST};

#[test]
fn runtime_descriptor_keeps_static_maturity_and_capability_status_in_sync() {
    let static_manifest = static_plugin_metadata(STATIC_SOUND_PLUGIN_MANIFEST);
    let descriptor = crate::runtime_plugin_descriptor();
    let runtime_manifest = crate::package_manifest();
    let catalog_descriptor = zircon_runtime::plugin::RuntimePluginDescriptor::builtin_catalog()
        .into_iter()
        .find(|descriptor| descriptor.runtime_id == zircon_runtime::RuntimePluginId::Sound)
        .expect("built-in runtime catalog should include sound");

    assert_eq!(static_manifest.maturity, descriptor.maturity);
    assert_eq!(static_manifest.maturity, runtime_manifest.maturity);
    assert_eq!(static_manifest.maturity, catalog_descriptor.maturity);
    assert_eq!(
        static_manifest.capability_statuses,
        descriptor.capability_statuses
    );
    assert_eq!(
        static_manifest.capability_statuses,
        runtime_manifest.capability_statuses
    );
    assert_eq!(
        static_manifest.capability_statuses,
        catalog_descriptor.capability_statuses
    );
    assert!(runtime_manifest.capability_statuses.iter().any(|status| {
        status.capability == "runtime.plugin.sound"
            && status.status == zircon_runtime::plugin::CapabilityStatus::Partial
    }));
}
