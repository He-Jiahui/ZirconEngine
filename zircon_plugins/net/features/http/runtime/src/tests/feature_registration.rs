use crate::{
    module_descriptor, plugin_feature_registration, NET_HTTP_FEATURE_CAPABILITY,
    NET_HTTP_FEATURE_ID, NET_HTTP_FEATURE_MANAGER_NAME, NET_HTTP_FEATURE_MODULE_NAME,
};

#[test]
fn http_feature_registration_contributes_runtime_module_and_manager() {
    let report = plugin_feature_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert_eq!(report.manifest.id, NET_HTTP_FEATURE_ID);
    assert!(report
        .manifest
        .capabilities
        .iter()
        .any(|capability| capability == NET_HTTP_FEATURE_CAPABILITY));
    let module = report
        .extensions
        .modules()
        .iter()
        .find(|module| module.name == NET_HTTP_FEATURE_MODULE_NAME)
        .expect("HTTP feature module should be registered");
    assert_eq!(
        module.managers[0].name.to_string(),
        NET_HTTP_FEATURE_MANAGER_NAME
    );
}

#[test]
fn http_feature_manager_is_the_canonical_net_manager_authority() {
    let runtime = zircon_runtime::core::CoreRuntime::new();
    runtime
        .register_module(zircon_plugin_net_runtime::module_descriptor())
        .unwrap();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_registered_modules().unwrap();

    let canonical = runtime
        .resolve_manager::<zircon_plugin_net_runtime::DefaultNetManager>(
            zircon_plugin_net_runtime::DEFAULT_NET_MANAGER_NAME,
        )
        .unwrap();
    assert!(
        zircon_runtime::core::framework::net::NetManager::backend_name(canonical.as_ref())
            .contains("+http"),
        "module activation must install HTTP before the feature manager is resolved"
    );
    let feature = runtime
        .resolve_manager::<zircon_plugin_net_runtime::DefaultNetManager>(
            NET_HTTP_FEATURE_MANAGER_NAME,
        )
        .unwrap();

    assert!(std::sync::Arc::ptr_eq(&canonical, &feature));
    assert!(
        zircon_runtime::core::framework::net::NetManager::backend_name(canonical.as_ref())
            .contains("+http")
    );
}
