use crate::{
    plugin_feature_registration, NET_CONTENT_DOWNLOAD_FEATURE_CAPABILITY,
    NET_CONTENT_DOWNLOAD_FEATURE_ID, NET_CONTENT_DOWNLOAD_FEATURE_MANAGER_NAME,
    NET_CONTENT_DOWNLOAD_FEATURE_MODULE_NAME,
};

#[test]
fn content_download_feature_registration_contributes_runtime_module_and_manager() {
    let report = plugin_feature_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert_eq!(report.manifest.id, NET_CONTENT_DOWNLOAD_FEATURE_ID);
    assert!(report
        .manifest
        .capabilities
        .iter()
        .any(|capability| capability == NET_CONTENT_DOWNLOAD_FEATURE_CAPABILITY));
    assert!(report.manifest.dependencies.iter().any(|dependency| {
        dependency.plugin_id == zircon_plugin_net_runtime::PLUGIN_ID
            && dependency.capability == "runtime.feature.net.http"
    }));
    let module = report
        .extensions
        .modules()
        .iter()
        .find(|module| module.name == NET_CONTENT_DOWNLOAD_FEATURE_MODULE_NAME)
        .expect("content download feature module should be registered");
    assert_eq!(
        module.managers[0].name.to_string(),
        NET_CONTENT_DOWNLOAD_FEATURE_MANAGER_NAME
    );
}
