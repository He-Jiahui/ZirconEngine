use super::*;

const STATUS: &str =
    "runtime_15_banned_names_global_module_guard_child_owner_split_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 banned-name global module guard child-owner split";
const GUARD: &str = "runtime_15_banned_names_global_module_guard_is_child_owner";

#[test]
fn runtime_15_banned_names_global_module_guard_is_child_owner() {
    let parent =
        read_runtime_src("tests/runtime_absorption/naming_boundary/runtime_15_m2/banned_names.rs");
    let child = read_runtime_src(
        "tests/runtime_absorption/naming_boundary/runtime_15_m2/banned_names/global_modules.rs",
    );

    assert_contains_all(
        "Runtime 15 banned-names parent mounts global banned-name child owner",
        &parent,
        &[
            "#[path = \"banned_names/global_modules.rs\"]",
            "mod global_modules;",
            "#[path = \"banned_names/scene_dynamic.rs\"]",
            "mod scene_dynamic;",
            "#[path = \"banned_names/graphics_construction.rs\"]",
            "mod graphics_construction;",
            "fn relative_display(",
        ],
    );
    assert!(
        !parent.contains("fn runtime_15_no_banned_name_modules"),
        "runtime_15_m2/banned_names.rs should mount global_modules child instead of defining the global banned-name guard"
    );
    assert!(
        !parent.contains("fn banned_module_components(")
            && !parent.contains("fn collect_banned_module_components(")
            && !parent.contains("fn is_banned_module_component("),
        "runtime_15_m2/banned_names.rs should move global banned-name scan helpers into the global_modules child owner"
    );

    assert_contains_all(
        "Runtime 15 banned-names global child owns global module naming guard",
        &child,
        &[
            "use std::fs;",
            "use super::*;",
            "fn runtime_15_no_banned_name_modules",
            "fn banned_module_components(",
            "fn collect_banned_module_components(",
            "fn is_banned_module_component(",
            "SLICE",
            "STATUS",
            "GUARD",
            "graphics/runtime/render_framework/render_framework_trait_binding/wgpu_framework.rs",
            "graphics/runtime/render_framework/wgpu_render_framework_construction/construct.rs",
        ],
    );

    for (path, source) in [
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/banned_names.rs",
            parent.as_str(),
        ),
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/banned_names/global_modules.rs",
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
