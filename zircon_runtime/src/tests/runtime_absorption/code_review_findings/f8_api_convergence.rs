#[test]
fn review_f8_texture_import_settings_use_fallible_apply_not_with() {
    let descriptor = include_str!("../../../asset/assets/texture/descriptor.rs");
    let texture_asset = include_str!("../../../asset/assets/texture/texture_asset.rs");
    let runtime_importer = include_str!("../../../asset/importer/ingest/import_texture.rs");
    let plugin_importer =
        include_str!("../../../../../zircon_plugins/texture_importer/runtime/src/importers.rs");
    let review_findings =
        include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_04_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention = include_str!("../../../../../docs/plans/engine-code-structure-convention.md");
    let importer_doc = include_str!("../../../../../docs/zircon_runtime/asset/importer.md");
    let render_asset_doc =
        include_str!("../../../../../docs/zircon_runtime/asset/render-assets.md");

    let old_fallible_with_name = ["with", "import", "settings"].join("_");
    for (name, source) in [
        ("TextureAssetDescriptor", descriptor),
        ("TextureAsset", texture_asset),
        ("runtime texture importer", runtime_importer),
        ("texture importer plugin", plugin_importer),
    ] {
        assert!(
            source.contains("apply_import_settings"),
            "F8 texture import settings source `{name}` should use the fallible apply_* API"
        );
        assert!(
            !source.contains(&old_fallible_with_name),
            "F8 texture import settings source `{name}` should not keep the fallible with_* API"
        );
    }
    assert!(
        descriptor
            .contains("pub fn apply_import_settings(mut self, settings: &toml::Table) -> Result<Self, String>")
            && texture_asset.contains(
                "pub fn apply_import_settings(mut self, settings: &toml::Table) -> Result<Self, String>"
            ),
        "Texture import settings should remain fallible but no longer use a builder-style with_* verb"
    );
    assert!(
        texture_asset.contains(".apply_import_settings(settings)?")
            && runtime_importer.contains(".apply_import_settings(&context.import_settings)")
            && plugin_importer.contains(".apply_import_settings(&context.import_settings)"),
        "Runtime and plugin importers should call the fallible apply_import_settings entry"
    );

    for doc_anchor in [
        "F8 texture import settings apply API",
        "texture_import_settings_apply_api_coremin_check_passed",
        "review_f8_texture_import_settings_use_fallible_apply_not_with",
        "apply_import_settings",
        "RuntimePluginDescriptor public-field convergence remains pending",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_04_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || importer_doc.contains(doc_anchor)
                || render_asset_doc.contains(doc_anchor),
            "F8 texture import settings docs should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f8_runtime_plugin_descriptor_exposes_builder_scaffold() {
    let descriptor = include_str!("../../../plugin/runtime_plugin/descriptor.rs");
    let builder_mod = include_str!("../../../plugin/runtime_plugin/descriptor/builder.rs");
    let builder_source = include_str!(
        "../../../plugin/runtime_plugin/descriptor/builder/runtime_plugin_descriptor_builder.rs"
    );
    let runtime_plugin_mod = include_str!("../../../plugin/runtime_plugin/mod.rs");
    let plugin_mod = include_str!("../../../plugin/mod.rs");
    let plugin_descriptor_tests =
        include_str!("../../../tests/plugin_extensions/runtime_plugin_descriptor.rs");
    let review_findings =
        include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_06_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention = include_str!("../../../../../docs/plans/engine-code-structure-convention.md");
    let package_manifest_doc =
        include_str!("../../../../../docs/zircon_runtime/plugin/package_manifest.md");

    assert!(
        descriptor.contains("pub use builder::RuntimePluginDescriptorBuilder;"),
        "RuntimePluginDescriptor should re-export RuntimePluginDescriptorBuilder from its descriptor owner"
    );
    assert!(
        builder_mod.contains("mod runtime_plugin_descriptor_builder;")
            && builder_mod.contains(
                "pub use runtime_plugin_descriptor_builder::RuntimePluginDescriptorBuilder;"
            ),
        "descriptor builder module should export the RuntimePluginDescriptorBuilder owner"
    );
    for required in [
        "pub struct RuntimePluginDescriptorBuilder",
        "descriptor: RuntimePluginDescriptor",
        "pub fn builder(",
        ") -> RuntimePluginDescriptorBuilder",
        "pub fn build(self) -> RuntimePluginDescriptor",
        "with_optional_feature",
        "with_default_packaging",
    ] {
        assert!(
            builder_source.contains(required),
            "RuntimePluginDescriptorBuilder source should contain `{required}`"
        );
    }
    assert!(
        runtime_plugin_mod
            .contains("pub use descriptor::{RuntimePluginDescriptor, RuntimePluginDescriptorBuilder};")
            && plugin_mod.contains("RuntimePluginDescriptorBuilder,"),
        "RuntimePluginDescriptorBuilder should be exported through runtime_plugin and plugin facades"
    );
    assert!(
        plugin_descriptor_tests
            .contains("runtime_plugin_descriptor_builder_matches_fluent_descriptor_projection")
            && plugin_descriptor_tests.contains("RuntimePluginDescriptor::builder(")
            && plugin_descriptor_tests.contains(".build()"),
        "RuntimePluginDescriptor builder behavior should stay covered by focused plugin descriptor tests"
    );

    for doc_anchor in [
        "F8 RuntimePluginDescriptor builder scaffold",
        "runtime_plugin_descriptor_builder_scaffold_coremin_check_passed",
        "review_f8_runtime_plugin_descriptor_exposes_builder_scaffold",
        "RuntimePluginDescriptorBuilder",
        "RuntimePluginDescriptor public-field convergence remains pending",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_06_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || package_manifest_doc.contains(doc_anchor),
            "RuntimePluginDescriptor builder docs should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f8_first_party_runtime_plugin_descriptors_use_builder() {
    let plugin_sources = [
        (
            "ai",
            include_str!("../../../../../zircon_plugins/ai/runtime/src/lib.rs"),
        ),
        (
            "animation",
            include_str!("../../../../../zircon_plugins/animation/runtime/src/lib.rs"),
        ),
        (
            "hybrid_gi",
            include_str!("../../../../../zircon_plugins/hybrid_gi/runtime/src/lib.rs"),
        ),
        (
            "navigation",
            include_str!("../../../../../zircon_plugins/navigation/runtime/src/lib.rs"),
        ),
        (
            "net",
            include_str!("../../../../../zircon_plugins/net/runtime/src/lib.rs"),
        ),
        (
            "particles",
            include_str!("../../../../../zircon_plugins/particles/runtime/src/lib.rs"),
        ),
        (
            "physics",
            include_str!("../../../../../zircon_plugins/physics/runtime/src/lib.rs"),
        ),
        (
            "prefab_tools",
            include_str!("../../../../../zircon_plugins/prefab_tools/runtime/src/lib.rs"),
        ),
        (
            "rendering",
            include_str!("../../../../../zircon_plugins/rendering/runtime/src/lib.rs"),
        ),
        (
            "solari",
            include_str!("../../../../../zircon_plugins/solari/runtime/src/lib.rs"),
        ),
        (
            "sound",
            include_str!(
                "../../../../../zircon_plugins/sound/runtime/src/runtime_plugin/descriptor.rs"
            ),
        ),
        (
            "terrain",
            include_str!("../../../../../zircon_plugins/terrain/runtime/src/lib.rs"),
        ),
        (
            "texture",
            include_str!("../../../../../zircon_plugins/texture/runtime/src/lib.rs"),
        ),
        (
            "tilemap_2d",
            include_str!("../../../../../zircon_plugins/tilemap_2d/runtime/src/lib.rs"),
        ),
        (
            "virtual_geometry",
            include_str!("../../../../../zircon_plugins/virtual_geometry/runtime/src/lib.rs"),
        ),
        (
            "zr_vm_language",
            include_str!("../../../../../zircon_plugins/zr_vm_language/runtime/src/lib.rs"),
        ),
    ];
    assert_eq!(
        plugin_sources.len(),
        16,
        "F8 first-party runtime plugin descriptor migration should enumerate every production runtime plugin"
    );

    for (name, source) in plugin_sources {
        assert!(
            source.contains("RuntimePluginDescriptor::builder("),
            "first-party runtime plugin `{name}` should construct descriptors through the builder"
        );
        assert!(
            !source.contains("RuntimePluginDescriptor::new("),
            "first-party runtime plugin `{name}` should not keep the old descriptor constructor"
        );
        assert!(
            source.contains(".build()"),
            "first-party runtime plugin `{name}` should finish descriptor construction with build()"
        );
    }

    let review_findings =
        include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_06_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention = include_str!("../../../../../docs/plans/engine-code-structure-convention.md");
    let package_manifest_doc =
        include_str!("../../../../../docs/zircon_runtime/plugin/package_manifest.md");
    let first_party_catalog_doc =
        include_str!("../../../../../docs/zircon_plugins/first_party_runtime_catalog.md");

    for doc_anchor in [
        "F8 first-party RuntimePluginDescriptor builder migration",
        "runtime_plugin_descriptor_first_party_builder_migration_coremin_check_passed",
        "review_f8_first_party_runtime_plugin_descriptors_use_builder",
        "first-party runtime plugin descriptor production files 16/16",
        "RuntimePluginDescriptor public-field convergence remains pending",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_06_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || package_manifest_doc.contains(doc_anchor)
                || first_party_catalog_doc.contains(doc_anchor),
            "first-party RuntimePluginDescriptor builder migration docs should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f8_runtime_plugin_descriptor_test_fixtures_use_builder() {
    let fixture_sources = [
        (
            "asset_importer_install",
            include_str!("../../plugin_extensions/asset_importer_install.rs"),
        ),
        (
            "extension_registry",
            include_str!("../../plugin_extensions/extension_registry.rs"),
        ),
        (
            "extension_registry_components",
            include_str!("../../plugin_extensions/extension_registry_components.rs"),
        ),
        (
            "extension_registry_event_catalogs",
            include_str!("../../plugin_extensions/extension_registry_event_catalogs.rs"),
        ),
        (
            "extension_registry_features",
            include_str!("../../plugin_extensions/extension_registry_features.rs"),
        ),
        (
            "extension_registry_managers",
            include_str!("../../plugin_extensions/extension_registry_managers.rs"),
        ),
        (
            "extension_registry_metadata",
            include_str!("../../plugin_extensions/extension_registry_metadata.rs"),
        ),
        (
            "extension_registry_modules",
            include_str!("../../plugin_extensions/extension_registry_modules.rs"),
        ),
        (
            "extension_registry_options",
            include_str!("../../plugin_extensions/extension_registry_options.rs"),
        ),
        (
            "profile_maturity",
            include_str!("../../plugin_extensions/profile_maturity.rs"),
        ),
        (
            "runtime_plugin_catalog_features",
            include_str!("../../plugin_extensions/runtime_plugin_catalog_features.rs"),
        ),
        (
            "runtime_plugin_descriptor",
            include_str!("../../plugin_extensions/runtime_plugin_descriptor.rs"),
        ),
        (
            "runtime_plugin_lifecycle",
            include_str!("../../plugin_extensions/runtime_plugin_lifecycle.rs"),
        ),
        (
            "runtime_plugin_package_manifest",
            include_str!("../../plugin_extensions/runtime_plugin_package_manifest.rs"),
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
        builder_count, 64,
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
        include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_06_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention = include_str!("../../../../../docs/plans/engine-code-structure-convention.md");
    let package_manifest_doc =
        include_str!("../../../../../docs/zircon_runtime/plugin/package_manifest.md");

    for doc_anchor in [
        "F8 RuntimePluginDescriptor test fixture builder migration",
        "runtime_plugin_descriptor_test_fixture_builder_migration_coremin_check_passed",
        "review_f8_runtime_plugin_descriptor_test_fixtures_use_builder",
        "plugin extension RuntimePluginDescriptor test fixtures 14/14",
        "RuntimePluginDescriptor public-field convergence remains pending",
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

#[test]
fn review_f8_runtime_plugin_descriptor_fields_are_private_with_accessors() {
    let descriptor = include_str!("../../../plugin/runtime_plugin/descriptor.rs");
    let accessors = include_str!("../../../plugin/runtime_plugin/descriptor/access.rs");
    let review_findings =
        include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_06_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention = include_str!("../../../../../docs/plans/engine-code-structure-convention.md");
    let package_manifest_doc =
        include_str!("../../../../../docs/zircon_runtime/plugin/package_manifest.md");

    assert!(
        descriptor.contains("mod access;"),
        "RuntimePluginDescriptor should keep field accessors in a dedicated descriptor/access owner"
    );

    for field in [
        "package_id",
        "display_name",
        "category",
        "runtime_id",
        "crate_name",
        "enabled_by_default",
        "required_by_default",
        "target_modes",
        "capabilities",
        "system_sets",
        "system_anchors",
        "capability_statuses",
        "maturity",
        "optional_features",
        "default_packaging",
    ] {
        assert!(
            !descriptor.contains(&format!("pub {field}:")),
            "RuntimePluginDescriptor field `{field}` should not remain public"
        );
    }

    for accessor in [
        "pub fn package_id(&self) -> &str",
        "pub fn display_name(&self) -> &str",
        "pub fn category(&self) -> &str",
        "pub fn runtime_id(&self) -> RuntimePluginId",
        "pub fn crate_name(&self) -> &str",
        "pub fn enabled_by_default(&self) -> bool",
        "pub fn required_by_default(&self) -> bool",
        "pub fn target_modes(&self) -> &[RuntimeTargetMode]",
        "pub fn capabilities(&self) -> &[String]",
        "pub fn system_sets(&self) -> &[String]",
        "pub fn system_anchors(&self) -> &[String]",
        "pub fn capability_statuses(&self) -> &[CapabilityStatusManifest]",
        "pub fn maturity(&self) -> PluginMaturity",
        "pub fn optional_features(&self) -> &[PluginFeatureBundleManifest]",
        "pub fn default_packaging(&self) -> &[ExportPackagingStrategy]",
    ] {
        assert!(
            accessors.contains(accessor),
            "RuntimePluginDescriptor accessors should expose `{accessor}`"
        );
    }

    for doc_anchor in [
        "F8 RuntimePluginDescriptor public-field convergence",
        "runtime_plugin_descriptor_public_field_convergence_coremin_check_passed",
        "review_f8_runtime_plugin_descriptor_fields_are_private_with_accessors",
        "RuntimePluginDescriptor private fields 15/15",
        "RuntimePluginDescriptor public-field convergence complete",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_06_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || package_manifest_doc.contains(doc_anchor),
            "RuntimePluginDescriptor public-field convergence docs should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f8_runtime_plugin_descriptor_public_constructor_is_retired() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .expect("runtime crate should have repository parent");
    let descriptor_builder_mod =
        include_str!("../../../plugin/runtime_plugin/descriptor/builder.rs");
    let descriptor_builder = include_str!(
        "../../../plugin/runtime_plugin/descriptor/builder/runtime_plugin_descriptor_builder.rs"
    );
    let builtin_catalog_root = include_str!("../../../plugin/runtime_plugin/builtin_catalog.rs");
    let plugin_sdk_runtime =
        std::fs::read_to_string(repo_root.join("zircon_plugins/plugin_sdk/src/runtime.rs"))
            .expect("read plugin SDK runtime declaration source");
    let review_findings =
        include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_06_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention = include_str!("../../../../../docs/plans/engine-code-structure-convention.md");
    let package_manifest_doc =
        include_str!("../../../../../docs/zircon_runtime/plugin/package_manifest.md");

    for retired_owner in [
        manifest_dir.join("src/plugin/runtime_plugin/descriptor/builder/construction.rs"),
        manifest_dir.join("src/plugin/runtime_plugin/descriptor/builder/fluent.rs"),
    ] {
        assert!(
            !retired_owner.exists(),
            "RuntimePluginDescriptor retired constructor/fluent owner should stay absent: {}",
            retired_owner.display()
        );
    }
    for forbidden in ["mod construction;", "mod fluent;"] {
        assert!(
            !descriptor_builder_mod.contains(forbidden),
            "RuntimePluginDescriptor builder module should not mount retired owner `{forbidden}`"
        );
    }
    for forbidden in ["RuntimePluginDescriptor::new("] {
        assert!(
            !descriptor_builder.contains(forbidden) && !plugin_sdk_runtime.contains(forbidden),
            "RuntimePluginDescriptor builder/SDK should not keep retired public constructor surface `{forbidden}`"
        );
    }
    assert!(
        descriptor_builder.contains("descriptor: RuntimePluginDescriptor {")
            && plugin_sdk_runtime.contains("builder: RuntimePluginDescriptorBuilder")
            && plugin_sdk_runtime.contains("RuntimePluginDescriptor::builder(")
            && builtin_catalog_root.contains("Self::builder(")
            && builtin_catalog_root.contains("type BuiltinCatalogDescriptorBuilder = RuntimePluginDescriptorBuilder;")
            && builtin_catalog_root.contains(".map(RuntimePluginDescriptorBuilder::build)")
            && !plugin_sdk_runtime.contains("self.descriptor = self.descriptor.with_"),
        "RuntimePluginDescriptor builder and plugin SDK declaration should use the blessed builder storage path"
    );

    fn collect_builtin_catalog_sources(
        directory: &std::path::Path,
        sources: &mut Vec<(std::path::PathBuf, String)>,
    ) {
        for entry in std::fs::read_dir(directory).expect("read builtin catalog directory") {
            let entry = entry.expect("read builtin catalog entry");
            let path = entry.path();
            if path.is_dir() {
                collect_builtin_catalog_sources(&path, sources);
            } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
                let source =
                    std::fs::read_to_string(&path).expect("read builtin catalog source file");
                sources.push((path, source));
            }
        }
    }

    let mut builtin_catalog_sources = Vec::new();
    collect_builtin_catalog_sources(
        &manifest_dir.join("src/plugin/runtime_plugin/builtin_catalog"),
        &mut builtin_catalog_sources,
    );
    for (path, source) in builtin_catalog_sources {
        assert!(
            !source.contains("RuntimePluginDescriptor"),
            "builtin catalog child modules should pass the descriptor builder instead of direct descriptor values: {}",
            path.display()
        );
    }

    for doc_anchor in [
        "F8 RuntimePluginDescriptor public constructor retired",
        "runtime_plugin_descriptor_public_constructor_retired_coremin_check_passed",
        "review_f8_runtime_plugin_descriptor_public_constructor_is_retired",
        "RuntimePluginDescriptor::new retired",
        "descriptor/builder/construction.rs retired",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_06_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || package_manifest_doc.contains(doc_anchor),
            "RuntimePluginDescriptor public constructor retirement docs should record `{doc_anchor}`"
        );
    }
}
