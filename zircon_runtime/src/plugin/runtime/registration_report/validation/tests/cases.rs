use crate::asset::{AssetImporterDescriptor, AssetKind};
use crate::core::framework::project::ExportPackagingStrategy;
use crate::core::framework::scene::ComponentTypeDescriptor;
use crate::plugin::{
    CapabilityStatus, CapabilityStatusManifest, PluginDependencyManifest,
    PluginEventCatalogManifest, PluginFeatureBundleManifest, PluginFeatureDependency,
    PluginInterfaceManifest, PluginInterfaceMethodManifest, PluginModuleManifest,
    PluginOptionManifest, PluginPackageManifest, UiComponentDescriptor,
};

use super::validate_runtime_plugin_package_manifest_with_stats;

#[test]
fn package_projection_preserves_duplicate_diagnostic_bytes_and_order() {
    let manifest = PluginPackageManifest::new("weather", "Weather")
        .with_capability("runtime.plugin.weather")
        .with_capability("runtime.plugin.weather")
        .with_default_packaging([ExportPackagingStrategy::SourceTemplate])
        .with_provided_interface(PluginInterfaceManifest::new("weather.query.v1"))
        .with_provided_interface(PluginInterfaceManifest::new("weather.query.v1"));
    let mut diagnostics = Vec::new();

    let stats =
        validate_runtime_plugin_package_manifest_with_stats(None, &manifest, &mut diagnostics);

    let duplicates = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.contains("must be unique"))
        .map(String::as_str)
        .collect::<Vec<_>>();
    assert_eq!(
        duplicates,
        [
            "runtime plugin package manifest capability `runtime.plugin.weather` must be unique",
            "runtime plugin package manifest provided interface `weather.query.v1` must be unique",
        ]
    );
    assert_eq!(stats.projection_builds, 1);
}

#[test]
fn complete_package_validation_builds_one_linear_projection() {
    for row_count in [1, 100, 1_000] {
        let manifest = (0..row_count).fold(
            PluginPackageManifest::new("validation", "Validation"),
            append_projection_fixture_row,
        );
        let mut diagnostics = Vec::new();

        let stats =
            validate_runtime_plugin_package_manifest_with_stats(None, &manifest, &mut diagnostics);

        assert_eq!(stats.projection_builds, 1);
        assert_eq!(stats.standalone_feature_projection_builds, 0);
        assert_eq!(stats.embedded_feature_projection_views, row_count);
        assert_eq!(stats.identity_rows_indexed, row_count * 26);
        assert_eq!(stats.membership_probes, row_count * 27);
        assert!(diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.contains("must be unique")));
    }
}

fn append_projection_fixture_row(
    manifest: PluginPackageManifest,
    index: usize,
) -> PluginPackageManifest {
    let package_capability = format!("validation.capability_{index}");
    let dependency_capability = format!("validation.dependency_{index}");
    let dependency_interface = format!("validation.provider_{index}.v1");
    let feature_id = format!("validation.feature_{index}");
    let feature_capability = format!("{feature_id}.capability");
    let feature_module = PluginModuleManifest::runtime(
        format!("{feature_id}.runtime"),
        format!("validation_feature_{index}_runtime"),
    )
    .with_capabilities([format!("{feature_capability}.module")]);
    let feature = PluginFeatureBundleManifest::new(
        &feature_id,
        format!("Validation Feature {index}"),
        "validation",
    )
    .with_provider_package_id("validation")
    .with_capability(&feature_capability)
    .with_dependency(PluginFeatureDependency::primary(
        format!("validation_provider_{index}"),
        format!("{dependency_capability}.feature"),
    ))
    .with_runtime_module(feature_module);
    let package_module = PluginModuleManifest::runtime(
        format!("validation.module_{index}.runtime"),
        format!("validation_module_{index}_runtime"),
    )
    .with_capabilities([format!("{package_capability}.module")])
    .with_system_sets([format!("validation.system_set_{index}")])
    .with_system_anchors([format!("validation.system_anchor_{index}")]);
    let provided_interface =
        PluginInterfaceManifest::new(format!("validation.interface_{index}.v1")).with_method(
            PluginInterfaceMethodManifest::new(format!("method_{index}"), index as u32)
                .with_required_capability(&package_capability),
        );
    let importer = AssetImporterDescriptor::new(
        format!("validation.importer_{index}"),
        "validation",
        AssetKind::Data,
        1,
    )
    .with_source_extensions([format!("validation_{index}")])
    .with_required_capabilities([package_capability.clone()]);

    manifest
        .with_capability(&package_capability)
        .with_asset_root(format!("assets/validation_{index}"))
        .with_content_root(format!("content/validation_{index}"))
        .with_dependency(
            PluginDependencyManifest::new(format!("validation_provider_{index}"), true)
                .with_capability(dependency_capability)
                .with_interface(dependency_interface),
        )
        .with_provided_interface(provided_interface)
        .with_option(PluginOptionManifest::new(
            format!("validation.option_{index}"),
            format!("Validation Option {index}"),
            "bool",
            "false",
        ))
        .with_event_catalog(PluginEventCatalogManifest::empty(
            format!("validation.events_{index}"),
            1,
        ))
        .with_component(ComponentTypeDescriptor {
            type_id: format!("validation.component_{index}"),
            plugin_id: "validation".to_string(),
            display_name: format!("Validation Component {index}"),
            properties: Vec::new(),
        })
        .with_ui_component(UiComponentDescriptor::new(
            format!("validation.ui_{index}"),
            "validation",
            format!("res://ui/validation_{index}.ui"),
        ))
        .with_asset_importer(importer)
        .with_optional_feature(feature)
        .with_capability_status(
            CapabilityStatusManifest::new(package_capability, CapabilityStatus::Complete)
                .with_bevy_reference(format!("bevy::validation_{index}")),
        )
        .with_module(package_module)
}
