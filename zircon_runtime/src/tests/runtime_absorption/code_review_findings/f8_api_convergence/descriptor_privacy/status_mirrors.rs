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
    let review_findings =
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
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
    let status_rows = include_str!(
        "../../../plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs"
    );
    let status_map = include_str!(
        "../../../plan_status/status_output_tables/expected_slices/status/runtime_15/foundation.rs"
    );
    let date_map = include_str!(
        "../../../plan_status/status_output_tables/expected_slices/date/runtime_15/foundation.rs"
    );

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
        ("status-output row data", status_rows),
        ("status-output status map", status_map),
        ("status-output date map", date_map),
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
            review_findings.contains(doc_anchor)
                && convention.contains(doc_anchor)
                && runtime_index.contains(doc_anchor)
                && runtime_06_plan.contains(doc_anchor)
                && runtime_15_plan.contains(doc_anchor)
                && package_manifest_doc.contains(doc_anchor)
                && status_rows.contains(doc_anchor),
            "RuntimePluginDescriptor status mirror docs/status should record `{doc_anchor}`"
        );
    }
    assert!(
        status_map.contains("Runtime 15 F8 RuntimePluginDescriptor status mirror cleanup")
            && status_map.contains(
                "runtime_15_runtime_plugin_descriptor_status_mirror_cleanup_static_passed_cargo_deferred",
            )
            && date_map.contains("Runtime 15 F8 RuntimePluginDescriptor status mirror cleanup")
            && date_map.contains("2026-06-27"),
        "RuntimePluginDescriptor status mirror slice should be indexed by status/date maps"
    );
    let f8_row = review_findings
        .lines()
        .find(|line| line.starts_with("| F8 |"))
        .expect("F8 review findings top row");
    assert!(
        f8_row.contains(
            "f8_f9_f10_runtime_surface_top_row_closed_status_static_passed_cargo_deferred"
        ) && f8_row
            .ends_with("| convention + Runtime 04 + Runtime 06 + Runtime 15 / review closed |"),
        "F8 top row should record runtime surface review closed status"
    );
}
