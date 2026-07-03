#[test]
fn review_f8_runtime_plugin_descriptor_test_fixtures_use_builder() {
    let fixture_sources = [
        (
            "asset_importer_install",
            include_str!("../../../../plugin_extensions/asset_importer_install.rs"),
        ),
        (
            "extension_registry",
            include_str!("../../../../plugin_extensions/extension_registry.rs"),
        ),
        (
            "extension_registry_components",
            include_str!("../../../../plugin_extensions/extension_registry_components.rs"),
        ),
        (
            "extension_registry_event_catalogs",
            include_str!("../../../../plugin_extensions/extension_registry_event_catalogs.rs"),
        ),
        (
            "extension_registry_features",
            include_str!("../../../../plugin_extensions/extension_registry_features.rs"),
        ),
        (
            "extension_registry_managers",
            include_str!("../../../../plugin_extensions/extension_registry_managers.rs"),
        ),
        (
            "extension_registry_metadata",
            include_str!("../../../../plugin_extensions/extension_registry_metadata.rs"),
        ),
        (
            "extension_registry_modules",
            include_str!("../../../../plugin_extensions/extension_registry_modules.rs"),
        ),
        (
            "extension_registry_options",
            include_str!("../../../../plugin_extensions/extension_registry_options.rs"),
        ),
        (
            "profile_maturity",
            include_str!("../../../../plugin_extensions/profile_maturity.rs"),
        ),
        (
            "runtime_plugin_catalog_features",
            include_str!("../../../../plugin_extensions/runtime_plugin_catalog_features.rs"),
        ),
        (
            "runtime_plugin_descriptor",
            include_str!("../../../../plugin_extensions/runtime_plugin_descriptor.rs"),
        ),
        (
            "runtime_plugin_lifecycle",
            include_str!("../../../../plugin_extensions/runtime_plugin_lifecycle.rs"),
        ),
        (
            "runtime_plugin_package_manifest",
            include_str!("../../../../plugin_extensions/runtime_plugin_package_manifest.rs"),
        ),
    ];
    assert_eq!(
        fixture_sources.len(),
        14,
        "F8 RuntimePluginDescriptor fixture migration should enumerate every plugin extension test file that used the old constructor"
    );

    let builder_count: usize = fixture_sources
        .iter()
        .map(|(_, source)| {
            source
                .match_indices("RuntimePluginDescriptor::builder(")
                .count()
        })
        .sum();
    assert_eq!(
        builder_count, 59,
        "F8 runtime/plugin extension test fixtures should keep the current builder call count"
    );

    for (name, source) in fixture_sources {
        assert!(
            !source.contains("RuntimePluginDescriptor::new("),
            "plugin extension test fixture `{name}` should not keep RuntimePluginDescriptor::new"
        );
        assert!(
            source.contains("RuntimePluginDescriptor::builder("),
            "plugin extension test fixture `{name}` should construct RuntimePluginDescriptor values through the builder"
        );
    }

    let review_findings =
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_06_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../../docs/plans/engine-code-structure-convention.md");
    let package_manifest_doc =
        include_str!("../../../../../../../docs/zircon_runtime/plugin/package_manifest.md");

    for doc_anchor in [
        "F8 RuntimePluginDescriptor test fixture builder migration",
        "runtime_plugin_descriptor_test_fixture_builder_migration_coremin_check_passed",
        "review_f8_runtime_plugin_descriptor_test_fixtures_use_builder",
        "plugin extension RuntimePluginDescriptor test fixtures 14/14",
        "RuntimePluginDescriptor public-field convergence complete",
        "RuntimePluginDescriptor::new retired",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_06_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || package_manifest_doc.contains(doc_anchor),
            "RuntimePluginDescriptor test fixture builder migration docs should record `{doc_anchor}`"
        );
    }
}
