#[test]
fn retained_host_asset_services_do_not_leak_a_generic_runtime_resolver() {
    let app_source = include_str!("../../ui/retained_host/app.rs");
    let assets_source = include_str!("../../ui/retained_host/app/assets.rs");
    let startup_managers_source =
        include_str!("../../ui/retained_host/app/host_lifecycle/startup/resources/managers.rs");
    let startup_bundle_source =
        include_str!("../../ui/retained_host/app/host_lifecycle/startup/resources/bundle.rs");
    let access_source = include_str!("../../ui/retained_host/app/asset_runtime_access.rs");

    for forbidden in [
        "resource_manager_resolver: ManagerResolver",
        "asset_manager: ManagerServiceHandle<dyn AssetManager>",
        "editor_asset_manager: ManagerServiceHandle<dyn EditorAssetManagerContract>",
        "resource_manager: ManagerServiceHandle<dyn ResourceManager>",
    ] {
        assert!(
            !app_source.contains(forbidden) && !startup_bundle_source.contains(forbidden),
            "retained-host state must not retain `{forbidden}` outside its typed asset access"
        );
    }
    assert!(
        !assets_source.contains("resource_manager_resolver")
            && !startup_managers_source.contains("ManagerResolver::new"),
        "asset callers and startup wiring must not recreate a generic manager resolver"
    );
    assert!(
        access_source.contains("struct RetainedHostAssetRuntimeAccess")
            && access_source.contains("resolver: ManagerResolver")
            && access_source.contains("fn asset_manager(")
            && access_source.contains("fn editor_asset_manager(")
            && access_source.contains("fn resource_manager(")
            && !access_source.contains("fn resolve<"),
        "the retained host must contain its generic resolver in a single named asset access leaf"
    );
}
