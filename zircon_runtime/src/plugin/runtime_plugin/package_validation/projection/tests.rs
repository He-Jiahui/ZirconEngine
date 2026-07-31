use crate::plugin::{PluginDependencyManifest, PluginModuleManifest, PluginPackageManifest};

use super::RuntimePluginPackageValidationProjection;

#[test]
fn projection_builds_once_and_capability_probes_scale_linearly() {
    for row_count in [1, 100, 1_000] {
        let manifest = (0..row_count).fold(
            PluginPackageManifest::new("projection", "Projection"),
            |manifest, index| manifest.with_capability(format!("projection.capability_{index}")),
        );
        let projection = RuntimePluginPackageValidationProjection::build(&manifest);

        for (index, capability) in manifest.capabilities.iter().enumerate() {
            assert!(!projection.package_capability_is_duplicate(index));
            assert!(projection.owns_capability(capability));
        }

        let metrics = projection.metrics();
        assert_eq!(metrics.projection_builds, 1);
        assert_eq!(metrics.identity_rows_indexed, row_count);
        assert_eq!(metrics.membership_probes, row_count * 2);
    }
}

#[test]
fn duplicate_ordinals_are_scoped_to_their_manifest_domain() {
    let mut manifest = PluginPackageManifest::new("projection", "Projection")
        .with_capability("projection.shared")
        .with_capability("projection.unique")
        .with_capability("projection.shared")
        .with_asset_roots(["shared", "assets", "shared"])
        .with_content_roots(["shared", "content", "shared"]);
    manifest.dependencies = vec![
        PluginDependencyManifest::new("provider", true).with_interface("projection.shared"),
        PluginDependencyManifest::new("provider", true).with_interface("projection.shared"),
    ];

    let projection = RuntimePluginPackageValidationProjection::build(&manifest);

    assert!(!projection.package_capability_is_duplicate(0));
    assert!(!projection.package_capability_is_duplicate(1));
    assert!(projection.package_capability_is_duplicate(2));
    assert!(!projection.asset_root_is_duplicate(0));
    assert!(!projection.asset_root_is_duplicate(1));
    assert!(projection.asset_root_is_duplicate(2));
    assert!(!projection.content_root_is_duplicate(0));
    assert!(!projection.content_root_is_duplicate(1));
    assert!(projection.content_root_is_duplicate(2));
    assert!(!projection.dependency_interface_is_duplicate(0, 0));
    assert!(!projection.dependency_interface_is_duplicate(1, 0));
}

#[test]
fn registration_projection_retains_manifest_order_and_membership() {
    let manifest = PluginPackageManifest::new("projection", "Projection")
        .with_runtime_module(PluginModuleManifest::runtime(
            "projection.first",
            "projection_first",
        ))
        .with_runtime_module(PluginModuleManifest::runtime(
            "projection.second",
            "projection_second",
        ))
        .with_provided_interface_id("projection.first.v1")
        .with_provided_interface_id("projection.second.v1")
        .with_dependency(
            PluginDependencyManifest::new("provider", true)
                .with_interfaces(["provider.first.v1", "provider.second.v1"]),
        );

    let projection = RuntimePluginPackageValidationProjection::build(&manifest);

    assert_eq!(
        projection.runtime_module_names().collect::<Vec<_>>(),
        ["projection.first", "projection.second"]
    );
    assert_eq!(
        projection.provided_interface_ids().collect::<Vec<_>>(),
        ["projection.first.v1", "projection.second.v1"]
    );
    assert_eq!(
        projection.dependency_interface_ids().collect::<Vec<_>>(),
        ["provider.first.v1", "provider.second.v1"]
    );
    assert!(projection.declares_provided_interface("projection.first.v1"));
    assert!(projection.declares_dependency_interface("provider.second.v1"));
}
