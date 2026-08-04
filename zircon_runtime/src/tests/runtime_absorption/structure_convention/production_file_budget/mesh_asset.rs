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
}
