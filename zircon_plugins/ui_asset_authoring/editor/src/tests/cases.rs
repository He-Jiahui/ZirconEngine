use super::*;
use zircon_runtime::{
    core::framework::platform::RuntimeTargetMode,
    plugin::{ExportPackagingStrategy, PluginModuleKind},
};

#[test]
fn ui_asset_authoring_plugin_contributes_view_template_and_capability() {
    let registration = plugin_registration();

    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    assert!(registration.capabilities.contains(&CAPABILITY.to_string()));
    assert_eq!(registration.package_manifest.category, "authoring");
    assert_eq!(
        registration.package_manifest.supported_targets,
        vec![zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost]
    );
    assert_eq!(
        registration.package_manifest.capabilities,
        vec![CAPABILITY.to_string()]
    );
    assert!(registration
        .extensions
        .views()
        .iter()
        .any(|view| view.id() == UI_ASSET_VIEW_ID));
    assert!(registration
        .extensions
        .ui_templates()
        .iter()
        .any(|template| template.id() == UI_ASSET_TEMPLATE_ID));
    assert!(registration
        .extensions
        .drawers()
        .iter()
        .any(|drawer| drawer.id() == UI_ASSET_DRAWER_ID));
    assert!(registration
        .extensions
        .menu_items()
        .iter()
        .any(|menu| menu.operation().as_str() == "view.editor.ui_asset.open"));
    assert!(registration
        .extensions
        .pending_commands()
        .any(|operation| operation.id().as_str() == "view.editor.ui_asset.open"));
    assert_eq!(registration.extensions.asset_type_contributions().len(), 3);
    for kind in [
        zircon_runtime_interface::resource::ResourceKind::UiLayout,
        zircon_runtime_interface::resource::ResourceKind::UiWidget,
        zircon_runtime_interface::resource::ResourceKind::UiStyle,
    ] {
        let asset_type = zircon_editor::core::asset::AssetTypeId::from_resource_kind(kind);
        let contribution = registration
            .extensions
            .asset_type_contributions()
            .into_iter()
            .find(|contribution| contribution.asset_type() == &asset_type)
            .expect("UI asset type contribution");
        assert_eq!(contribution.toolkit().unwrap().view_id(), UI_ASSET_VIEW_ID);
        assert_eq!(contribution.creation_templates().len(), 1);
        assert_eq!(contribution.context_commands().len(), 1);
        assert_eq!(
            contribution.context_commands()[0].operation().as_str(),
            "view.editor.ui_asset.open"
        );
    }
}

#[test]
fn ui_asset_authoring_package_manifest_declares_editor_dist_contract() {
    let manifest = package_manifest();

    assert!(manifest
        .default_packaging
        .contains(&ExportPackagingStrategy::NativeDynamic));
    let distribution = manifest
        .distribution
        .as_ref()
        .expect("ui_asset_authoring declares standalone distribution");
    assert_eq!(distribution.forms, vec!["dist"]);
    assert_eq!(
        distribution.default_packaging,
        vec![ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(distribution.dist_crate, UI_ASSET_AUTHORING_DIST_CRATE_NAME);
    assert_eq!(
        distribution.descriptor_symbol,
        "zircon_native_plugin_descriptor_v3"
    );
    assert!(distribution.runtime_entry.is_empty());
    assert_eq!(
        distribution.editor_entry,
        UI_ASSET_AUTHORING_DIST_EDITOR_ENTRY
    );

    let dist_module = manifest
        .modules
        .iter()
        .find(|module| module.name == "ui_asset_authoring.dist")
        .expect("ui_asset_authoring dist module is declared");
    assert_eq!(dist_module.kind, PluginModuleKind::Native);
    assert_eq!(dist_module.crate_name, UI_ASSET_AUTHORING_DIST_CRATE_NAME);
    assert_eq!(
        dist_module.target_modes,
        vec![RuntimeTargetMode::EditorHost]
    );
    assert_eq!(dist_module.capabilities, vec![CAPABILITY.to_string()]);
}
