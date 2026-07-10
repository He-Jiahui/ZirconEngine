use std::fs;

use super::*;

const GRAPHICS_CONSTRUCTION_NEW_SWEEP_SLICE: &str =
    "Runtime 15 M2 graphics construction new owner naming hard cutover";
const GRAPHICS_CONSTRUCTION_NEW_SWEEP_STATUS: &str =
    "runtime_15_graphics_construction_new_owner_naming_hard_cutover_static_passed_cargo_deferred";
const GRAPHICS_CONSTRUCTION_NEW_SWEEP_GUARD: &str =
    "runtime_15_graphics_construction_new_owners_use_construct_names";

const RETIRED_GRAPHICS_NEW_OWNERS: &[&str] = &[
    "graphics/feature/render_feature_descriptor/new.rs",
    "graphics/feature/render_feature_pass_descriptor/new.rs",
    "graphics/runtime/history/new.rs",
    "graphics/runtime/render_framework/viewport_record/new.rs",
    "graphics/scene/scene_renderer/core/scene_renderer_construct/new.rs",
    "graphics/scene/scene_renderer/deferred/deferred_scene_resources/new.rs",
    "graphics/scene/scene_renderer/history/scene_frame_history_textures/new.rs",
    "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/new.rs",
    "graphics/scene/scene_renderer/overlay/passes/scene_gizmo_pass/new.rs",
    "graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/construct/new.rs",
    "graphics/scene/scene_renderer/particle/particle_renderer/new.rs",
    "graphics/scene/scene_renderer/particle/particle_velocity_vertex/new.rs",
    "graphics/scene/scene_renderer/particle/particle_vertex/new.rs",
    "graphics/scene/scene_renderer/post_process/resources/new",
    "graphics/scene/scene_renderer/post_process/resources/new/construct/new.rs",
    "graphics/scene/scene_renderer/ui/new.rs",
];

const GRAPHICS_CONSTRUCT_OWNERS: &[&str] = &[
    "graphics/feature/render_feature_descriptor/construct.rs",
    "graphics/feature/render_feature_pass_descriptor/construct.rs",
    "graphics/runtime/history/construct.rs",
    "graphics/runtime/render_framework/viewport_record/construct.rs",
    "graphics/scene/scene_renderer/core/scene_renderer_construct/construct.rs",
    "graphics/scene/scene_renderer/deferred/deferred_scene_resources/construct.rs",
    "graphics/scene/scene_renderer/history/scene_frame_history_textures/construct.rs",
    "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/construct.rs",
    "graphics/scene/scene_renderer/overlay/passes/scene_gizmo_pass/construct.rs",
    "graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/construct/construct.rs",
    "graphics/scene/scene_renderer/particle/particle_renderer/construct.rs",
    "graphics/scene/scene_renderer/particle/particle_velocity_vertex/construct.rs",
    "graphics/scene/scene_renderer/particle/particle_vertex/construct.rs",
    "graphics/scene/scene_renderer/post_process/resources/construct",
    "graphics/scene/scene_renderer/post_process/resources/construct/construct/construct.rs",
    "graphics/scene/scene_renderer/ui/construct.rs",
];

#[test]
fn runtime_15_graphics_construction_new_owners_use_construct_names() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_root = manifest_root.join("src");
    let graphics_root = source_root.join("graphics");

    for retired in RETIRED_GRAPHICS_NEW_OWNERS {
        let retired_path = source_root.join(retired);
        assert!(
            !retired_path.exists(),
            "graphics construction owner should not keep retired `new` module path {:?}",
            retired_path
        );
    }
    for owner in GRAPHICS_CONSTRUCT_OWNERS {
        let owner_path = source_root.join(owner);
        assert!(
            owner_path.exists(),
            "graphics construction owner should live under construct-named path {:?}",
            owner_path
        );
    }

    let graphics_new_modules = new_module_components(&graphics_root);
    assert!(
        graphics_new_modules.is_empty(),
        "graphics source should not keep bare `new` owner modules after construction cutover:\n{}",
        graphics_new_modules.join("\n")
    );

    for parent in [
        "graphics/feature/render_feature_descriptor/mod.rs",
        "graphics/feature/render_feature_pass_descriptor/mod.rs",
        "graphics/runtime/history/mod.rs",
        "graphics/runtime/render_framework/viewport_record/mod.rs",
        "graphics/scene/scene_renderer/core/scene_renderer_construct/mod.rs",
        "graphics/scene/scene_renderer/deferred/deferred_scene_resources/mod.rs",
        "graphics/scene/scene_renderer/history/scene_frame_history_textures/mod.rs",
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mod.rs",
        "graphics/scene/scene_renderer/overlay/passes/scene_gizmo_pass/mod.rs",
        "graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/construct/mod.rs",
        "graphics/scene/scene_renderer/particle/particle_renderer/mod.rs",
        "graphics/scene/scene_renderer/particle/particle_velocity_vertex/mod.rs",
        "graphics/scene/scene_renderer/particle/particle_vertex/mod.rs",
        "graphics/scene/scene_renderer/post_process/resources/mod.rs",
        "graphics/scene/scene_renderer/post_process/resources/construct/construct/mod.rs",
        "graphics/scene/scene_renderer/ui/mod.rs",
    ] {
        let source = read_text(
            &source_root.join(parent),
            "graphics construction module entry should be readable",
        );
        assert_contains_all(parent, &source, &["mod construct;"]);
        assert!(
            !source.contains("mod new;"),
            "{parent} should not preserve retired `new` module entry"
        );
    }

    let post_process_buffer_bundle = read_text(
        &source_root.join(
            "graphics/scene/scene_renderer/post_process/resources/construct/buffer_bundle/buffer_bundle.rs",
        ),
        "post-process construct buffer bundle should be readable",
    );
    let post_process_pipeline_bundle = read_text(
        &source_root.join(
            "graphics/scene/scene_renderer/post_process/resources/construct/pipeline_bundle/pipeline_bundle.rs",
        ),
        "post-process construct pipeline bundle should be readable",
    );
    assert!(
        !format!("{post_process_buffer_bundle}\n{post_process_pipeline_bundle}")
            .contains("resources::new"),
        "post-process construct child owners should not keep resources::new visibility paths"
    );

    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    let runtime_index = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let graphics_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/graphics/render-product-submit.md",
    );
    let status_rows = read_runtime_15_naming_status_rows(manifest_root);
    let status_slice = read_runtime_15_naming_status_map(manifest_root);
    let date_slice = read_runtime_15_naming_date_map(manifest_root);

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan),
        ("runtime index", runtime_index),
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("module convention doc", module_doc),
        ("graphics render-product doc", graphics_doc),
        ("status row data", status_rows),
        ("status slice", status_slice),
        ("date slice", date_slice),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                GRAPHICS_CONSTRUCTION_NEW_SWEEP_SLICE,
                GRAPHICS_CONSTRUCTION_NEW_SWEEP_STATUS,
                "graphics/feature/render_feature_descriptor/construct.rs",
                "graphics/scene/scene_renderer/post_process/resources/construct/construct/construct.rs",
                GRAPHICS_CONSTRUCTION_NEW_SWEEP_GUARD,
                GUARD,
            ],
        );
    }
}

fn new_module_components(root: &Path) -> Vec<String> {
    let mut modules = Vec::new();
    collect_new_module_components(root, root, &mut modules);
    modules.sort();
    modules
}

fn collect_new_module_components(root: &Path, current: &Path, modules: &mut Vec<String>) {
    for entry in fs::read_dir(current).expect("graphics source directory should be readable") {
        let entry = entry.expect("graphics source entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .expect("graphics source directory should have a valid name");
            if name == "new" {
                modules.push(relative_display(root, &path));
            }
            collect_new_module_components(root, &path, modules);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            let name = path
                .file_stem()
                .and_then(|name| name.to_str())
                .expect("graphics Rust file should have a valid stem");
            if name == "new" {
                modules.push(relative_display(root, &path));
            }
        }
    }
}
