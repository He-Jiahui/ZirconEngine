use super::*;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{
    ExportPackagingStrategy, ProjectPluginManifest, ProjectPluginSelection,
};

#[test]
fn required_native_plugin_with_discovered_manifest_but_missing_library_is_rejected() {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock must be after Unix epoch")
        .as_nanos();
    let managed_target = PathBuf::from(
        std::env::var_os("CARGO_TARGET_DIR").expect("managed CARGO_TARGET_DIR is required"),
    )
    .canonicalize()
    .expect("managed Cargo target directory must exist");
    let export_root = managed_target.join(format!(
        "zircon-native-admission-negative-{}-{stamp}",
        std::process::id()
    ));
    let package_root = export_root.join("plugins/virtual_geometry");
    std::fs::create_dir_all(&package_root).expect("create isolated native package fixture");
    std::fs::write(
        package_root.join("plugin.toml"),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../zircon_plugins/virtual_geometry/plugin.toml"
        )),
    )
    .expect("stage the source-pinned VirtualGeometry manifest");
    std::fs::write(
        export_root.join("plugins/native_plugins.toml"),
        "[[plugins]]\nid = \"virtual_geometry\"\npath = \"plugins/virtual_geometry\"\nmanifest = \"plugins/virtual_geometry/plugin.toml\"\n",
    )
    .expect("write native load manifest");
    let discovered =
        zircon_runtime::plugin::native::discovery::load_native_runtime_from_load_manifest(
            &export_root,
        );
    assert!(
        discovered
            .runtime_plugin_registration_reports()
            .iter()
            .any(|report| report.package_manifest.id == "virtual_geometry"),
        "fixture must project the discovered manifest before testing admission"
    );

    let config = EntryConfig::new(crate::entry::EntryProfile::Runtime).with_project_plugins(
        ProjectPluginManifest {
            selections: vec![ProjectPluginSelection::runtime_plugin(
                "virtual_geometry",
                true,
                true,
            )
            .with_packaging(ExportPackagingStrategy::NativeDynamic)
            .with_target_modes([RuntimeTargetMode::ClientRuntime])],
        },
    );
    let metadata_only_result = ProductCompositionRequest::new(config.clone())
        .with_runtime_plugin_registrations(discovered.runtime_plugin_registration_reports())
        .module_selection_report();
    let result = ProductCompositionRequest::new(config)
        .with_native_plugins_from_export_root(&export_root)
        .module_selection_report();
    let optional_config = EntryConfig::new(crate::entry::EntryProfile::Runtime)
        .with_project_plugins(ProjectPluginManifest {
            selections: vec![ProjectPluginSelection::runtime_plugin(
                "virtual_geometry",
                true,
                false,
            )
            .with_packaging(ExportPackagingStrategy::NativeDynamic)
            .with_target_modes([RuntimeTargetMode::ClientRuntime])],
        });
    let optional_metadata_only = ProductCompositionRequest::new(optional_config.clone())
        .with_runtime_plugin_registrations(discovered.runtime_plugin_registration_reports())
        .module_selection_report()
        .expect("an optional missing native plugin cannot block the product");
    let optional_with_export_root = ProductCompositionRequest::new(optional_config)
        .with_runtime_plugin_registrations(discovered.runtime_plugin_registration_reports())
        .with_native_plugins_from_export_root(&export_root)
        .module_selection_report()
        .expect("an optional missing native library cannot block the export product");
    std::fs::remove_dir_all(&export_root).expect("remove only the isolated fixture");

    for report in [&optional_metadata_only, &optional_with_export_root] {
        assert!(!report.runtime_plugin_availability.contains(
            zircon_runtime::plugin::RuntimePluginAvailabilityCategory::NativeDynamic,
            zircon_runtime::builtin::RuntimePluginId::VirtualGeometry,
        ));
        assert!(!report.module_keys().contains(&"virtual_geometry.runtime"));
    }

    let metadata_error = metadata_only_result.expect_err(
        "injected discovery metadata without a native host cannot admit a required plugin",
    );
    assert!(
        metadata_error
            .to_string()
            .contains("required native plugin virtual_geometry was not admitted"),
        "{metadata_error}"
    );
    let error =
        result.expect_err("manifest metadata without a DLL cannot admit a required native plugin");
    assert!(
        error
            .to_string()
            .contains("required native plugin virtual_geometry was not admitted"),
        "{error}"
    );
}

#[test]
fn required_native_plugin_rejection_prevents_product_preparation() {
    let config = EntryConfig::new(crate::entry::EntryProfile::Headless).with_project_plugins(
        ProjectPluginManifest {
            selections: vec![
                ProjectPluginSelection::runtime_plugin("missing_native", true, true)
                    .with_packaging(ExportPackagingStrategy::NativeDynamic),
            ],
        },
    );
    let request = ProductCompositionRequest::new(config).with_native_plugins_from_export_root(
        std::env::temp_dir().join(format!("zircon-missing-native-{}", std::process::id())),
    );
    let error = request
        .module_selection_report()
        .expect_err("required native admission must fail before bootstrap");
    assert!(
        error
            .to_string()
            .contains("required native plugin missing_native was not admitted"),
        "{error}"
    );
}

#[test]
fn invalid_config_file_path_precedes_native_admission_branches() {
    let config = EntryConfig::new(crate::entry::EntryProfile::Headless).with_project_plugins(
        ProjectPluginManifest {
            selections: vec![
                ProjectPluginSelection::runtime_plugin("missing_native", true, true)
                    .with_packaging(ExportPackagingStrategy::NativeDynamic),
            ],
        },
    );
    for export_root in [
        None,
        Some(std::env::temp_dir().join("zircon-native-preflight-unused-root")),
    ] {
        let mut request = ProductCompositionRequest::new(config.clone())
            .with_config_file_path(PathBuf::from("relative-config.json"));
        if let Some(export_root) = export_root {
            request = request.with_native_plugins_from_export_root(export_root);
        }
        let error = request
            .module_selection_report()
            .expect_err("invalid config path must precede native admission");
        assert!(
            error.to_string().contains("nonempty and absolute"),
            "{error}"
        );
    }
}
