use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{
    ExportPackagingStrategy, ProjectPluginManifest, ProjectPluginSelection,
};
use zircon_runtime::plugin::{PluginPackageManifest, PluginPackageRole};

use super::{missing_native_selection_statuses, product_native_packages};

#[test]
fn missing_optional_native_selection_has_an_explicit_status_result() {
    let selections = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin("missing.native", true, false)
                .with_packaging(ExportPackagingStrategy::NativeDynamic),
            ProjectPluginSelection::runtime_plugin("required.native", true, true)
                .with_packaging(ExportPackagingStrategy::NativeDynamic),
            ProjectPluginSelection::runtime_plugin("known.native", true, false)
                .with_packaging(ExportPackagingStrategy::NativeDynamic),
            ProjectPluginSelection::runtime_plugin("fixture.native", true, false)
                .with_packaging(ExportPackagingStrategy::NativeDynamic),
            ProjectPluginSelection::runtime_plugin("disabled.native", false, false)
                .with_packaging(ExportPackagingStrategy::NativeDynamic),
            ProjectPluginSelection::runtime_plugin("client.only", true, false)
                .with_packaging(ExportPackagingStrategy::NativeDynamic)
                .with_target_modes([RuntimeTargetMode::ClientRuntime]),
        ],
    };
    let discovered = [
        PluginPackageManifest::new("known.native", "Known"),
        PluginPackageManifest::new("fixture.native", "Fixture")
            .with_package_role(PluginPackageRole::TestFixture),
    ];
    let statuses =
        missing_native_selection_statuses(&selections, &discovered, RuntimeTargetMode::EditorHost);

    assert_eq!(statuses.len(), 2);
    assert_eq!(statuses[0].plugin_id, "missing.native");
    assert_eq!(statuses[0].load_state, "missing package");
    assert!(!statuses[0].required);
    assert_eq!(
        statuses[0].packaging,
        ExportPackagingStrategy::NativeDynamic
    );
    assert!(statuses[0].diagnostics[0].contains("missing.native"));
    assert_eq!(statuses[1].plugin_id, "required.native");
    assert!(statuses[1].required);
}

#[test]
fn status_catalog_never_projects_sample_or_test_carriers() {
    let packages = [
        PluginPackageManifest::new("production", "Production"),
        PluginPackageManifest::new("tool", "Tool")
            .with_package_role(PluginPackageRole::DeveloperTool),
        PluginPackageManifest::new("sample", "Sample").with_package_role(PluginPackageRole::Sample),
        PluginPackageManifest::new("fixture", "Fixture")
            .with_package_role(PluginPackageRole::TestFixture),
    ];
    let product = product_native_packages(&packages);

    assert_eq!(
        product
            .iter()
            .map(|package| package.id.as_str())
            .collect::<Vec<_>>(),
        ["production", "tool"]
    );
}
