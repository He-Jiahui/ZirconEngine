use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_rhi_wgpu_ui_surface_render_setup_are_child_owners() {
    let parent = read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface.rs");
    let batching = read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/batching.rs");
    let batching_tests =
        read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/batching/tests.rs");
    let image_cache = read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/image_cache.rs");
    let image_resource =
        read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/image_cache/resource.rs");
    let presentation =
        read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/presentation.rs");
    let render_pass = read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/render_pass.rs");
    let retained_cache =
        read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/retained_cache.rs");
    let surface_setup =
        read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/surface_setup.rs");
    let tests = read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/tests.rs");
    let native_submission_tests =
        read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/tests/native_submission.rs");
    let text = read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/text.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let rhi_ui_doc = read_repo("docs/zircon_runtime/rhi/ui_surface.md");

    assert_contains_all(
        "WGPU UI surface parent keeps presenter and renderer resource lifecycle responsibilities",
        &parent,
        &[
            "mod render_pass;",
            "mod retained_cache;",
            "mod surface_setup;",
            "mod text;",
            "use render_pass::{",
            "WgpuUiDrawBufferCache",
            "WgpuUiDrawBufferStats",
            "WgpuUiRecordedDrawStats",
            "stats.render_pass_count",
            "stats.retained_cache_copy_bytes",
            "use retained_cache::WgpuRetainedSurfaceCache;",
            "use surface_setup::{configure_surface, create_surface, instance_descriptor, request_device};",
            "use text::{WgpuUiTextPrepareStats, WgpuUiTextRenderer};",
            "fn present(",
            "mod presentation;",
            "use image_cache::WgpuUiImageCache;",
            "#[cfg(test)]",
            "mod tests;",
        ],
    );
    for moved_owner in [
        "struct WgpuUiDrawBuffers",
        "fn begin_ui_surface_pass",
        "fn render_draw_list_to_surface",
        "record_draw_ops_to_view",
        "enum TargetLoad",
        "struct WgpuUiImageResource",
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
        "image-cache child owns admission, prepare, and O(1) residency accounting",
        &image_cache,
        &[
            "mod resource;",
            "pub(super) use resource::WgpuUiImageResource;",
            "pub(super) fn residency_stats",
            "fn admit",
            "fn invalidate",
        ],
    );
    assert_contains_all(
        "image-resource child owns texture and bind-group construction",
        &image_resource,
        &[
            "pub(in super::super) struct WgpuUiImageResource",
            "pub(in super::super) bind_group: wgpu::BindGroup",
            "pub(super) fn new",
            "pub(super) fn from_external",
            "create_texture",
            "create_bind_group",
        ],
    );
    assert_contains_all(
        "presentation child owns native submission and retained resize projection reuse",
        &presentation,
        &[
            "fn render_draw_list_to_surface",
            "RetainedProjectionCopy",
            "fn surface_render_mode",
            "fn render_damage",
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
            "enum RetainedSurfaceState",
            "pub(super) fn ordinary_baseline_ready",
            "pub(super) fn is_projection_ready",
            "pub(super) fn matches",
            "pub(super) fn mark_ordinary_baseline_ready",
            "pub(super) fn mark_projection_ready",
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
            "fn target_only_resize_uses_retained_copy_only_after_the_projection_is_ready",
            "fn wgpu_ui_surface_image_cache_admission_evicts_the_oldest_inactive_entry",
            "mod native_submission;",
        ],
    );
    assert_contains_all(
        "native-submission tests child keeps WGPU submit and shared-device boundary coverage",
        &native_submission_tests,
        &[
            "fn wgpu_ui_surface_marks_the_complete_present_submission_for_renderdoc",
            "fn wgpu_ui_surface_shared_context_path_does_not_request_a_second_device",
            "fn wgpu_ui_surface_external_image_path_uses_the_shared_texture_without_cpu_upload",
            "fn wgpu_ui_surface_presents_submitted_frame_before_readback_error",
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
        ("rhi_wgpu/ui_surface/image_cache.rs", image_cache.as_str()),
        (
            "rhi_wgpu/ui_surface/image_cache/resource.rs",
            image_resource.as_str(),
        ),
        ("rhi_wgpu/ui_surface/presentation.rs", presentation.as_str()),
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
        (
            "rhi_wgpu/ui_surface/tests/native_submission.rs",
            native_submission_tests.as_str(),
        ),
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
        "RHI UI surface current owner and resize projection contract",
        &rhi_ui_doc,
        &[
            "ui_surface/presentation.rs",
            "generation-tagged native-resize projection",
            "ordinary damage baseline",
            "copy-only resize frames",
            "runtime_15_rhi_wgpu_ui_surface_render_setup_are_child_owners",
        ],
    );
}
