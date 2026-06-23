use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_scene_fixed_light_reflection_write_fields_are_child_owner() {
    let parent = read_runtime_src("scene/reflect/fixed/lights.rs");
    let write_fields = read_runtime_src("scene/reflect/fixed/lights/write_fields.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let reflect_doc = read_repo("docs/zircon_runtime/scene/reflect.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
    );

    assert_contains_all(
        "fixed light reflection parent keeps registration and read/remove responsibilities",
        &parent,
        &[
            "mod write_fields;",
            "use self::write_fields::{",
            "ambient_adapter()",
            "directional_adapter()",
            "point_adapter()",
            "rect_adapter()",
            "spot_adapter()",
            "fn ambient_read_field",
            "fn spot_read_fields",
            "fn ambient_remove",
            "fn spot_remove",
        ],
    );
    for moved_owner in [
        "fn ambient_write_field",
        "fn spot_write_field",
        "fn write_ambient_vec3",
        "fn write_rect_vec2",
        "shared::expect_vec3(",
        "shared::expect_vec2(",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "scene/reflect/fixed/lights.rs should delegate {moved_owner} to lights/write_fields.rs"
        );
    }
    assert_contains_all(
        "fixed light write-field child owns editable field mutation",
        &write_fields,
        &[
            "pub(super) fn ambient_write_field",
            "pub(super) fn directional_write_field",
            "pub(super) fn point_write_field",
            "pub(super) fn rect_write_field",
            "pub(super) fn spot_write_field",
            "fn write_ambient_vec3",
            "fn write_rect_vec2",
            "shared::ensure_component::<AmbientLight>",
            "shared::expect_vec3(",
            "shared::expect_vec2(",
            "shared::expect_scalar(",
        ],
    );

    for (path, source) in [
        ("scene/reflect/fixed/lights.rs", parent.as_str()),
        (
            "scene/reflect/fixed/lights/write_fields.rs",
            write_fields.as_str(),
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
        ("scene reflect doc", reflect_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M4 scene fixed light reflection write-field owner split",
                "runtime_15_scene_fixed_light_reflection_write_fields_owner_split_static_passed_cargo_lock_blocked",
                "scene/reflect/fixed/lights.rs",
                "scene/reflect/fixed/lights/write_fields.rs",
                "runtime_15_scene_fixed_light_reflection_write_fields_are_child_owner",
            ],
        );
    }
}
