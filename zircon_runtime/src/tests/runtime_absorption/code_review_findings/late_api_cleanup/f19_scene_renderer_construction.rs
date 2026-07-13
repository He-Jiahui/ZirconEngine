#[test]
fn review_f19_scene_renderer_construction_modules_use_construct_names() {
    let core_mod = include_str!("../../../../graphics/scene/scene_renderer/core/mod.rs");
    let core_construct_mod = include_str!(
        "../../../../graphics/scene/scene_renderer/core/scene_renderer_core_construct/mod.rs"
    );
    let core_construct_layouts = include_str!(
        "../../../../graphics/scene/scene_renderer/core/scene_renderer_core_construct/layouts/mod.rs"
    );
    let core_construct_scene_bind_group = include_str!(
        "../../../../graphics/scene/scene_renderer/core/scene_renderer_core_construct/scene_bind_group_bundle/mod.rs"
    );
    let renderer_construct_mod = include_str!(
        "../../../../graphics/scene/scene_renderer/core/scene_renderer_construct/mod.rs"
    );
    let renderer_construct = include_str!(
        "../../../../graphics/scene/scene_renderer/core/scene_renderer_construct/construct.rs"
    );
    let renderer_construct_new_with_icon_source = include_str!(
        "../../../../graphics/scene/scene_renderer/core/scene_renderer_construct/new_with_icon_source.rs"
    );
    let review_findings = concat!(
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md"),
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md")
    );
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let render_index = include_str!("../../../../../../docs/plans/zircon_runtime/render/index.md");
    let runtime_15 = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let shadow_doc = include_str!(
        "../../../../../../docs/zircon_runtime/graphics/scene/scene_renderer/shadow.md"
    );
    let f19_row = review_findings
        .lines()
        .find(|line| line.starts_with("| F19 |"))
        .expect("F19 review findings top row");

    assert!(
        f19_row.contains("scene renderer construction owner")
            && f19_row.ends_with("| Runtime 15 + render |"),
        "F19 overview row should keep only the finding and delegated owners"
    );
    assert!(
        review_findings.contains(
            "f19_scene_renderer_construction_top_row_closed_status_static_passed_cargo_deferred"
        ),
        "F19 numbered output should record scene renderer construction naming review closed status"
    );

    let core_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/graphics/scene/scene_renderer/core");
    let old_core_construct_owner = ["scene_renderer_core", "new"].join("_");
    let old_renderer_construct_owner = ["scene_renderer", "new"].join("_");
    assert!(
        core_dir.join("scene_renderer_core_construct").is_dir()
            && core_dir.join("scene_renderer_construct").is_dir(),
        "F19 scene renderer construction owners should live in construct-named directories"
    );
    assert!(
        !core_dir.join(&old_core_construct_owner).exists()
            && !core_dir.join(&old_renderer_construct_owner).exists(),
        "F19 should hard-cut old *_new construction directories instead of keeping migration paths"
    );

    for required in [
        "mod scene_renderer_core_construct;",
        "mod scene_renderer_construct;",
    ] {
        assert!(
            core_mod.contains(required),
            "scene renderer core module wiring should contain `{required}`"
        );
    }

    for (name, source) in [
        ("core/mod.rs", core_mod),
        ("scene_renderer_core_construct/mod.rs", core_construct_mod),
        (
            "scene_renderer_core_construct/layouts/mod.rs",
            core_construct_layouts,
        ),
        (
            "scene_renderer_core_construct/scene_bind_group_bundle/mod.rs",
            core_construct_scene_bind_group,
        ),
        ("scene_renderer_construct/mod.rs", renderer_construct_mod),
        ("scene_renderer_construct/construct.rs", renderer_construct),
        (
            "scene_renderer_construct/new_with_icon_source.rs",
            renderer_construct_new_with_icon_source,
        ),
    ] {
        for forbidden in [&old_core_construct_owner, &old_renderer_construct_owner] {
            assert!(
                !source.contains(forbidden),
                "F19 should not leave old construction owner `{forbidden}` in {name}"
            );
        }
    }

    for doc_anchor in [
        "F19 scene renderer construction module rename",
        "render_scene_renderer_construct_modules_coremin_passed",
        "f19_scene_renderer_construction_top_row_closed_status_static_passed_cargo_deferred",
        "review_f19_scene_renderer_construction_modules_use_construct_names",
        "scene_renderer_core_construct",
        "scene_renderer_construct",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || render_index.contains(doc_anchor)
                || runtime_15.contains(doc_anchor)
                || shadow_doc.contains(doc_anchor),
            "F19 docs should record `{doc_anchor}`"
        );
    }
}
