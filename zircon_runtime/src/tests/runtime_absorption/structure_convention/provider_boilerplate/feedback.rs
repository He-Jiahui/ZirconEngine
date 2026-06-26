use super::*;

#[test]
fn runtime_15_provider_feedback_uses_shared_payload_owner() {
    let shared_feedback = read_runtime_src("graphics/runtime_provider/feedback.rs");
    let runtime_provider_mod = read_runtime_src("graphics/runtime_provider/mod.rs");
    let hybrid_feedback =
        read_runtime_src("graphics/hybrid_gi_runtime_provider/runtime_feedback.rs");
    let virtual_geometry_feedback =
        read_runtime_src("graphics/virtual_geometry_runtime_provider/runtime_feedback.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let provider_doc = read_repo("docs/zircon_runtime/graphics/runtime_provider/feedback.md");

    assert_contains_all(
        "shared runtime provider feedback owner",
        &shared_feedback,
        &[
            "pub(crate) struct RuntimeProviderFeedback<G, V>",
            "gpu_completion: Option<G>",
            "visibility_feedback: Option<V>",
            "gpu_completion(&self) -> Option<&G>",
            "visibility_feedback(&self) -> Option<&V>",
        ],
    );
    assert_contains_all(
        "runtime provider owner exports",
        &runtime_provider_mod,
        &["mod feedback;", "RuntimeProviderFeedback"],
    );

    for (label, source, owner_type, gpu_type, visibility_type) in [
        (
            "hybrid GI provider feedback",
            hybrid_feedback.as_str(),
            "RuntimeProviderFeedback<HybridGiGpuCompletion, VisibilityHybridGiFeedback>",
            "HybridGiGpuCompletion",
            "VisibilityHybridGiFeedback",
        ),
        (
            "virtual geometry provider feedback",
            virtual_geometry_feedback.as_str(),
            "RuntimeProviderFeedback<VirtualGeometryGpuCompletion, VisibilityVirtualGeometryFeedback>",
            "VirtualGeometryGpuCompletion",
            "VisibilityVirtualGeometryFeedback",
        ),
    ] {
        assert_contains_all(label, source, &["RuntimeProviderFeedback", owner_type]);
        for duplicated_field in [
            format!("\n    gpu_completion: Option<{gpu_type}>,"),
            format!("\n    visibility_feedback: Option<{visibility_type}>,"),
        ] {
            assert!(
                !source.contains(&duplicated_field),
                "{label} should delegate common feedback payload storage to RuntimeProviderFeedback instead of declaring `{duplicated_field}`"
            );
        }
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("runtime provider doc", provider_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 F13 provider feedback shared payload owner",
                "runtime_15_provider_feedback_shared_payload_owner_coremin_check_passed",
                "runtime_15_provider_feedback_uses_shared_payload_owner",
            ],
        );
    }
}
