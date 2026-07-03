use super::*;

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
