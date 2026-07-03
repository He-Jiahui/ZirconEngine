#[test]
fn review_f8_first_party_runtime_plugin_descriptors_use_builder() {
    let plugin_sources = [
        (
            "ai",
            include_str!("../../../../../../../zircon_plugins/ai/runtime/src/plugin.rs"),
        ),
        (
            "animation",
            include_str!("../../../../../../../zircon_plugins/animation/runtime/src/plugin.rs"),
        ),
        (
            "hybrid_gi",
            include_str!("../../../../../../../zircon_plugins/hybrid_gi/runtime/src/plugin.rs"),
        ),
        (
            "navigation",
            include_str!("../../../../../../../zircon_plugins/navigation/runtime/src/plugin.rs"),
        ),
        (
            "net",
            include_str!("../../../../../../../zircon_plugins/net/runtime/src/plugin.rs"),
        ),
        (
            "particles",
            include_str!("../../../../../../../zircon_plugins/particles/runtime/src/plugin.rs"),
        ),
        (
            "physics",
            include_str!("../../../../../../../zircon_plugins/physics/runtime/src/plugin.rs"),
        ),
        (
            "prefab_tools",
            include_str!("../../../../../../../zircon_plugins/prefab_tools/runtime/src/plugin.rs"),
        ),
        (
            "rendering",
            include_str!("../../../../../../../zircon_plugins/rendering/runtime/src/plugin.rs"),
        ),
        (
            "solari",
            include_str!("../../../../../../../zircon_plugins/solari/runtime/src/plugin.rs"),
        ),
        (
            "sound",
            include_str!(
                "../../../../../../../zircon_plugins/sound/runtime/src/runtime_plugin/descriptor.rs"
            ),
        ),
        (
            "terrain",
            include_str!("../../../../../../../zircon_plugins/terrain/runtime/src/plugin.rs"),
        ),
        (
            "texture",
            include_str!("../../../../../../../zircon_plugins/texture/runtime/src/plugin.rs"),
        ),
        (
            "tilemap_2d",
            include_str!("../../../../../../../zircon_plugins/tilemap_2d/runtime/src/plugin.rs"),
        ),
        (
            "virtual_geometry",
            include_str!(
                "../../../../../../../zircon_plugins/virtual_geometry/runtime/src/plugin.rs"
            ),
        ),
        (
            "zr_vm_language",
            include_str!(
                "../../../../../../../zircon_plugins/zr_vm_language/runtime/src/plugin.rs"
            ),
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
    let first_party_catalog_doc =
        include_str!("../../../../../../../docs/zircon_plugins/first_party_runtime_catalog.md");

    for doc_anchor in [
        "F8 first-party RuntimePluginDescriptor builder migration",
        "runtime_plugin_descriptor_first_party_builder_migration_coremin_check_passed",
        "review_f8_first_party_runtime_plugin_descriptors_use_builder",
        "first-party runtime plugin descriptor production files 16/16",
        "RuntimePluginDescriptor public-field convergence complete",
        "RuntimePluginDescriptor::new retired",
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
