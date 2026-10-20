//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。以结果断言检查当前接口或源码快照对应的边界。
use crate::tests::runtime_absorption::structure_convention::fixed_history_archive_source as history;
use crate::tests::runtime_absorption::structure_convention::rust_source_view::production_section;
#[test]
fn review_f4_render_submit_capability_gaps_return_typed_errors() {
    let viewport_guard =
        include_str!("../../../../graphics/runtime/framework/extract/viewport_generate_guard.rs");
    let prepare_runtime_submission =
        include_str!("../../../../graphics/extract/prepare_runtime_submission/prepare.rs");
    let submit_frame_extract =
        include_str!("../../../../graphics/runtime/framework/extract/submit/submit.rs");
    let submit_runtime_frame =
        include_str!("../../../../graphics/runtime/extract/submit/submit_runtime_frame.rs");
    let present_frame_extract =
        include_str!("../../../../graphics/runtime/extract/submit/present_frame_extract.rs");
    let review_findings = history(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let _advanced_doc = include_str!(
        "../../../../../../docs/crates/zircon_runtime/core/framework/render/advanced.md"
    );

    for required in [
        "pub(super) fn validate_viewport_generation",
        "RenderFrameworkError::UnknownViewport",
        "RenderFrameworkError::ViewportChanged",
        "pub(super) fn viewport_record_mut_after_generation_check",
        "viewport_record_mut_after_generation_check_in(&mut state.viewports, viewport, context)",
        "if actual_generation != context.viewport_generation()",
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

    for (label, source) in [
        ("submit generated frame", submit_frame_extract),
        ("submit direct runtime frame", submit_runtime_frame),
        ("present generated frame", present_frame_extract),
    ] {
        let production = production_source(source);
        let validation = production
            .find("validate_viewport_generation(&state, viewport, &context)")
            .expect("generation admission");
        let writeback = production
            .find("viewport_record_mut_after_generation_check(&mut state, viewport, &context)?")
            .expect("checked writeback");
        assert!(
            validation < writeback,
            "{label} must validate before writeback"
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

fn production_source(source: &str) -> String {
    production_section(source)
}
