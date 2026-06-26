use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_mesh_asset_management_records_are_child_owner() {
    let parent = read_runtime_src("asset/assets/mesh/mesh_asset.rs");
    let management = read_runtime_src("asset/assets/mesh/mesh_asset/management.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let render_asset_doc = read_repo("docs/zircon_runtime/asset/render-assets.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
    );
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
    );

    assert_contains_all(
        "mesh asset parent keeps mesh DTO and re-exports management records",
        &parent,
        &[
            "mod management;",
            "pub use self::management::{",
            "MeshAssetManagementRecord, MeshAssetManagementRecordFailure, MeshAssetManagementRecordSet,",
            "MeshAssetManagementRecordSetSummary, MeshAssetOverview,",
            "pub struct MeshAsset",
            "pub fn overview(&self) -> Result<MeshAssetOverview, MeshValidationError>",
            "pub fn management_record(",
            "Ok(MeshAssetManagementRecord {",
        ],
    );
    for moved_owner in [
        "pub struct MeshAssetOverview",
        "pub struct MeshAssetManagementRecordSetSummary",
        "pub struct MeshAssetManagementRecordSet {",
        "impl MeshAssetManagementRecordSetSummary",
        "impl MeshAssetManagementRecordSet",
        "pub fn from_records_and_failures(",
        "pub fn from_results(",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "asset/assets/mesh/mesh_asset.rs should delegate {moved_owner} to mesh_asset/management.rs"
        );
    }
    assert_contains_all(
        "mesh management child owns overview and record-set aggregation DTOs",
        &management,
        &[
            "pub struct MeshAssetOverview",
            "pub struct MeshAssetManagementRecord",
            "pub struct MeshAssetManagementRecordFailure",
            "pub struct MeshAssetManagementRecordSetSummary",
            "pub struct MeshAssetManagementRecordSet",
            "pub fn from_records_and_failures(",
            "pub fn from_results(",
            "virtual_geometry_mesh_count",
        ],
    );

    for (path, source) in [
        ("asset/assets/mesh/mesh_asset.rs", parent.as_str()),
        (
            "asset/assets/mesh/mesh_asset/management.rs",
            management.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("render asset doc", render_asset_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M4 mesh asset management record owner split",
                "runtime_15_mesh_asset_management_record_owner_split_static_passed_cargo_deferred",
                "asset/assets/mesh/mesh_asset.rs",
                "asset/assets/mesh/mesh_asset/management.rs",
                "runtime_15_mesh_asset_management_records_are_child_owner",
            ],
        );
    }
    assert_contains_all(
        "status-output status map",
        &status_map,
        &[
            "Runtime 15 M4 mesh asset management record owner split",
            "runtime_15_mesh_asset_management_record_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "status-output date map",
        &date_map,
        &[
            "Runtime 15 M4 mesh asset management record owner split",
            "2026-06-24",
        ],
    );
}
