use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_render_frame_submission_context_tests_are_child_owner() {
    let parent = read_runtime_src(
        "graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs",
    );
    let tests = read_runtime_src(
        "graphics/runtime/render_framework/submit_frame_extract/frame_submission_context/tests.rs",
    );

    let plan_09 = read_repo("docs/plans/zircon_runtime/render/09-camera-render-ordering.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let advanced_doc = read_repo("docs/zircon_runtime/core/framework/render/advanced.md");
    let anti_alias_doc = read_repo("docs/zircon_runtime/core/framework/render/anti_alias.md");
    let session_note = read_repo(".codex/sessions/20260617-0926-render-hzb-progress.md");

    assert_contains_all(
        "frame submission context parent keeps submit context production responsibilities and test mount",
        &parent,
        &[
            "pub(super) struct FrameSubmissionContext {",
            "pub(super) fn new(",
            "pub(super) fn source_extract(&self) -> Arc<RenderFrameExtract>",
            "pub(super) fn view_visibility(",
            "pub(super) fn advanced_provider_reports(&self) -> &[AdvancedProviderReport]",
            "pub(super) fn temporal_jitter_for_submission(",
            "impl UiSubmissionStats {",
            "#[cfg(test)]\nmod tests;",
        ],
    );

    for moved_test in [
        "fn advanced_runtime_plan_gates_provider_missing_feature_payloads(",
        "fn advanced_runtime_plan_keeps_provider_backed_features_enabled(",
        "fn frame_submission_context_exposes_view_visibility_by_key(",
        "fn virtual_geometry_payload_source_clears_when_plan_degrades_feature(",
        "fn virtual_geometry_payload_source_survives_for_provider_backed_extract(",
        "fn render_taa_jitter_zero_when_taa_inactive(",
        "fn hybrid_gi_payload_source_clears_when_plan_degrades_feature(",
        "fn hybrid_gi_payload_source_survives_for_provider_backed_extract(",
    ] {
        assert!(
            !parent.contains(moved_test),
            "frame_submission_context.rs should mount the test child instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "frame submission context test child owns advanced feature gating and jitter coverage",
        &tests,
        &[
            "use super::*;",
            "fn advanced_runtime_plan_gates_provider_missing_feature_payloads(",
            "fn advanced_runtime_plan_keeps_provider_backed_features_enabled(",
            "fn frame_submission_context_exposes_view_visibility_by_key(",
            "fn virtual_geometry_payload_source_clears_when_plan_degrades_feature(",
            "fn virtual_geometry_payload_source_survives_for_provider_backed_extract(",
            "fn render_taa_jitter_zero_when_taa_inactive(",
            "fn hybrid_gi_payload_source_clears_when_plan_degrades_feature(",
            "fn hybrid_gi_payload_source_survives_for_provider_backed_extract(",
            "fn context_with_advanced_plan(",
            "fn empty_pipeline() -> CompiledRenderPipeline",
        ],
    );

    for (path, source) in [
        (
            "graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs",
            parent.as_str(),
        ),
        (
            "graphics/runtime/render_framework/submit_frame_extract/frame_submission_context/tests.rs",
            tests.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 frame submission context owner budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 09", plan_09.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("advanced docs", advanced_doc.as_str()),
        ("anti alias docs", anti_alias_doc.as_str()),
        ("session note", session_note.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Plan 09 frame submission context tests owner split",
                "render_plan09_frame_submission_context_tests_owner_split_static_passed_cargo_deferred_active_compile_lane",
                "graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs",
                "graphics/runtime/render_framework/submit_frame_extract/frame_submission_context/tests.rs",
                "runtime_15_render_frame_submission_context_tests_are_child_owner",
            ],
        );
    }
}
