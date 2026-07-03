#[test]
fn review_f8_runtime_plugin_descriptor_public_constructor_is_retired() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .expect("runtime crate should have repository parent");
    let descriptor_builder_mod =
        include_str!("../../../../../plugin/runtime_plugin/descriptor/builder.rs");
    let descriptor_builder = include_str!(
        "../../../../../plugin/runtime_plugin/descriptor/builder/runtime_plugin_descriptor_builder.rs"
    );
    let builtin_catalog_root =
        include_str!("../../../../../plugin/runtime_plugin/builtin_catalog.rs");
    let plugin_sdk_runtime =
        std::fs::read_to_string(repo_root.join("zircon_plugins/plugin_sdk/src/runtime.rs"))
            .expect("read plugin SDK runtime declaration source");
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
