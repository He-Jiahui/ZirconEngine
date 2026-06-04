use crate::{
    plugin_feature_registration, NET_RPC_FEATURE_CAPABILITY, NET_RPC_FEATURE_ID,
    NET_RPC_FEATURE_MANAGER_NAME, NET_RPC_FEATURE_MODULE_NAME,
};

#[test]
fn rpc_feature_registration_contributes_runtime_module_and_manager() {
    let report = plugin_feature_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert_eq!(report.manifest.id, NET_RPC_FEATURE_ID);
    assert!(report
        .manifest
        .capabilities
        .iter()
        .any(|capability| capability == NET_RPC_FEATURE_CAPABILITY));
    let module = report
        .extensions
        .modules()
        .iter()
        .find(|module| module.name == NET_RPC_FEATURE_MODULE_NAME)
        .expect("RPC feature module should be registered");
    assert_eq!(
        module.managers[0].name.to_string(),
        NET_RPC_FEATURE_MANAGER_NAME
    );
}
