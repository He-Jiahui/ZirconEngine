use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_rhi_wgpu_ui_surface_render_setup_are_child_owners() {
    let parent = read_runtime_src("rhi_wgpu/ui_surface.rs");
    let render_pass = read_runtime_src("rhi_wgpu/ui_surface/render_pass.rs");
    let surface_setup = read_runtime_src("rhi_wgpu/ui_surface/surface_setup.rs");
    let tests = read_runtime_src("rhi_wgpu/ui_surface/tests.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let rhi_ui_doc = read_repo("docs/zircon_runtime/rhi/ui_surface.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
    );

    assert_contains_all(
        "WGPU UI surface parent keeps presenter, renderer lifecycle, and image cache responsibilities",
        &parent,
        &[
            "mod render_pass;",
            "mod surface_setup;",
            "use render_pass::{record_draw_ops_to_view, TargetLoad, WgpuUiDrawBuffers};",
            "use surface_setup::{configure_surface, create_surface, instance_descriptor, request_device};",
            "fn present(",
            "fn render_draw_list_to_surface",
            "struct WgpuUiImageResource",
            "fn image_cache_keys_to_prune",
            "#[cfg(test)]",
            "mod tests;",
        ],
    );
    for moved_owner in [
        "struct WgpuUiDrawBuffers",
        "fn begin_ui_surface_pass",
        "fn set_surface_viewport",
        "fn choose_surface_format",
        "fn choose_present_mode",
        "fn required_nonzero_isize",
        "SurfaceTargetUnsafe::RawHandle",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "rhi_wgpu/ui_surface.rs should delegate {moved_owner} to UI surface child owners"
        );
    }
    assert_contains_all(
        "render-pass child owns draw buffers and render-pass recording",
        &render_pass,
        &[
            "pub(super) struct WgpuUiDrawBuffers",
            "pub(super) enum TargetLoad",
            "pub(super) fn record_draw_ops_to_view",
            "fn begin_ui_surface_pass",
            "fn set_surface_viewport",
            "super::batching::DrawOp",
            "super::geometry::{ImageVertex, SolidVertex}",
        ],
    );
    assert_contains_all(
        "surface setup child owns device, swapchain, and raw surface setup",
        &surface_setup,
        &[
            "pub(super) fn configure_surface",
            "pub(super) fn request_device",
            "pub(super) fn instance_descriptor",
            "pub(super) fn create_surface",
            "fn choose_present_mode",
            "fn required_nonzero_isize",
            "SurfaceTargetUnsafe::RawHandle",
        ],
    );
    assert_contains_all(
        "tests child keeps WGPU UI surface focused behavior coverage",
        &tests,
        &[
            "fn wgpu_ui_surface_presenter_records_present_stats",
            "fn wgpu_ui_surface_prefers_opaque_swapchain_alpha",
            "fn wgpu_ui_surface_uses_non_srgb_formats_for_byte_exact_editor_parity",
            "fn wgpu_ui_surface_image_cache_prune_keeps_recent_entries",
            "image_cache_keys_to_prune",
        ],
    );

    for (path, source) in [
        ("rhi_wgpu/ui_surface.rs", parent.as_str()),
        ("rhi_wgpu/ui_surface/render_pass.rs", render_pass.as_str()),
        (
            "rhi_wgpu/ui_surface/surface_setup.rs",
            surface_setup.as_str(),
        ),
        ("rhi_wgpu/ui_surface/tests.rs", tests.as_str()),
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
        ("RHI UI surface doc", rhi_ui_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M4 RHI WGPU UI surface render/setup owner split",
                "runtime_15_rhi_wgpu_ui_surface_render_setup_owner_split_static_passed_cargo_timeout_no_result",
                "rhi_wgpu/ui_surface.rs",
                "rhi_wgpu/ui_surface/render_pass.rs",
                "rhi_wgpu/ui_surface/surface_setup.rs",
                "rhi_wgpu/ui_surface/tests.rs",
                "runtime_15_rhi_wgpu_ui_surface_render_setup_are_child_owners",
            ],
        );
    }
    assert_contains_all(
        "status-output row data",
        &status_rows,
        &[
            "Runtime 15 M4 RHI WGPU UI surface render/setup owner split",
            "runtime_15_rhi_wgpu_ui_surface_render_setup_owner_split_static_passed_cargo_timeout_no_result",
            "rhi_wgpu/ui_surface.rs",
            "rhi_wgpu/ui_surface/render_pass.rs",
            "runtime_15_rhi_wgpu_ui_surface_render_setup_are_child_owners",
        ],
    );
}
