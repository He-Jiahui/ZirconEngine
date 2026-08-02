use super::super::assert_contains_all;
use super::{read_repo, read_runtime_src, DEAD_CODE_ALLOW_ATTRIBUTE};

#[test]
fn runtime_15_offscreen_target_texture_owner_cleanup() {
    let offscreen_target = read_runtime_src("graphics/backend/render_backend/offscreen_target.rs");
    let offscreen_construct =
        read_runtime_src("graphics/backend/render_backend/offscreen_target_construct/construct.rs");
    let frame_graph_binder = read_runtime_src(
        "graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_frame_graph_resources.rs",
    );
    let runtime_15_plan = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    let runtime_index = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let render_product_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert!(
        !offscreen_target.contains(DEAD_CODE_ALLOW_ATTRIBUTE),
        "OffscreenTarget texture owners should be live ownership contracts, not dead-code suppressions"
    );
    assert_contains_all(
        "offscreen retained WGPU texture owners",
        &offscreen_target,
        &[
            "pub(crate) const RETAINED_FRAME_TEXTURE_COUNT: usize = 10;",
            "pub(crate) fn retained_frame_texture_count(&self) -> usize",
            "&self.final_color",
            "&self.global_illumination",
            "&self.scene_color",
            "&self.bloom",
            "&self.gbuffer_albedo",
            "&self.gbuffer_emissive",
            "&self.gbuffer_material",
            "&self.normal",
            "&self.ambient_occlusion",
            "&self.depth",
        ],
    );
    assert_contains_all(
        "offscreen construction still materializes every retained owner",
        &offscreen_construct,
        &[
            "final_color: final_color.texture",
            "global_illumination: global_illumination.texture",
            "scene_color: scene_color.texture",
            "bloom: bloom.texture",
            "gbuffer_albedo: gbuffer_albedo.texture",
            "gbuffer_emissive: gbuffer_emissive.texture",
            "gbuffer_material: gbuffer_material.texture",
            "normal: normal.texture",
            "ambient_occlusion: ambient_occlusion.texture",
            "depth,",
        ],
    );
    assert_contains_all(
        "compiled-scene frame graph binder consumes retained owner contract",
        &frame_graph_binder,
        &[
            "target.retained_frame_texture_count()",
            "OffscreenTarget::RETAINED_FRAME_TEXTURE_COUNT",
            "fixed offscreen frame target must retain every WGPU texture owner backing imported views",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("render product doc", render_product_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 F12 offscreen target texture owner cleanup",
                "runtime_15_offscreen_target_texture_owner_cleanup_static_passed_cargo_timeout_no_result",
                "runtime_15_offscreen_target_texture_owner_cleanup",
            ],
        );
    }
}

#[test]
fn runtime_15_render_backend_state_owner_cleanup() {
    let render_backend = read_runtime_src("graphics/backend/render_backend/render_backend.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let render_product_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert!(
        !render_backend.contains(DEAD_CODE_ALLOW_ATTRIBUTE),
        "RenderBackend state owners should be live ownership contracts, not dead-code suppressions"
    );
    assert_contains_all(
        "render backend retained state owner contract",
        &render_backend,
        &[
            "pub(crate) const RETAINED_STATE_OWNER_COUNT: usize = 3;",
            "pub(crate) fn retained_state_owner_count(&self) -> usize",
            "&self.instance",
            "&self.adapter",
            "&self.config",
            "self.retained_state_owner_count()",
            "RenderBackend must retain instance, adapter, and config owners while reporting caps",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("render product doc", render_product_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 F12 render backend state owner cleanup",
                "runtime_15_render_backend_state_owner_cleanup_coremin_check_passed",
                "runtime_15_render_backend_state_owner_cleanup",
            ],
        );
    }
}
