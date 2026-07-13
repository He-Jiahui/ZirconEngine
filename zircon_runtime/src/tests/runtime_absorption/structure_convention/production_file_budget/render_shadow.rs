use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_shadow_plan_view_projection_is_child_owner() {
    let shadow_mod = read_runtime_src("graphics/scene/scene_renderer/shadow/mod.rs");
    let parent = read_runtime_src("graphics/scene/scene_renderer/shadow/plan.rs");
    let view_projection =
        read_runtime_src("graphics/scene/scene_renderer/shadow/view_projection.rs");
    let plan_05 = read_repo(
        "docs/plans/zircon_runtime/render/05/2026-07-09-lighting-shadows-output-records.md",
    );
    let plan_09 = read_repo(
        "docs/plans/zircon_runtime/render/09/2026-07-09-camera-render-ordering-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let shadow_doc = read_repo("docs/zircon_runtime/graphics/scene/scene_renderer/shadow.md");

    assert_contains_all(
        "shadow module mounts plan and view-projection owners",
        &shadow_mod,
        &["mod plan;", "mod view_projection;"],
    );
    assert_contains_all(
        "shadow plan delegates view-projection construction",
        &parent,
        &[
            "use super::view_projection::{",
            "directional_cascade_view_projection",
            "point_light_face_view_projection",
            "spot_light_view_projection",
        ],
    );
    for moved_owner in [
        "fn directional_cascade_view_projection",
        "fn spot_light_view_projection",
        "fn point_light_face_view_projection",
        "fn point_light_face_axes",
        "fn sanitize_direction",
        "SHADOW_CAMERA_DISTANCE_SCALE",
        "DEFAULT_SHADOW_LIGHT_DIRECTION_COMPONENTS",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "shadow/plan.rs should delegate {moved_owner} to view_projection.rs"
        );
    }
    assert_contains_all(
        "view-projection child owns shadow camera matrices and sanitizing",
        &view_projection,
        &[
            "pub(super) fn directional_cascade_view_projection",
            "pub(super) fn spot_light_view_projection",
            "pub(super) fn point_light_face_view_projection",
            "fn point_light_face_axes",
            "fn sanitize_direction",
            "cascade_shadow_bounds_from_camera_slice",
            "snapped_cascade_view_projection",
            "Mat4::perspective_rh",
            "Transform::looking_at",
        ],
    );

    for (path, source) in [
        (
            "graphics/scene/scene_renderer/shadow/plan.rs",
            parent.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/shadow/view_projection.rs",
            view_projection.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 900,
            "{path} should stay below the near-threshold render owner budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 05", plan_05.as_str()),
        ("Plan 09", plan_09.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("shadow module doc", shadow_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Plan 05/09 shadow view-projection owner split",
                "render_plan05_09_shadow_view_projection_owner_split_static_passed",
                "graphics/scene/scene_renderer/shadow/view_projection.rs",
                "runtime_15_shadow_plan_view_projection_is_child_owner",
            ],
        );
    }
}
