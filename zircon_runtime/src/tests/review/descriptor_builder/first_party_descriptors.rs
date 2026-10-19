//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。以结果断言检查当前接口或源码快照对应的边界。
use crate::tests::runtime_absorption::structure_convention::fixed_history_archive_source as history;
#[test]
fn review_f8_first_party_runtime_plugin_descriptors_use_builder() {
    let plugin_sources = [
        (
            "ai",
            include_str!("../../../../../zircon_plugins/ai/runtime/src/plugin.rs"),
        ),
        (
            "animation",
            include_str!("../../../../../zircon_plugins/animation/runtime/src/plugin.rs"),
        ),
        (
            "hybrid_gi",
            include_str!("../../../../../zircon_plugins/hybrid_gi/runtime/src/plugin.rs"),
        ),
        (
            "navigation",
            include_str!("../../../../../zircon_plugins/navigation/runtime/src/plugin.rs"),
        ),
        (
            "net",
            include_str!("../../../../../zircon_plugins/net/runtime/src/plugin.rs"),
        ),
        (
            "particles",
            include_str!("../../../../../zircon_plugins/particles/runtime/src/plugin.rs"),
        ),
        (
            "physics",
            include_str!("../../../../../zircon_plugins/physics/runtime/src/plugin.rs"),
        ),
        (
            "prefab_tools",
            include_str!("../../../../../zircon_plugins/prefab_tools/runtime/src/plugin.rs"),
        ),
        (
            "rendering",
            include_str!("../../../../../zircon_plugins/rendering/runtime/src/plugin.rs"),
        ),
        (
            "solari",
            include_str!("../../../../../zircon_plugins/solari/runtime/src/plugin.rs"),
        ),
        (
            "sound",
            include_str!(
                "../../../../../zircon_plugins/sound/runtime/src/runtime_plugin/descriptor.rs"
            ),
        ),
        (
            "terrain",
            include_str!("../../../../../zircon_plugins/terrain/runtime/src/plugin.rs"),
        ),
        (
            "texture",
            include_str!("../../../../../zircon_plugins/texture/runtime/src/plugin.rs"),
        ),
        (
            "tilemap_2d",
            include_str!("../../../../../zircon_plugins/tilemap_2d/runtime/src/plugin.rs"),
        ),
        (
            "virtual_geometry",
            include_str!("../../../../../zircon_plugins/virtual_geometry/runtime/src/plugin.rs"),
        ),
        (
            "zr_vm_language",
            include_str!("../../../../../zircon_plugins/zr_vm_language/runtime/src/plugin.rs"),
        ),
    ];
    assert_eq!(
        plugin_sources.len(),
        16,
        "F8 first-party runtime plugin descriptor migration should enumerate every production runtime plugin"
    );

    let declarations = [
        ("AI_DECLARATION", "RUNTIME_CRATE_NAME"),
        ("ANIMATION_DECLARATION", "RUNTIME_CRATE_NAME"),
        ("HYBRID_GI_DECLARATION", "RUNTIME_CRATE_NAME"),
        ("NAVIGATION_DECLARATION", "RUNTIME_CRATE_NAME"),
        ("NET_DECLARATION", "RUNTIME_CRATE_NAME"),
        ("PARTICLES_DECLARATION", "RUNTIME_CRATE_NAME"),
        ("PHYSICS_DECLARATION", "RUNTIME_CRATE_NAME"),
        ("PREFAB_TOOLS_DECLARATION", "RUNTIME_CRATE_NAME"),
        ("RENDERING_DECLARATION", "RUNTIME_CRATE_NAME"),
        ("SOLARI_DECLARATION", "RUNTIME_CRATE_NAME"),
        ("SOUND_DECLARATION", "RUNTIME_CRATE_NAME"),
        ("TERRAIN_DECLARATION", "RUNTIME_CRATE_NAME"),
        ("TEXTURE_PLUGIN_DECLARATION", "TEXTURE_RUNTIME_CRATE_NAME"),
        ("TILEMAP_2D_DECLARATION", "RUNTIME_CRATE_NAME"),
        ("VIRTUAL_GEOMETRY_DECLARATION", "RUNTIME_CRATE_NAME"),
        ("ZR_VM_LANGUAGE_DECLARATION", "RUNTIME_CRATE_NAME"),
    ];
    for ((name, source), (declaration, crate_name)) in plugin_sources.iter().zip(declarations) {
        let compact: String = source.split_whitespace().collect();
        assert!(
            compact.contains(&format!("{declaration}.runtime_declaration({crate_name})")),
            "{name} descriptor must use its own named declaration"
        );
    }

    for (name, source) in plugin_sources {
        assert!(
            source.contains(".runtime_declaration("),
            "first-party runtime plugin `{name}` should construct descriptors through the builder"
        );
        assert!(
            !source.contains("RuntimePluginDescriptor::new("),
            "first-party runtime plugin `{name}` should not keep the old descriptor constructor"
        );
        assert!(
            source.contains(".into_descriptor()"),
            "first-party runtime plugin `{name}` should finish the SDK declaration through into_descriptor()"
        );
    }

    let declaration = include_str!("../../../../../zircon_plugins/plugin_sdk/src/declaration.rs");
    let runtime = include_str!("../../../../../zircon_plugins/plugin_sdk/src/runtime.rs");
    for required in [
        "pub fn runtime_declaration(self, crate_name: impl Into<String>)",
        "RuntimePluginDeclaration::new(self.id, self.display_name, runtime_id, crate_name)",
        ".with_target_modes(self.target_modes())",
    ] {
        assert!(
            declaration.contains(required),
            "SDK declaration missing {required}"
        );
    }
    for required in [
        "builder: RuntimePluginDescriptor::builder(",
        "pub fn into_descriptor(self) -> RuntimePluginDescriptor",
        "self.builder.build()",
    ] {
        assert!(
            runtime.contains(required),
            "SDK runtime declaration missing {required}"
        );
    }

    let review_findings = crate::runtime_test_history_concat!(
        include_str!("../../../../../docs/architecture/runtime-architecture-review-m0.md"),
        history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md")
    );
    let runtime_06_plan = include_str!(
        "../../../../../docs/testing-and-validation/runtime-plugin-validation-worklist.md"
    );
    let runtime_index = include_str!(
        "../../../../../docs/crates/zircon_runtime/structure/runtime-absorption-evidence.md"
    );
    let convention =
        include_str!("../../../../../docs/testing-and-validation/development-conventions.md");
    let package_manifest_doc =
        include_str!("../../../../../docs/crates/zircon_runtime/plugin/package_manifest.md");
    let first_party_catalog_doc =
        include_str!("../../../../../docs/crates/zircon_plugins/first_party_runtime_catalog.md");

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
