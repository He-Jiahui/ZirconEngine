use super::product_install_role_is_allowed;
use crate::plugin::{PluginPackageManifest, PluginPackageRole};

#[test]
fn package_install_excludes_test_and_sample_carriers() {
    for role in [PluginPackageRole::TestFixture, PluginPackageRole::Sample] {
        let manifest = PluginPackageManifest::new("fixture", "Fixture").with_package_role(role);
        assert!(!product_install_role_is_allowed(&manifest));
    }

    let production = PluginPackageManifest::new("product", "Product");
    assert!(product_install_role_is_allowed(&production));
}
