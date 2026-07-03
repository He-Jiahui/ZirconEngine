#[test]
fn review_f11_shading_model_registry_has_no_dead_plugin_registration_surface() {
    let registry = include_str!("../../../../graphics/material/shading_models/registry.rs");
    let core_contract = include_str!("../../../../core/framework/render/material/shading_model.rs");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let render_index = include_str!("../../../../../../docs/plans/zircon_runtime/render/index.md");
    let material_doc =
        include_str!("../../../../../../docs/zircon_runtime/core/framework/render/material.md");

    assert!(
        !registry.contains("#[allow(dead_code)]"),
        "shading-model registry should not preserve disconnected production API behind dead_code"
    );
    for forbidden_dead_surface in [
        "pub(crate) fn supported_channels(",
        "pub(crate) fn len(",
        "pub(crate) fn register_plugin(",
        "PluginIdBelowReservedRange",
    ] {
        assert!(
            !registry.contains(forbidden_dead_surface)
                && !core_contract.contains(forbidden_dead_surface),
            "F11 should not reintroduce disconnected shading-model registry surface `{forbidden_dead_surface}`"
        );
    }

    for live_registry_anchor in [
        "fn resolve_token(&self, token: &str)",
        "self.resolve_token(&model.as_token())",
        "pub(crate) fn register_builtin(",
        "RequiredChannelsUnsupported",
    ] {
        assert!(
            registry.contains(live_registry_anchor),
            "shading-model registry should retain live built-in resolver anchor `{live_registry_anchor}`"
        );
    }

    for doc_anchor in [
        "F11 shading-model registry dead API removal",
        "render_shading_model_registry_dead_api_removed_coremin_passed",
        "review_f11_shading_model_registry_has_no_dead_plugin_registration_surface",
        "custom shading-model plugin registration remains a future Plan 08 surface",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || render_index.contains(doc_anchor)
                || material_doc.contains(doc_anchor),
            "F11 docs should record `{doc_anchor}`"
        );
    }
}
