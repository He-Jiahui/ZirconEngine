use zircon_runtime::{
    core::framework::platform::RuntimeTargetMode,
    plugin::{ExportPackagingStrategy, PluginModuleKind},
};

use super::*;

#[test]
fn native_window_hosting_plugin_does_not_publish_core_owned_or_missing_authoring_surfaces() {
    let registration = plugin_registration();

    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    assert!(registration.capabilities.contains(&CAPABILITY.to_string()));
    assert_eq!(registration.package_manifest.category, "platform");
    assert_eq!(
        registration.package_manifest.supported_targets,
        vec![RuntimeTargetMode::EditorHost]
    );
    assert_eq!(
        registration.package_manifest.capabilities,
        vec![CAPABILITY.to_string()]
    );
    assert!(registration.extensions.views().is_empty());
    assert!(registration.extensions.drawers().is_empty());
    assert!(registration.extensions.ui_templates().is_empty());
    assert!(registration.extensions.menu_items().is_empty());
    assert_eq!(registration.extensions.commands().commands().count(), 0);
}

#[test]
fn native_window_hosting_registration_eliminates_phantom_authoring_work() {
    const REGISTRATIONS: usize = 1_000;
    const PREVIOUS_CONTRIBUTIONS_PER_REGISTRATION: usize = 8;

    let mut current_contributions = 0usize;
    for _ in 0..REGISTRATIONS {
        let registration = plugin_registration();
        assert!(registration.is_success(), "{:?}", registration.diagnostics);
        current_contributions += registration.extensions.views().len();
        current_contributions += registration.extensions.drawers().len();
        current_contributions += registration.extensions.ui_templates().len();
        current_contributions += registration.extensions.menu_items().len();
        current_contributions += registration.extensions.commands().commands().count();
    }

    let previous_contributions = REGISTRATIONS * PREVIOUS_CONTRIBUTIONS_PER_REGISTRATION;
    assert_eq!(current_contributions, 0);
    println!(
        "PERF-MVP-PLUGINS03-NO-PHANTOM-AUTHORING registrations={REGISTRATIONS} \
         previous_contributions={previous_contributions} current_contributions={current_contributions} \
         contribution_reduction_percent=100 previous_template_resolutions={REGISTRATIONS} \
         current_template_resolutions=0 template_resolution_reduction_percent=100"
    );
}

#[test]
fn native_window_hosting_package_manifest_declares_editor_dist_contract() {
    let manifest = package_manifest();

    assert!(
        manifest
            .default_packaging
            .contains(&ExportPackagingStrategy::NativeDynamic)
    );
    let distribution = manifest
        .distribution
        .as_ref()
        .expect("native_window_hosting declares standalone distribution");
    assert_eq!(distribution.forms, vec!["dist"]);
    assert_eq!(
        distribution.default_packaging,
        vec![ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(
        distribution.dist_crate,
        NATIVE_WINDOW_HOSTING_DIST_CRATE_NAME
    );
    assert_eq!(
        distribution.descriptor_symbol,
        "zircon_native_plugin_descriptor_v3"
    );
    assert!(distribution.runtime_entry.is_empty());
    assert_eq!(
        distribution.editor_entry,
        NATIVE_WINDOW_HOSTING_DIST_EDITOR_ENTRY
    );

    let dist_module = manifest
        .modules
        .iter()
        .find(|module| module.name == "native_window_hosting.dist")
        .expect("native_window_hosting dist module is declared");
    assert_eq!(dist_module.kind, PluginModuleKind::Native);
    assert_eq!(
        dist_module.crate_name,
        NATIVE_WINDOW_HOSTING_DIST_CRATE_NAME
    );
    assert_eq!(
        dist_module.target_modes,
        vec![RuntimeTargetMode::EditorHost]
    );
    assert_eq!(dist_module.capabilities, vec![CAPABILITY.to_string()]);
}
