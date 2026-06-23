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
    let backend_owner_guards = runtime_src_path(
        "tests/runtime_absorption/structure_convention/graphics_dead_code/backend_owners.rs",
    );
    let gpu_resource_owner_guards = runtime_src_path(
        "tests/runtime_absorption/structure_convention/graphics_dead_code/gpu_resource_owners.rs",
    );
    let resource_streamer_guards = runtime_src_path(
        "tests/runtime_absorption/structure_convention/graphics_dead_code/resource_streamer_cleanup.rs",
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
        backend_owner_guards.is_file(),
        "graphics backend owner guards should live in their own module at {:?}",
        backend_owner_guards
    );
    assert!(
        gpu_resource_owner_guards.is_file(),
        "GPU resource owner guards should live in their own module at {:?}",
        gpu_resource_owner_guards
    );
    assert!(
        resource_streamer_guards.is_file(),
        "resource-streamer cleanup guards should live in their own module at {:?}",
        resource_streamer_guards
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
    let backend_owner_source = std::fs::read_to_string(&backend_owner_guards)
        .expect("backend owner guard module should be readable");
    let gpu_resource_owner_source = std::fs::read_to_string(&gpu_resource_owner_guards)
        .expect("GPU resource owner guard module should be readable");
    let resource_streamer_source = std::fs::read_to_string(&resource_streamer_guards)
        .expect("resource-streamer cleanup guard module should be readable");
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
            "mod backend_owners;",
            "mod gpu_resource_owners;",
            "mod module_layout;",
            "mod renderer_output_accessors;",
            "mod resource_streamer_cleanup;",
            "fn read_runtime_src(relative: &str) -> String",
            "fn read_repo(relative: &str) -> String",
        ],
    );
    for moved_guard in [
        "fn runtime_15_offscreen_target_texture_owner_cleanup()",
        "fn runtime_15_render_backend_state_owner_cleanup()",
        "fn runtime_15_gpu_texture_resource_owner_cleanup()",
        "fn runtime_15_gpu_material_uniform_owner_cleanup()",
        "fn runtime_15_gpu_mesh_order_signature_cleanup()",
        "fn runtime_15_gpu_model_identity_cleanup()",
        "fn runtime_15_post_process_lut_texture_owner_cleanup()",
        "fn runtime_15_output_target_texture_owner_cleanup()",
        "fn runtime_15_material_runtime_capture_seed_cleanup()",
        "fn runtime_15_resource_streamer_diagnostics_accessor_cleanup()",
        "fn runtime_15_resource_streamer_resolve_texture_id_cleanup()",
    ] {
        assert!(
            !parent_source.contains(moved_guard),
            "graphics_dead_code/mod.rs should mount child guard owners instead of defining {moved_guard}"
        );
    }
    assert_contains_all(
        "backend owner child module guards",
        &backend_owner_source,
        &[
            "fn runtime_15_offscreen_target_texture_owner_cleanup()",
            "fn runtime_15_render_backend_state_owner_cleanup()",
        ],
    );
    assert_contains_all(
        "GPU resource owner child module guards",
        &gpu_resource_owner_source,
        &[
            "fn runtime_15_gpu_texture_resource_owner_cleanup()",
            "fn runtime_15_gpu_material_uniform_owner_cleanup()",
            "fn runtime_15_gpu_mesh_order_signature_cleanup()",
            "fn runtime_15_gpu_model_identity_cleanup()",
            "fn runtime_15_post_process_lut_texture_owner_cleanup()",
            "fn runtime_15_output_target_texture_owner_cleanup()",
        ],
    );
    assert_contains_all(
        "resource-streamer cleanup child module guards",
        &resource_streamer_source,
        &[
            "fn runtime_15_material_runtime_capture_seed_cleanup()",
            "fn runtime_15_resource_streamer_diagnostics_accessor_cleanup()",
            "fn runtime_15_resource_streamer_resolve_texture_id_cleanup()",
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
    for (path, source) in [
        ("graphics_dead_code/mod.rs", parent_source.as_str()),
        (
            "graphics_dead_code/backend_owners.rs",
            backend_owner_source.as_str(),
        ),
        (
            "graphics_dead_code/gpu_resource_owners.rs",
            gpu_resource_owner_source.as_str(),
        ),
        (
            "graphics_dead_code/resource_streamer_cleanup.rs",
            resource_streamer_source.as_str(),
        ),
        (
            "graphics_dead_code/renderer_output_accessors.rs",
            renderer_output_source.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 guard-owner budget; found {line_count}"
        );
    }

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
                "Runtime 15 M3 graphics dead-code guard child-owner split",
                "runtime_15_graphics_dead_code_guard_child_owner_split_static_passed_cargo_deferred",
                "graphics_dead_code/backend_owners.rs",
                "graphics_dead_code/gpu_resource_owners.rs",
                "graphics_dead_code/resource_streamer_cleanup.rs",
            ],
        );
    }
}
