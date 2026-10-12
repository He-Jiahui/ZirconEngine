use super::super::assert_contains_all;
use super::{read_repo, read_runtime_src, DEAD_CODE_ALLOW_ATTRIBUTE};

#[test]
fn runtime_15_particle_gpu_readback_output_accessor_cleanup() {
    let take_particle_readback = read_runtime_src(
        "graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/take_last_particle_gpu_readback_outputs.rs",
    );
    let collect_feedback = read_runtime_src(
        "graphics/runtime/render_framework/submit_frame_extract/submit/collect_runtime_feedback.rs",
    );
    let runtime_15_plan =
        read_repo(
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
        !take_particle_readback.contains(DEAD_CODE_ALLOW_ATTRIBUTE),
        "SceneRenderer particle GPU readback output accessor is consumed by runtime feedback and should not keep a dead-code suppression"
    );
    assert_contains_all(
        "particle GPU readback output accessor remains the renderer output drain",
        &take_particle_readback,
        &[
            "pub(in crate::graphics) fn take_last_particle_gpu_readback_outputs(",
            ") -> RenderParticleGpuReadbackOutputs",
            ".take_particle_gpu_readback_outputs()",
        ],
    );
    assert_contains_all(
        "runtime feedback consumes the particle GPU readback accessor",
        &collect_feedback,
        &[
            "fn collect_particle_feedback(",
            "renderer.take_last_particle_gpu_readback_outputs()",
            "sidebands.take_particle_readback_outputs()",
            "ParticleGpuFeedback::new(readback_outputs)",
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
                "Runtime 15 F12 particle GPU readback output accessor cleanup",
                "runtime_15_particle_gpu_readback_output_accessor_cleanup_static_passed_cargo_lock_blocked",
                "runtime_15_particle_gpu_readback_output_accessor_cleanup",
            ],
        );
    }
}

#[test]
fn runtime_15_advanced_plugin_output_test_accessor_cleanup() {
    let output_access = read_runtime_src(
        "graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_access.rs",
    );
    let output_storage = read_runtime_src(
        "graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_storage.rs",
    );
    let collect_into_outputs = read_runtime_src(
        "graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_readbacks/collect_into_outputs.rs",
    );
    let runtime_15_plan =
        read_repo(
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
    let particles_doc = read_repo("docs/zircon_plugins/particles/runtime.md");

    assert!(
        !output_access.contains(DEAD_CODE_ALLOW_ATTRIBUTE),
        "advanced plugin output observation helpers should be test-only or production-live, not dead-code suppressions"
    );

    let output_access = output_access.replace("\r\n", "\n");
    for accessor in [
        "has_virtual_geometry_gpu_readback",
        "plugin_renderer_outputs",
        "has_particle_gpu_readback",
    ] {
        let expected = format!(
            "#[cfg(test)]\n    pub(in crate::graphics::scene::scene_renderer::core) fn {accessor}("
        );
        assert!(
            output_access.contains(&expected),
            "{accessor} should be a test-only observation helper"
        );
    }
    assert_contains_all(
        "production advanced plugin output drains remain live",
        &output_access,
        &[
            "pub(in crate::graphics::scene::scene_renderer::core) fn take_hybrid_gi_readback_outputs(",
            "pub(in crate::graphics::scene::scene_renderer::core) fn take_particle_gpu_readback_outputs(",
            "pub(in crate::graphics::scene::scene_renderer::core) fn take_virtual_geometry_readback_outputs(",
            "std::mem::take(&mut self.plugin_renderer_outputs_mut().particles)",
        ],
    );
    assert_contains_all(
        "advanced plugin output tests still exercise mailbox observation helpers",
        &output_storage,
        &[
            "fn stores_neutral_plugin_renderer_outputs()",
            "outputs.plugin_renderer_outputs()",
            "outputs.has_virtual_geometry_gpu_readback()",
            "outputs.has_particle_gpu_readback()",
        ],
    );
    assert_contains_all(
        "readback collection tests still inspect neutral plugin renderer outputs",
        &collect_into_outputs,
        &[
            "fn advanced_plugin_readbacks_collect_neutral_plugin_renderer_outputs()",
            "outputs",
            ".plugin_renderer_outputs()",
            ".particles",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("render product doc", render_product_doc.as_str()),
        ("particles runtime doc", particles_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 F12 advanced plugin output test accessor cleanup",
                "runtime_15_advanced_plugin_output_test_accessor_cleanup_static_passed_cargo_lock_blocked",
                "runtime_15_advanced_plugin_output_test_accessor_cleanup",
            ],
        );
    }
}
