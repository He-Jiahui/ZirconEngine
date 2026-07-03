#[test]
fn review_f8_runtime_plugin_descriptor_fields_are_private_with_accessors() {
    let descriptor = include_str!("../../../../../plugin/runtime_plugin/descriptor.rs");
    let accessors = include_str!("../../../../../plugin/runtime_plugin/descriptor/access.rs");
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
