use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_scene_world_render_visibility_input_is_child_owner() {
    let parent = read_runtime_src("scene/world/render.rs");
    let world_mod = read_runtime_src("scene/world/mod.rs");
    let visibility_owner = read_runtime_src("scene/world/render_visibility.rs");
    let plan_09 = read_repo(
        "docs/plans/zircon_runtime/render/09/2026-07-09-camera-render-ordering-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let render_extract_doc = read_repo("docs/zircon_runtime/scene/render_extract.md");
    let visibility_doc = read_repo("docs/zircon_runtime/graphics/visibility.md");

    assert_contains_all(
        "world module mounts the visibility input owner",
        &world_mod,
        &[
            "mod render;",
            "mod render_visibility;",
            "mod render_particles;",
        ],
    );
    assert_contains_all(
        "render parent delegates visibility input construction",
        &parent,
        &[
            "use super::render_visibility::{build_visibility_input, empty_visibility_input};",
            "let visibility = build_visibility_input(&meshes, &sprites, &particles);",
            "visibility: empty_visibility_input()",
        ],
    );
    for moved_owner in [
        "fn build_visibility_input",
        "fn particle_emitter_render_layer_masks",
        "fn empty_visibility_input",
        "BTreeMap<crate::scene::EntityId, RenderLayerSet>",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "scene/world/render.rs should delegate {moved_owner} to render_visibility.rs"
        );
    }
    assert_contains_all(
        "visibility input child owns scene visibility DTO construction",
        &visibility_owner,
        &[
            "pub(super) fn build_visibility_input",
            "fn particle_emitter_render_layer_masks",
            "pub(super) fn empty_visibility_input",
            "VisibilityRenderableInput",
            "RenderLayerSet::union",
        ],
    );

    for (path, source) in [
        ("scene/world/render.rs", parent.as_str()),
        (
            "scene/world/render_visibility.rs",
            visibility_owner.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 900,
            "{path} should stay below the near-threshold render owner budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 09", plan_09.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("scene render-extract doc", render_extract_doc.as_str()),
        ("visibility doc", visibility_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Plan 09 CO-M4 world visibility input owner split",
                "render_plan09_world_visibility_input_owner_split_static_passed_cargo_timeout_no_result",
                "scene/world/render_visibility.rs",
                "runtime_15_scene_world_render_visibility_input_is_child_owner",
            ],
        );
    }
}
