use crate::plugin::{PluginPackageManifest, PluginPackageRole, RuntimePluginCatalog};

#[test]
fn product_catalog_excludes_typed_test_roles_from_auto_selection() {
    let production = PluginPackageManifest::new("production", "Production");
    let fixture = PluginPackageManifest::new("fixture", "Fixture")
        .with_package_role(PluginPackageRole::TestFixture);
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [
            crate::plugin::RuntimePluginRegistrationReport::from_native_package_manifest(
                production,
            ),
            crate::plugin::RuntimePluginRegistrationReport::from_native_package_manifest(fixture),
        ],
        [],
    );

    let manifest = catalog.project_manifest();
    assert_eq!(
        manifest
            .selections
            .iter()
            .map(|selection| selection.id.as_str())
            .collect::<Vec<_>>(),
        ["production"]
    );
}
