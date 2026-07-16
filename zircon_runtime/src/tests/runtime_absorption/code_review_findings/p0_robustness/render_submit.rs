#[test]
fn review_f4_render_submit_capability_gaps_return_typed_errors() {
    let viewport_guard = include_str!(
        "../../../../graphics/runtime/render_framework/submit_frame_extract/viewport_generation_guard.rs"
    );
    let prepare_runtime_submission = include_str!(
        "../../../../graphics/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/prepare.rs"
    );
    let submit_frame_extract = include_str!(
        "../../../../graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs"
    );
    let submit_runtime_frame = include_str!(
        "../../../../graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs"
    );
    let present_frame_extract = include_str!(
        "../../../../graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs"
    );
    let review_findings = concat!(
        include_str!("../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"),
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md")
    );
    let runtime_07_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let render_index = include_str!("../../../../../../docs/plans/zircon_runtime/render/index.md");
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let advanced_doc =
        include_str!("../../../../../../docs/zircon_runtime/core/framework/render/advanced.md");
    let status_rows = include_str!(
        "../../plan_status/status_output_tables/expected_status_row_data/runtime_06_09/runtime_07/performance.rs"
    );
    let status_map = include_str!(
        "../../plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_06_10.rs"
    );
    let date_map = include_str!(
        "../../plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_06_10.rs"
    );

    for required in [
        "pub(super) fn validate_viewport_generation",
        "RenderFrameworkError::UnknownViewport",
        "RenderFrameworkError::ViewportChanged",
        "pub(super) fn viewport_record_mut_after_generation_check",
        "validate_viewport_generation(state, viewport, context)?",
    ] {
        assert!(
            viewport_guard.contains(required),
            "viewport generation guard should keep typed error anchor `{required}`"
        );
    }

    for required in [
        "return Err(missing_runtime_provider(\"hybrid global illumination\"));",
        "return Err(missing_runtime_provider(\"virtual geometry\"));",
        "RenderFrameworkError::UnsupportedCapability",
        "capability: format!(\"{feature} runtime provider\")",
        "record.clear_hybrid_gi_runtimes();",
        "record.clear_virtual_geometry_runtimes();",
    ] {
        assert!(
            prepare_runtime_submission.contains(required),
            "prepare runtime submission should keep provider-missing typed error anchor `{required}`"
        );
    }

    for (label, source) in [
        ("submit generated frame", submit_frame_extract),
        ("submit direct runtime frame", submit_runtime_frame),
        ("present generated frame", present_frame_extract),
        ("prepare runtime submission", prepare_runtime_submission),
    ] {
        let production = production_source(source);
        assert!(
            !production.contains(".unwrap("),
            "{label} production path should not panic through unwrap"
        );
        assert!(
            !production.contains(".expect("),
            "{label} production path should not panic through expect"
        );
    }

    for (label, source) in [
        ("submit generated frame", submit_frame_extract),
        ("submit direct runtime frame", submit_runtime_frame),
        ("present generated frame", present_frame_extract),
    ] {
        let production = production_source(source);
        assert!(
            production.contains("validate_viewport_generation(&state, viewport, &context)"),
            "{label} should validate viewport generation before record writeback"
        );
        assert!(
            production.contains(
                "viewport_record_mut_after_generation_check(&mut state, viewport, &context)?"
            ),
            "{label} should fetch viewport records through the checked helper"
        );
    }

    for doc_anchor in [
        "Runtime 07 render submit viewport/provider errors",
        "render_submit_viewport_provider_errors_review_guard_static_passed_cargo_timeout_no_result_full_runtime07_pending",
        "review_f4_render_submit_capability_gaps_return_typed_errors",
        "p0_f1_f2_f4_top_row_closed_status_static_passed_cargo_deferred",
        "RenderFrameworkError::UnsupportedCapability",
        "viewport_record_mut_after_generation_check",
        "submit_frame_extract production paths must return RenderFrameworkError",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_07_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || render_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || advanced_doc.contains(doc_anchor)
                || status_rows.contains(doc_anchor)
                || status_map.contains(doc_anchor)
                || date_map.contains(doc_anchor),
            "F4 render submit typed-error docs/status should record `{doc_anchor}`"
        );
    }
    let f4_row = review_findings
        .lines()
        .find(|line| line.starts_with("| F4 |"))
        .expect("F4 row should exist");
    assert!(
        f4_row.ends_with("| Runtime 07 + render index / review closed |"),
        "F4 row should mark the typed-error review state closed while full Runtime 07 gate remains separate"
    );
}

fn production_source(source: &str) -> &str {
    source.split("\n#[cfg(test)]").next().unwrap_or(source)
}
