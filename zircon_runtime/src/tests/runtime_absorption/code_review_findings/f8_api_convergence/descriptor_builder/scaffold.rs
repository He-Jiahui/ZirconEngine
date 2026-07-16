#[test]
fn review_f8_runtime_plugin_descriptor_exposes_builder_scaffold() {
    let descriptor = include_str!("../../../../../plugin/runtime_plugin/descriptor.rs");
    let builder_mod = include_str!("../../../../../plugin/runtime_plugin/descriptor/builder.rs");
    let builder_source = include_str!(
        "../../../../../plugin/runtime_plugin/descriptor/builder/runtime_plugin_descriptor_builder.rs"
    );
    let runtime_plugin_mod = include_str!("../../../../../plugin/runtime_plugin/mod.rs");
    let plugin_mod = include_str!("../../../../../plugin/mod.rs");
    let plugin_descriptor_tests =
        include_str!("../../../../../tests/plugin_extensions/runtime_plugin_descriptor.rs");
    let review_findings = concat!(
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md"),
        include_str!("../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md")
    );
    let runtime_06_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../../docs/plans/engine-code-structure-convention.md");
    let package_manifest_doc =
        include_str!("../../../../../../../docs/zircon_runtime/plugin/package_manifest.md");

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
        "RuntimePluginDescriptor public-field convergence complete",
        "RuntimePluginDescriptor::new retired",
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
