use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_rhi_wgpu_command_validation_state_is_child_owner() {
    let parent = read_runtime_src("rhi_wgpu/command_validation.rs");
    let render_state = read_runtime_src("rhi_wgpu/command_validation/render_state.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let rhi_doc = read_repo("docs/zircon_runtime/rhi/descriptors.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
    );

    assert_contains_all(
        "command validation parent keeps command-list entry points",
        &parent,
        &[
            "mod render_state;",
            "use self::render_state::{",
            "pub(super) fn validate_recorded_commands",
            "pub(super) fn execute_recorded_commands",
            "CommandListCommand::Draw",
            "CommandListCommand::DrawIndexed",
            "CommandListCommand::DispatchCompute",
        ],
    );
    for moved_owner in [
        "struct RecordedRenderState",
        "trait CommandResourceLookup",
        "fn validate_strided_binding_range",
        "fn pipeline_layout_for_command",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "rhi_wgpu/command_validation.rs should delegate {moved_owner} to render_state.rs"
        );
    }
    assert_contains_all(
        "render-state child owns recorded render state validation",
        &render_state,
        &[
            "pub(super) struct RecordedRenderState",
            "pub(super) trait CommandResourceLookup",
            "pub(super) fn validate_bind_group_slot",
            "pub(super) fn ensure_binding_range",
            "pub(super) fn validate_index_range",
            "fn validate_strided_binding_range",
            "fn pipeline_layout_for_command",
        ],
    );

    for (path, source) in [
        ("rhi_wgpu/command_validation.rs", parent.as_str()),
        (
            "rhi_wgpu/command_validation/render_state.rs",
            render_state.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("RHI descriptor doc", rhi_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M4 RHI WGPU command validation render-state owner split",
                "runtime_15_rhi_wgpu_command_validation_render_state_split_static_passed_cargo_lock_blocked",
                "rhi_wgpu/command_validation.rs",
                "rhi_wgpu/command_validation/render_state.rs",
                "runtime_15_rhi_wgpu_command_validation_state_is_child_owner",
            ],
        );
    }
}
