use std::fs;
use std::path::Path;

use super::super::{assert_contains_all, read_repo_text, read_text};

const SLICE: &str = "Runtime 15 M2 render framework trait/construction owner naming hard cutover";
const STATUS: &str =
    "runtime_15_render_framework_trait_construction_owner_naming_hard_cutover_static_passed_cargo_deferred";
const GUARD: &str = "runtime_15_no_banned_name_modules";

const GRAPHICS_CONSTRUCTION_NEW_SWEEP_SLICE: &str =
    "Runtime 15 M2 graphics construction new owner naming hard cutover";
const GRAPHICS_CONSTRUCTION_NEW_SWEEP_STATUS: &str =
    "runtime_15_graphics_construction_new_owner_naming_hard_cutover_static_passed_cargo_deferred";
const GRAPHICS_CONSTRUCTION_NEW_SWEEP_GUARD: &str =
    "runtime_15_graphics_construction_new_owners_use_construct_names";

const SCENE_DYNAMIC_DOCUMENT_V1_SLICE: &str =
    "Runtime 15 M2 scene dynamic document v1 owner naming hard cutover";
const SCENE_DYNAMIC_DOCUMENT_V1_STATUS: &str =
    "runtime_15_scene_dynamic_document_v1_owner_naming_hard_cutover_static_passed_cargo_deferred";
const SCENE_DYNAMIC_DOCUMENT_V1_GUARD: &str =
    "runtime_15_scene_dynamic_document_v1_owner_uses_versioned_name";

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
fn runtime_15_no_banned_name_modules() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_root = manifest_root.join("src");
    let render_framework_dir = source_root.join("graphics/runtime/render_framework");
    let retired_trait_dir = render_framework_dir.join("render_framework_impl");
    let retired_construction_dir = render_framework_dir.join("wgpu_render_framework_new");
    let trait_binding_dir = render_framework_dir.join("render_framework_trait_binding");
    let construction_dir = render_framework_dir.join("wgpu_render_framework_construction");
    let render_framework_mod = read_text(
        &render_framework_dir.join("mod.rs"),
        "render framework module entry should be readable",
    );
    let trait_binding_mod = read_text(
        &trait_binding_dir.join("mod.rs"),
        "render framework trait binding module entry should be readable",
    );
    let wgpu_trait_binding = read_text(
        &trait_binding_dir.join("wgpu_framework.rs"),
        "WGPU render framework trait-binding owner should be readable",
    );
    let construction_mod = read_text(
        &construction_dir.join("mod.rs"),
        "WGPU render framework construction module entry should be readable",
    );
    let construction_owner = read_text(
        &construction_dir.join("construct.rs"),
        "WGPU render framework construction owner should be readable",
    );
    let banned_modules = banned_module_components(&source_root);

    assert!(
        banned_modules.is_empty(),
        "runtime source should not keep banned ownerless or migration-scented module names:\n{}",
        banned_modules.join("\n")
    );
    assert!(
        !retired_trait_dir.exists(),
        "render framework should not keep retired _impl owner directory {:?}",
        retired_trait_dir
    );
    assert!(
        !retired_construction_dir.exists(),
        "render framework should not keep retired *_new construction directory {:?}",
        retired_construction_dir
    );
    assert_contains_all(
        "render framework root module",
        &render_framework_mod,
        &[
            "mod render_framework_trait_binding;",
            "mod wgpu_render_framework_construction;",
        ],
    );
    assert!(
        !render_framework_mod.contains("mod render_framework_impl;"),
        "render framework root should not preserve the retired render_framework_impl module"
    );
    assert!(
        !render_framework_mod.contains("mod wgpu_render_framework_new;"),
        "render framework root should not preserve the retired wgpu_render_framework_new module"
    );
    assert_contains_all(
        "render framework trait binding module",
        &trait_binding_mod,
        &["mod wgpu_framework;"],
    );
    assert!(
        !trait_binding_mod.contains("mod trait_impl;"),
        "trait binding module should not preserve the retired trait_impl child"
    );
    assert_contains_all(
        "WGPU render framework trait binding owner",
        &wgpu_trait_binding,
        &[
            "impl RenderFramework for WgpuRenderFramework",
            "fn create_viewport(",
            "fn present_frame_extract_with_ui(",
            "fn set_quality_profile(",
        ],
    );
    assert_contains_all(
        "WGPU render framework construction module",
        &construction_mod,
        &["mod construct;", "mod create_default_pipelines;"],
    );
    assert!(
        !construction_mod.contains("mod new;"),
        "WGPU construction module should not preserve the retired new child"
    );
    assert_contains_all(
        "WGPU render framework construction owner",
        &construction_owner,
        &[
            "impl WgpuRenderFramework",
            "pub fn new(",
            "pub fn new_with_plugin_render_extensions(",
            "TaskPool::new(TaskPoolDescriptor::compute())",
        ],
    );

    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
    );
    let runtime_index = read_repo_text(manifest_root, "docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-review-findings-2026-06.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-structure-convention.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let graphics_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/graphics/render-product-submit.md",
    );
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
        ),
        "Runtime 15 status rows should be readable",
    );
    let status_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 expected status slice should be readable",
    );
    let date_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 expected date slice should be readable",
    );

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
                SLICE,
                STATUS,
                "graphics/runtime/render_framework/render_framework_trait_binding/wgpu_framework.rs",
                "graphics/runtime/render_framework/wgpu_render_framework_construction/construct.rs",
                GUARD,
            ],
        );
    }
}

#[test]
fn runtime_15_scene_dynamic_document_v1_owner_uses_versioned_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_root = manifest_root.join("src");
    let document_dir = source_root.join("scene/dynamic_scene/document");
    let retired_document = document_dir.join("legacy.rs");
    let v1_document = document_dir.join("v1_project_document.rs");

    assert!(
        !retired_document.exists(),
        "dynamic scene document owner should not keep retired legacy-named path {:?}",
        retired_document
    );
    assert!(
        v1_document.exists(),
        "dynamic scene document owner should use explicit v1 schema path {:?}",
        v1_document
    );

    let document_mod = read_text(
        &document_dir.join("mod.rs"),
        "dynamic scene document module entry should be readable",
    );
    let document_read = read_text(
        &document_dir.join("read.rs"),
        "dynamic scene document reader should be readable",
    );
    let document_owner = read_text(
        &v1_document,
        "dynamic scene v1 project document owner should be readable",
    );
    let audit_script = read_repo_text(
        manifest_root,
        ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/scene_project_serialization_boundary.py",
    );

    assert_contains_all(
        "dynamic scene document module",
        &document_mod,
        &["mod read;", "mod v1_project_document;", "mod write;"],
    );
    assert!(
        !document_mod.contains("mod legacy;"),
        "dynamic scene document module should not preserve retired legacy child"
    );
    assert_contains_all(
        "dynamic scene document reader",
        &document_read,
        &[
            "use super::v1_project_document::V1ProjectDocument;",
            "let document: V1ProjectDocument",
            "Self::from_world(&document.world)",
        ],
    );
    assert!(
        !document_read.contains("LegacyProjectDocument")
            && !document_read.contains("super::legacy"),
        "dynamic scene document reader should not preserve legacy owner references"
    );
    assert_contains_all(
        "dynamic scene v1 project document owner",
        &document_owner,
        &[
            "pub(super) struct V1ProjectDocument",
            "pub(super) world: World",
        ],
    );
    assert!(
        !document_owner.contains("LegacyProjectDocument"),
        "dynamic scene v1 project document owner should not preserve legacy type name"
    );
    assert_contains_all(
        "scene project serialization audit",
        &audit_script,
        &["zircon_runtime/src/scene/dynamic_scene/document/v1_project_document.rs"],
    );
    assert!(
        !audit_script.contains("zircon_runtime/src/scene/dynamic_scene/document/legacy.rs"),
        "scene project serialization audit should not keep retired dynamic scene document path"
    );

    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
    );
    let runtime_index = read_repo_text(manifest_root, "docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-review-findings-2026-06.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-structure-convention.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let dynamic_scene_doc =
        read_repo_text(manifest_root, "docs/zircon_runtime/scene/dynamic_scene.md");
    let session_note = read_repo_text(
        manifest_root,
        ".codex/sessions/20260612-0847-runtime-architecture-implementation.md",
    );
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
        ),
        "Runtime 15 status rows should be readable",
    );
    let status_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 expected status slice should be readable",
    );
    let date_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 expected date slice should be readable",
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan),
        ("runtime index", runtime_index),
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("module convention doc", module_doc),
        ("dynamic scene doc", dynamic_scene_doc),
        ("session note", session_note),
        ("status row data", status_rows),
        ("status slice", status_slice),
        ("date slice", date_slice),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                SCENE_DYNAMIC_DOCUMENT_V1_SLICE,
                SCENE_DYNAMIC_DOCUMENT_V1_STATUS,
                "scene/dynamic_scene/document/v1_project_document.rs",
                "V1ProjectDocument",
                SCENE_DYNAMIC_DOCUMENT_V1_GUARD,
            ],
        );
    }
}

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
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
    );
    let runtime_index = read_repo_text(manifest_root, "docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-review-findings-2026-06.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-structure-convention.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let graphics_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/graphics/render-product-submit.md",
    );
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
        ),
        "Runtime 15 status rows should be readable",
    );
    let status_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 expected status slice should be readable",
    );
    let date_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 expected date slice should be readable",
    );

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

fn banned_module_components(root: &Path) -> Vec<String> {
    let mut banned = Vec::new();
    collect_banned_module_components(root, root, &mut banned);
    banned.sort();
    banned
}

fn collect_banned_module_components(root: &Path, current: &Path, banned: &mut Vec<String>) {
    for entry in fs::read_dir(current).expect("runtime source directory should be readable") {
        let entry = entry.expect("runtime source entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .expect("runtime source directory should have a valid name");
            if is_banned_module_component(name) {
                banned.push(relative_display(root, &path));
            }
            collect_banned_module_components(root, &path, banned);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            let name = path
                .file_stem()
                .and_then(|name| name.to_str())
                .expect("runtime Rust file should have a valid stem");
            if is_banned_module_component(name) {
                banned.push(relative_display(root, &path));
            }
        }
    }
}

fn is_banned_module_component(name: &str) -> bool {
    matches!(
        name,
        "common" | "helper" | "helpers" | "misc" | "new" | "util" | "utils"
    ) || name.ends_with("_helper")
        || name.ends_with("_helpers")
        || name.ends_with("_impl")
        || name.ends_with("_inner")
        || name.ends_with("_new")
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

fn relative_display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}
