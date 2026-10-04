use zircon_runtime::plugin::{PluginPackageManifest, PluginPackageRole};

use super::product_native_package;

#[test]
fn project_cannot_enable_a_selected_test_carrier_as_a_product_plugin() {
    let packages = [
        PluginPackageManifest::new("production", "Production"),
        PluginPackageManifest::new("fixture", "Fixture")
            .with_package_role(PluginPackageRole::TestFixture),
    ];

    assert!(product_native_package(&packages, "production").is_some());
    assert!(product_native_package(&packages, "fixture").is_none());
    assert!(product_native_package(&packages, "missing").is_none());
}
