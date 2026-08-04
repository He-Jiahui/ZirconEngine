#[test]
fn review_f8_runtime_plugin_descriptor_status_mirrors_do_not_claim_public_field_pending() {
    let descriptor = include_str!("../../../../../plugin/runtime_plugin/descriptor.rs");
    let accessors = include_str!("../../../../../plugin/runtime_plugin/descriptor/access.rs");
    let descriptor_builder_mod =
        include_str!("../../../../../plugin/runtime_plugin/descriptor/builder.rs");
    let descriptor_builder = include_str!(
        "../../../../../plugin/runtime_plugin/descriptor/builder/runtime_plugin_descriptor_builder.rs"
    );
    let plugin_sdk_runtime =
        include_str!("../../../../../../../zircon_plugins/plugin_sdk/src/runtime.rs");
    let review_findings = concat!(
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md"),
        include_str!("../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md")
    );
    let convention =
        include_str!("../../../../../../../docs/plans/engine-code-structure-convention.md");
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let runtime_06_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md"
    );
    let runtime_15_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let package_manifest_doc =
        include_str!("../../../../../../../docs/zircon_runtime/plugin/package_manifest.md");

    let stale_pending_anchor = [
        "RuntimePluginDescriptor public-field convergence",
        "remains pending",
    ]
    .join(" ");
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
            "RuntimePluginDescriptor field `{field}` should stay private"
        );
        assert!(
            accessors.contains(&format!("pub fn {field}(&self)")),
            "RuntimePluginDescriptor accessor `{field}` should stay available"
        );
    }
    assert!(
        descriptor_builder.contains("descriptor: RuntimePluginDescriptor {")
            && !descriptor_builder_mod.contains("mod construction;")
            && !descriptor_builder_mod.contains("mod fluent;")
            && !descriptor_builder.contains("RuntimePluginDescriptor::new(")
            && !plugin_sdk_runtime.contains("RuntimePluginDescriptor::new("),
        "RuntimePluginDescriptor construction should stay on the direct builder path without the retired public constructor"
    );

    for (label, source) in [
        ("review findings", review_findings),
        ("structure convention", convention),
        ("runtime index", runtime_index),
        ("Runtime 06 plan", runtime_06_plan),
        ("Runtime 15 plan", runtime_15_plan),
        ("package manifest docs", package_manifest_doc),
    ] {
        assert!(
            !source.contains(&stale_pending_anchor),
            "{label} should not claim RuntimePluginDescriptor public-field convergence is pending after the private-field and constructor-retirement slices"
        );
    }

    let completion_doc_anchors = [
        "Runtime 15 F8 RuntimePluginDescriptor status mirror cleanup",
        "runtime_15_runtime_plugin_descriptor_status_mirror_cleanup_static_passed_cargo_deferred",
        "review_f8_runtime_plugin_descriptor_status_mirrors_do_not_claim_public_field_pending",
        "RuntimePluginDescriptor private fields 15/15",
        "RuntimePluginDescriptor public-field convergence complete",
        "RuntimePluginDescriptor::new retired",
        "f8_f9_f10_runtime_surface_top_row_closed_status_static_passed_cargo_deferred",
    ];
    for doc_anchor in completion_doc_anchors {
        assert!(
            review_findings.contains(doc_anchor),
            "RuntimePluginDescriptor numbered output should record `{doc_anchor}`"
        );
    }
    let f8_row = review_findings
        .lines()
        .find(|line| line.starts_with("| F8 |"))
        .expect("F8 review findings top row");
    assert!(
        f8_row.contains("texture import settings")
            && f8_row.ends_with("| Runtime 04 + Runtime 06 + Runtime 15 |"),
        "F8 overview row should keep only the finding and delegated owners"
    );
    assert!(
        review_findings.contains(
            "f8_f9_f10_runtime_surface_top_row_closed_status_static_passed_cargo_deferred"
        ),
        "F8 numbered output should record runtime surface review closed status"
    );
}
