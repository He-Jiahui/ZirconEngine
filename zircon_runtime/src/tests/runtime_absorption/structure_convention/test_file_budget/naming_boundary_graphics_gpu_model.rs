use super::*;

const STATUS: &str =
    "runtime_15_graphics_gpu_model_guard_child_owner_split_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 graphics GPU-model guard child-owner split";
const GUARD: &str = "runtime_15_graphics_gpu_model_guard_is_child_owner";

#[test]
fn runtime_15_graphics_gpu_model_guard_is_child_owner() {
    let parent =
        read_runtime_src("tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics.rs");
    let child = read_runtime_src(
        "tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics/gpu_model_embedded_primitive.rs",
    );

    assert_contains_all(
        "Runtime 15 graphics naming parent mounts GPU-model child owner",
        &parent,
        &[
            "#[path = \"graphics/gpu_model_embedded_primitive.rs\"]",
            "mod gpu_model_embedded_primitive;",
            "#[path = \"graphics/offscreen_target_construct.rs\"]",
            "mod offscreen_target_construct;",
            "#[path = \"graphics/resource_streamer_construction.rs\"]",
            "mod resource_streamer_construction;",
            "#[path = \"graphics/render_framework_receiver.rs\"]",
            "mod render_framework_receiver;",
            "#[path = \"graphics/render_fixtures.rs\"]",
            "mod render_fixtures;",
            "fn rust_files(",
        ],
    );
    for moved_test in [
        "fn runtime_15_gpu_model_embedded_primitive_uses_current_names",
        "fn runtime_15_offscreen_target_construct_uses_owner_name",
        "fn runtime_15_resource_streamer_construction_uses_owner_name",
    ] {
        assert!(
            !parent.contains(moved_test),
            "runtime_15_m2/graphics.rs should mount graphics child owners instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "Runtime 15 graphics GPU-model child owns guard",
        &child,
        &[
            "use super::*;",
            "fn runtime_15_gpu_model_embedded_primitive_uses_current_names",
            "graphics/scene/resources/gpu_model/gpu_model_resource_from_asset.rs",
            "embedded primitive",
            "runtime_15_gpu_model_embedded_primitive_naming_hard_cutover_static_passed_cargo_deferred",
        ],
    );

    for (path, source) in [
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics.rs",
            parent.as_str(),
        ),
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics/gpu_model_embedded_primitive.rs",
            child.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
}
