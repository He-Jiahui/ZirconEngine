use std::fs;

use super::*;

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
