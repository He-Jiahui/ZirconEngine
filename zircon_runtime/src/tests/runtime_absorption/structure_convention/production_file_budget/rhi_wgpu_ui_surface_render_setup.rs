use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_rhi_wgpu_ui_surface_render_setup_are_child_owners() {
    let parent = read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface.rs");
    let batching = read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/batching.rs");
    let batching_tests =
        read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/batching/tests.rs");
    let render_pass = read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/render_pass.rs");
    let retained_cache =
        read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/retained_cache.rs");
    let surface_setup =
        read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/surface_setup.rs");
    let tests = read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/tests.rs");
    let text = read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/text.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let rhi_ui_doc = read_repo("docs/zircon_runtime/rhi/ui_surface.md");

    assert_contains_all(
        "WGPU UI surface parent keeps presenter, renderer lifecycle, and image cache responsibilities",
        &parent,
        &[
            "mod render_pass;",
            "mod retained_cache;",
            "mod surface_setup;",
            "mod text;",
            "use render_pass::{",
            "record_draw_ops_to_view",
            "TargetLoad",
            "WgpuUiDrawBufferCache",
            "WgpuUiDrawBufferStats",
            "WgpuUiRecordedDrawStats",
            "stats.render_pass_count",
            "stats.retained_cache_copy_bytes",
            "use retained_cache::WgpuRetainedSurfaceCache;",
            "use surface_setup::{configure_surface, create_surface, instance_descriptor, request_device};",
            "use text::{WgpuUiTextPrepareStats, WgpuUiTextRenderer};",
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
        "struct WgpuRetainedSurfaceCache",
        "struct WgpuUiTextRenderer",
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
        "batching child keeps full-projection planning while its focused tests stay external",
        &batching,
        &[
            "struct CompiledUiBatchPlanCache",
            "pub(super) struct TextDraw",
            "pub(super) solid_vertices: Vec<SolidVertex>",
            "pub(super) image_vertices: Vec<ImageVertex>",
            "#[path = \"batching/tests.rs\"]",
            "mod tests;",
        ],
    );
    assert_contains_all(
        "batching tests child keeps full-projection and damage-culling coverage",
        &batching_tests,
        &[
            "fn compiled_plan_cache_reuses_the_full_projection_for_versioned_damage",
            "fn text_batch_keeps_union_bounds_for_damage_culling",
        ],
    );
    assert_contains_all(
        "render-pass child owns draw buffers and render-pass recording",
        &render_pass,
        &[
            "pub(super) struct WgpuUiDrawBuffers",
            "pub(super) struct WgpuUiDrawBufferCache",
            "pub(super) struct WgpuUiRecordedDrawStats",
            "pub(super) enum TargetLoad",
            "pub(super) fn record_draw_ops_to_view",
            "render_pass_count",
            "retained_cache_copy_bytes",
            "bytemuck::cast_slice(draw_plan.solid_vertices.as_slice())",
            "bytemuck::cast_slice(draw_plan.image_vertices.as_slice())",
            "fn begin_ui_surface_pass",
            "fn set_surface_viewport",
            "fn damage_scissor",
            "super::batching::DrawOp",
            "super::geometry::{ImageVertex, SolidVertex}",
        ],
    );
    assert_contains_all(
        "retained-cache child owns persistent target state and surface copy",
        &retained_cache,
        &[
            "pub(super) struct WgpuRetainedSurfaceCache",
            "pub(super) fn initialized",
            "pub(super) fn matches",
            "pub(super) fn mark_initialized",
            "pub(super) fn record_copy_to_surface",
            "fn retained_copy_byte_count",
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
    assert_contains_all(
        "text child owns glyphon preparation and draw execution",
        &text,
        &[
            "pub(super) struct WgpuUiTextRenderer",
            "pub(super) fn prepare",
            "pub(super) fn render_batch",
            "fn text_has_visible_content",
        ],
    );

    for (path, source) in [
        ("rhi_wgpu/ui_surface.rs", parent.as_str()),
        ("rhi_wgpu/ui_surface/batching.rs", batching.as_str()),
        (
            "rhi_wgpu/ui_surface/batching/tests.rs",
            batching_tests.as_str(),
        ),
        ("rhi_wgpu/ui_surface/render_pass.rs", render_pass.as_str()),
        (
            "rhi_wgpu/ui_surface/retained_cache.rs",
            retained_cache.as_str(),
        ),
        (
            "rhi_wgpu/ui_surface/surface_setup.rs",
            surface_setup.as_str(),
        ),
        ("rhi_wgpu/ui_surface/tests.rs", tests.as_str()),
        ("rhi_wgpu/ui_surface/text.rs", text.as_str()),
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
}
