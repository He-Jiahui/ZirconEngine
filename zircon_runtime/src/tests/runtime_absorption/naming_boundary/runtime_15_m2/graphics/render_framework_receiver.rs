use super::*;

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
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
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
