use super::*;

const STATUS: &str = "runtime_15_graphics_render_framework_receiver_guard_child_owner_split_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 graphics render-framework receiver guard child-owner split";
const GUARD: &str = "runtime_15_graphics_render_framework_receiver_guard_is_child_owner";

#[test]
fn runtime_15_graphics_render_framework_receiver_guard_is_child_owner() {
    let parent =
        read_runtime_src("tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics.rs");
    let child = read_runtime_src(
        "tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics/render_framework_receiver.rs",
    );

    assert_contains_all(
        "Runtime 15 graphics naming parent mounts render-framework receiver child owner",
        &parent,
        &[
            "#[path = \"graphics/render_framework_receiver.rs\"]",
            "mod render_framework_receiver;",
            "#[path = \"graphics/render_fixtures.rs\"]",
            "mod render_fixtures;",
            "#[path = \"graphics/resource_streamer_construction.rs\"]",
            "mod resource_streamer_construction;",
            "#[path = \"graphics/offscreen_target_construct.rs\"]",
            "mod offscreen_target_construct;",
            "#[path = \"graphics/gpu_model_embedded_primitive.rs\"]",
            "mod gpu_model_embedded_primitive;",
            "fn rust_files(",
        ],
    );
    assert!(
        !parent.contains("fn runtime_15_render_framework_receiver_uses_framework_name"),
        "runtime_15_m2/graphics.rs should mount render_framework_receiver child instead of defining the naming guard"
    );
    assert!(
        !parent.contains("fn runtime_15_gpu_model_embedded_primitive_uses_current_names"),
        "runtime_15_m2/graphics.rs should mount gpu_model_embedded_primitive child instead of defining the naming guard"
    );

    assert_contains_all(
        "Runtime 15 graphics render-framework receiver child owns guard",
        &child,
        &[
            "use super::*;",
            "fn runtime_15_render_framework_receiver_uses_framework_name",
            "graphics/runtime/render_framework",
            "graphics-render-framework-debt",
            "runtime_15_graphics_render_framework_receiver_naming_hard_cutover_static_passed_cargo_deferred",
        ],
    );

    for (path, source) in [
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics.rs",
            parent.as_str(),
        ),
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics/render_framework_receiver.rs",
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
