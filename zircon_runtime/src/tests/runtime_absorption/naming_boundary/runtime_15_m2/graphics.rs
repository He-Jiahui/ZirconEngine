use std::fs;
use std::path::{Path, PathBuf};

use super::super::{assert_contains_all, read_repo_text, read_text};

#[test]
fn runtime_15_render_framework_receiver_uses_framework_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let render_framework_dir = manifest_root.join("src/graphics/runtime/render_framework");
    let render_framework_files = rust_files(&render_framework_dir);
    let representative_files = [
        "src/graphics/runtime/render_framework/capture_frame/capture_frame.rs",
        "src/graphics/runtime/render_framework/create_viewport/create.rs",
        "src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs",
        "src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs",
        "src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs",
    ];
    let naming_boundary = read_text(
        &manifest_root.join("src/tests/runtime_absorption/naming_boundary.rs"),
        "runtime naming boundary guard should be readable",
    );
    let audit_script = read_repo_text(
        manifest_root,
        ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/non_network_server_naming.py",
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
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
        ),
        "Runtime 15 status rows should be readable",
    );
    let status_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
        ),
        "Runtime 15 expected status slice should be readable",
    );
    let date_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
        ),
        "Runtime 15 expected date slice should be readable",
    );

    assert!(
        !render_framework_files.is_empty(),
        "render framework source owner should contain Rust files"
    );
    for file in &render_framework_files {
        let source = read_text(file, "render framework source should be readable");
        assert!(
            !source.to_ascii_lowercase().contains("server"),
            "{} should not use non-network server naming for render-framework receiver/context variables",
            file.strip_prefix(manifest_root)
                .expect("render framework file should be under manifest root")
                .display()
        );
    }
    for relative_path in representative_files {
        let source = read_text(
            &manifest_root.join(relative_path),
            "representative render framework source should be readable",
        );
        assert_contains_all(
            relative_path,
            &source,
            &["framework: &WgpuRenderFramework", "framework.lock_"],
        );
    }
    assert!(
        !naming_boundary.contains("graphics-render-framework-debt"),
        "runtime naming boundary should not require the retired graphics render-framework server debt bucket"
    );
    assert!(
        !audit_script.contains("graphics-render-framework-debt"),
        "runtime structure audit should not retain the retired graphics render-framework server debt bucket"
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
                "Runtime 15 M2 graphics render-framework receiver naming hard cutover",
                "runtime_15_graphics_render_framework_receiver_naming_hard_cutover_static_passed_cargo_deferred",
                "graphics/runtime/render_framework",
                "runtime_15_render_framework_receiver_uses_framework_name",
            ],
        );
    }
}

#[test]
fn runtime_15_resource_streamer_construction_uses_owner_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let resource_streamer_dir =
        manifest_root.join("src/graphics/scene/resources/resource_streamer");
    let retired_resource_streamer_new = resource_streamer_dir.join("resource_streamer_new.rs");
    let resource_streamer_mod = read_text(
        &resource_streamer_dir.join("mod.rs"),
        "resource streamer module entry should be readable",
    );
    let resource_streamer_construction = read_text(
        &resource_streamer_dir.join("resource_streamer_construction.rs"),
        "resource streamer construction owner should be readable",
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
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
        ),
        "Runtime 15 status rows should be readable",
    );
    let status_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
        ),
        "Runtime 15 expected status slice should be readable",
    );
    let date_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
        ),
        "Runtime 15 expected date slice should be readable",
    );

    assert!(
        !retired_resource_streamer_new.exists(),
        "resource streamer should not keep *_new construction owner file {:?}",
        retired_resource_streamer_new
    );
    assert_contains_all(
        "resource streamer module entry",
        &resource_streamer_mod,
        &["mod resource_streamer_construction;"],
    );
    assert!(
        !resource_streamer_mod.contains("mod resource_streamer_new;"),
        "resource_streamer/mod.rs should not preserve the retired resource_streamer_new module name"
    );
    assert_contains_all(
        "resource streamer construction owner",
        &resource_streamer_construction,
        &[
            "impl ResourceStreamer",
            "pub(crate) fn new(",
            "fallback_texture: Arc::new",
            "OutputTargetWritebackConverter::new(device)",
        ],
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
                "Runtime 15 M2 resource streamer construction module naming hard cutover",
                "runtime_15_resource_streamer_construction_naming_hard_cutover_static_passed_cargo_deferred",
                "graphics/scene/resources/resource_streamer/resource_streamer_construction.rs",
                "runtime_15_resource_streamer_construction_uses_owner_name",
            ],
        );
    }
}

#[test]
fn runtime_15_offscreen_target_construct_uses_owner_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let render_backend_dir = manifest_root.join("src/graphics/backend/render_backend");
    let retired_offscreen_target_new = render_backend_dir.join("offscreen_target_new");
    let offscreen_target_construct_dir = render_backend_dir.join("offscreen_target_construct");
    let render_backend_mod = read_text(
        &render_backend_dir.join("mod.rs"),
        "render backend module entry should be readable",
    );
    let construct_mod = read_text(
        &offscreen_target_construct_dir.join("mod.rs"),
        "offscreen target construct module entry should be readable",
    );
    let construct_owner = read_text(
        &offscreen_target_construct_dir.join("construct.rs"),
        "offscreen target construct owner should be readable",
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
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
        ),
        "Runtime 15 status rows should be readable",
    );
    let status_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
        ),
        "Runtime 15 expected status slice should be readable",
    );
    let date_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
        ),
        "Runtime 15 expected date slice should be readable",
    );

    assert!(
        !retired_offscreen_target_new.exists(),
        "render backend should not keep *_new construction owner directory {:?}",
        retired_offscreen_target_new
    );
    assert!(
        offscreen_target_construct_dir.is_dir(),
        "render backend should keep offscreen target construction in a construct-named directory"
    );
    assert_contains_all(
        "render backend module entry",
        &render_backend_mod,
        &["mod offscreen_target_construct;"],
    );
    assert!(
        !render_backend_mod.contains("mod offscreen_target_new;"),
        "render_backend/mod.rs should not preserve the retired offscreen_target_new module name"
    );
    assert_contains_all(
        "offscreen target construct module entry",
        &construct_mod,
        &[
            "mod construct;",
            "mod create_cluster_buffer;",
            "mod create_texture_bundle;",
            "mod texture_bundle;",
        ],
    );
    assert_contains_all(
        "offscreen target construct owner",
        &construct_owner,
        &[
            "impl OffscreenTarget",
            "pub(crate) fn new(",
            "zircon-offscreen-final-color",
            "create_cluster_buffer(device, cluster_buffer_bytes)",
        ],
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
                "Runtime 15 M2 offscreen target construct directory naming hard cutover",
                "runtime_15_offscreen_target_construct_naming_hard_cutover_static_passed_cargo_timeout_no_result",
                "graphics/backend/render_backend/offscreen_target_construct/construct.rs",
                "runtime_15_offscreen_target_construct_uses_owner_name",
            ],
        );
    }
}

fn rust_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_files(root, &mut files);
    files.sort();
    files
}

fn collect_rust_files(root: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).expect("render framework directory should be readable") {
        let entry = entry.expect("render framework entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, files);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}
