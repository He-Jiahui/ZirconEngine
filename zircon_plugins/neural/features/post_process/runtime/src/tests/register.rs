use super::*;

#[test]
fn neural_post_process_registration_delegates_to_the_runtime_manifest_owner() {
    let registration = plugin_feature_registration();

    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    assert_eq!(
        registration.manifest,
        zircon_plugin_neural_runtime::neural_post_process_feature_manifest()
    );
    assert_eq!(registration.manifest.id, FEATURE_ID);
    assert_eq!(registration.manifest.capabilities, [RUNTIME_CAPABILITY]);
    assert!(!registration.manifest.enabled_by_default);
    assert_eq!(
        zircon_plugin_neural_runtime::RENDERING_POST_PROCESS_RUNTIME_CAPABILITY,
        zircon_plugin_rendering_post_process_runtime::RUNTIME_CAPABILITY
    );
    assert!(registration.extensions.modules().is_empty());
    assert!(registration.extensions.managers().is_empty());
    assert!(registration.extensions.shader_module_sources().is_empty());
    assert!(registration.extensions.asset_importers().is_empty());
    assert!(registration.extensions.components().is_empty());
    assert!(registration.extensions.plugin_options().is_empty());
    assert!(registration.extensions.plugin_event_catalogs().is_empty());
    assert!(registration.extensions.scene_hooks().is_empty());
    assert!(registration.extensions.render_features().is_empty());
    assert!(registration.extensions.render_pass_executors().is_empty());
    assert!(registration
        .extensions
        .runtime_prepare_collectors()
        .is_empty());
    assert!(registration
        .extensions
        .hybrid_gi_runtime_providers()
        .is_empty());
    assert!(registration
        .extensions
        .solari_runtime_providers()
        .is_empty());
    assert!(registration
        .extensions
        .virtual_geometry_runtime_providers()
        .is_empty());
    assert!(registration.extensions.geometry_sources().is_empty());
    assert!(registration.extensions.shading_models().is_empty());
}
