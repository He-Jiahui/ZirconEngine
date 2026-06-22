use super::super::{assert_contains_all, runtime_src_path};

#[test]
fn runtime_15_graphics_dead_code_guard_is_folder_backed() {
    let old_flat_guard =
        runtime_src_path("tests/runtime_absorption/structure_convention/graphics_dead_code.rs");
    let parent_guard =
        runtime_src_path("tests/runtime_absorption/structure_convention/graphics_dead_code/mod.rs");
    let layout_guard = runtime_src_path(
        "tests/runtime_absorption/structure_convention/graphics_dead_code/module_layout.rs",
    );
    let renderer_output_guards = runtime_src_path(
        "tests/runtime_absorption/structure_convention/graphics_dead_code/renderer_output_accessors.rs",
    );
    assert!(
        !old_flat_guard.exists(),
        "graphics dead-code guards should stay folder-backed, not return to {:?}",
        old_flat_guard
    );
    assert!(
        parent_guard.is_file(),
        "graphics dead-code parent guard module should exist at {:?}",
        parent_guard
    );
    assert!(
        layout_guard.is_file(),
        "graphics dead-code layout guard should live in its own module at {:?}",
        layout_guard
    );
    assert!(
        renderer_output_guards.is_file(),
        "renderer output accessor guards should live in their own module at {:?}",
        renderer_output_guards
    );

    let structure_convention = std::fs::read_to_string(runtime_src_path(
        "tests/runtime_absorption/structure_convention.rs",
    ))
    .expect("structure convention test owner should be readable");
    let parent_source = std::fs::read_to_string(&parent_guard)
        .expect("graphics dead-code parent should be readable");
    let renderer_output_source = std::fs::read_to_string(&renderer_output_guards)
        .expect("renderer output accessor guard module should be readable");

    assert_contains_all(
        "graphics dead-code folder-backed module mount",
        &structure_convention,
        &[
            "#[path = \"structure_convention/graphics_dead_code/mod.rs\"]",
            "mod graphics_dead_code;",
        ],
    );
    assert_contains_all(
        "graphics dead-code parent module split",
        &parent_source,
        &[
            "mod module_layout;",
            "mod renderer_output_accessors;",
            "fn runtime_15_offscreen_target_texture_owner_cleanup()",
        ],
    );
    assert!(
        !parent_source.contains("fn runtime_15_particle_gpu_readback_output_accessor_cleanup()"),
        "particle output accessor guard should stay in renderer_output_accessors.rs"
    );
    assert!(
        !parent_source.contains("fn runtime_15_advanced_plugin_output_test_accessor_cleanup()"),
        "advanced plugin output accessor guard should stay in renderer_output_accessors.rs"
    );
    assert_contains_all(
        "renderer output accessor child module guards",
        &renderer_output_source,
        &[
            "fn runtime_15_particle_gpu_readback_output_accessor_cleanup()",
            "fn runtime_15_advanced_plugin_output_test_accessor_cleanup()",
        ],
    );
    let parent_line_count = parent_source.lines().count();
    assert!(
        parent_line_count < 850,
        "graphics_dead_code/mod.rs should stay below the near-large-file planning threshold after split; found {parent_line_count}"
    );

    let runtime_15_plan = super::read_repo(
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
    );
    let runtime_index = super::read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = super::read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention_doc =
        super::read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = super::read_repo("docs/zircon_runtime/structure/module-convention.md");

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention_doc.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 graphics dead-code guard module split",
                "runtime_15_graphics_dead_code_guard_module_split_static_passed_cargo_lock_blocked",
                "runtime_15_graphics_dead_code_guard_is_folder_backed",
                "graphics_dead_code/module_layout.rs",
                "graphics_dead_code/renderer_output_accessors.rs",
            ],
        );
    }
}
