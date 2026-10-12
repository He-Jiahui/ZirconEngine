use crate::{
    plugin_feature_registration, NET_RELIABLE_UDP_FEATURE_CAPABILITY, NET_RELIABLE_UDP_FEATURE_ID,
    NET_RELIABLE_UDP_FEATURE_MANAGER_NAME, NET_RELIABLE_UDP_FEATURE_MODULE_NAME,
};

#[test]
fn reliable_udp_feature_registration_contributes_runtime_module_and_manager() {
    let report = plugin_feature_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert_eq!(report.manifest.id, NET_RELIABLE_UDP_FEATURE_ID);
    assert!(report
        .manifest
        .capabilities
        .iter()
        .any(|capability| capability == NET_RELIABLE_UDP_FEATURE_CAPABILITY));
    let module = report
        .extensions
        .modules()
        .iter()
        .find(|module| module.name == NET_RELIABLE_UDP_FEATURE_MODULE_NAME)
        .expect("reliable UDP feature module should be registered");
    assert_eq!(
        module.managers[0].name.to_string(),
        NET_RELIABLE_UDP_FEATURE_MANAGER_NAME
    );
}
