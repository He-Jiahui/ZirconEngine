use super::super::assert_contains_all;
use super::{read_repo, read_runtime_src};

#[test]
fn runtime_15_material_runtime_capture_seed_cleanup() {
    let material_runtime = read_runtime_src("graphics/scene/resources/runtime/material_runtime.rs");
    let runtime_mod = read_runtime_src("graphics/scene/resources/runtime/mod.rs");
    let resources_mod = read_runtime_src("graphics/scene/resources/mod.rs");
    let resource_streamer_accessors = read_runtime_src(
        "graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs",
    );
    let material_capture = read_runtime_src(
        "graphics/scene/resources/resource_streamer/resource_streamer_accessors/material_capture.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let render_product_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert!(
        !material_runtime.contains("#[allow(dead_code)]"),
        "MaterialRuntime and MaterialCaptureSeed should not hide production dead-code surfaces behind suppressions"
    );
    assert_contains_all(
        "material runtime capture seed is test-only",
        &material_runtime,
        &[
            "#[cfg(test)]",
            "pub(crate) struct MaterialCaptureSeed",
            "impl MaterialRuntime",
            "pub(crate) fn capture_seed(&self) -> MaterialCaptureSeed",
        ],
    );
    assert_contains_all(
        "material capture seed re-export stays behind test cfg",
        &runtime_mod,
        &[
            "pub(crate) use material_runtime::MaterialRuntime;",
            "#[cfg(test)]",
            "pub(crate) use material_runtime::MaterialCaptureSeed;",
        ],
    );
    assert_contains_all(
        "resources facade keeps production material runtime separate from test capture seed",
        &resources_mod,
        &[
            "pub(crate) use runtime::MaterialRuntime;",
            "#[cfg(test)]",
            "pub(crate) use runtime::MaterialCaptureSeed;",
        ],
    );
    assert_contains_all(
        "resource streamer capture seed accessor is test-only",
        &resource_streamer_accessors,
        &["#[cfg(test)]", "mod material_capture;"],
    );
    assert_contains_all(
        "resource streamer material capture accessors are child-owned",
        &material_capture,
        &[
            "use super::super::super::MaterialCaptureSeed;",
            "pub(crate) fn material_capture_seed(",
            "pub(crate) fn sample_texture_rgba(",
            "fn sample_texture_asset_rgba(",
            "fn wrap01(",
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
                "Runtime 15 F12 material runtime capture seed cleanup",
                "runtime_15_material_runtime_capture_seed_cleanup_coremin_check_passed",
                "runtime_15_material_runtime_capture_seed_cleanup",
            ],
        );
    }
}

#[test]
fn runtime_15_resource_streamer_diagnostics_accessor_cleanup() {
    let resource_streamer_accessors = read_runtime_src(
        "graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs",
    );
    let material_capture = read_runtime_src(
        "graphics/scene/resources/resource_streamer/resource_streamer_accessors/material_capture.rs",
    );
    let resource_streamer_ensure = read_runtime_src(
        "graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let render_product_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert!(
        !resource_streamer_accessors.contains("#[allow(dead_code)]"),
        "ResourceStreamer diagnostics accessors should be test-only or production-live, not dead-code suppressions"
    );
    assert_contains_all(
        "test-only asset and material diagnostics accessors",
        &resource_streamer_accessors,
        &[
            "#[cfg(test)]",
            "pub(crate) fn model_asset_overview(",
            "pub(crate) fn asset_management_record_sets(",
            "pub(crate) fn material_uniform_payload_byte_len(",
            "pub(crate) fn material_management_record_set(",
            "pub(crate) fn material_prepared_state(",
        ],
    );
    assert_contains_all(
        "resource streamer accessors delegates material capture helpers to child owner",
        &resource_streamer_accessors,
        &["#[cfg(test)]", "mod material_capture;"],
    );
    for moved_helper in [
        "pub(crate) fn material_capture_seed(",
        "pub(crate) fn sample_texture_rgba(",
        "fn shading_model_id_for_lighting_model(",
        "fn sample_texture_asset_rgba(",
        "fn wrap01(",
    ] {
        assert!(
            !resource_streamer_accessors.contains(moved_helper),
            "resource_streamer_accessors.rs should delegate `{moved_helper}` to material_capture.rs"
        );
        assert!(
            material_capture.contains(moved_helper),
            "material_capture.rs should own `{moved_helper}`"
        );
    }
    for (path, source) in [
        (
            "graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs",
            resource_streamer_accessors.as_str(),
        ),
        (
            "graphics/scene/resources/resource_streamer/resource_streamer_accessors/material_capture.rs",
            material_capture.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 owner budget after the material capture split; got {line_count}"
        );
    }
    assert_contains_all(
        "production material readiness accessor remains live",
        &resource_streamer_accessors,
        &[
            "pub(crate) fn material_readiness_report(",
            "pub(crate) fn material_readiness_summary(",
            "self.material_readiness_report(id)",
        ],
    );
    assert_contains_all(
        "scene resource ensure path consumes production readiness summary",
        &resource_streamer_ensure,
        &[
            "if let Some(summary) = self.material_readiness_summary(&material_id)",
            "summary.is_ready",
            "summary.uses_fallback",
            "summary.validation_error_count",
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
                "Runtime 15 F12 resource streamer diagnostics accessor cleanup",
                "runtime_15_resource_streamer_diagnostics_accessor_cleanup_static_passed_cargo_lock_blocked",
                "runtime_15_resource_streamer_material_capture_child_owner_static_passed_cargo_deferred_implementation_cadence",
                "runtime_15_resource_streamer_diagnostics_accessor_cleanup",
            ],
        );
    }
}

#[test]
fn runtime_15_resource_streamer_resolve_texture_id_cleanup() {
    let resolve_texture = read_runtime_src(
        "graphics/scene/resources/resource_streamer/resource_streamer_resolve_texture_id.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let render_product_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert!(
        !resolve_texture.contains("#[allow(dead_code)]"),
        "ResourceStreamer texture-reference resolution should not hide unused helpers behind dead-code suppression"
    );
    assert!(
        !resolve_texture.contains("fn resolve_texture_id("),
        "the unused ResourceStreamer::resolve_texture_id helper should stay retired"
    );
    assert_contains_all(
        "production texture-reference resolution entry points remain live",
        &resolve_texture,
        &[
            "pub(in crate::graphics::scene::resources) fn resolve_texture_reference(",
            "pub(in crate::graphics::scene::resources) fn resolve_texture_reference_with_support(",
            "pub(in crate::graphics::scene::resources) fn id(&self) -> Option<ResourceId>",
            "RenderMaterialValidationError::TextureNotUploadReady",
            "RenderMaterialTextureSlotFallback::not_upload_ready",
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
                "Runtime 15 F12 resource streamer resolve texture id cleanup",
                "runtime_15_resource_streamer_resolve_texture_id_cleanup_static_passed_cargo_lock_blocked",
                "runtime_15_resource_streamer_resolve_texture_id_cleanup",
            ],
        );
    }
}
