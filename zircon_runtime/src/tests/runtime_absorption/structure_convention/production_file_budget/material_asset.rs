use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_material_asset_value_readiness_helpers_are_child_owners() {
    let parent = read_runtime_src("asset/assets/material/material_asset.rs");
    let value_sync = read_runtime_src("asset/assets/material/material_asset/value_sync.rs");
    let readiness = read_runtime_src("asset/assets/material/material_asset/readiness.rs");
    let management = read_runtime_src("asset/assets/material/material_asset/management.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let material_doc = read_repo("docs/zircon_runtime/asset/zmeta-shader-material.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
    );

    assert_contains_all(
        "material asset parent keeps public DTO and descriptor/readiness entry ownership",
        &parent,
        &[
            "mod readiness;",
            "mod value_sync;",
            "mod management;",
            "pub use self::management::{",
            "use self::readiness::{material_readiness_diagnostics, push_shader_readiness_validation_errors};",
            "use self::value_sync::{",
            "pub struct MaterialAsset",
            "pub fn from_zmaterial_document",
            "pub fn to_zmaterial_document",
            "pub fn readiness_report_with_shader_contract",
            "pub fn standard_material_descriptor",
            "fn readiness_report_from_texture_slots",
        ],
    );
    for moved_owner in [
        "fn texture_slot_reference",
        "fn toml_number_as_f32",
        "fn sync_texture_slot",
        "fn push_shader_readiness_validation_errors",
        "fn material_readiness_diagnostics",
        "ShaderRuntimeSourceKind::Unavailable",
        "RenderMaterialDiagnosticSource::ShaderReadiness",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "asset/assets/material/material_asset.rs should delegate {moved_owner} to material_asset child owners"
        );
    }
    assert_contains_all(
        "value-sync child owns TOML override and texture-slot hydration helpers",
        &value_sync,
        &[
            "pub(super) fn texture_slot_reference",
            "pub(super) fn override_f32",
            "pub(super) fn override_vec4",
            "pub(super) fn sync_texture_slot",
            "pub(super) fn sync_f32_override",
            "fn toml_array",
        ],
    );
    assert_contains_all(
        "readiness child owns shader and material readiness diagnostic projection",
        &readiness,
        &[
            "pub(super) fn push_shader_readiness_validation_errors",
            "ShaderRuntimeSourceKind::Unavailable",
            "RenderMaterialDiagnosticSource::ShaderReadiness",
            "RenderMaterialDiagnosticSource::WgslCapture",
            "pub(super) fn material_readiness_diagnostics",
            "RenderMaterialDiagnosticSource::MaterialAsset",
        ],
    );

    for (path, source) in [
        ("asset/assets/material/material_asset.rs", parent.as_str()),
        (
            "asset/assets/material/material_asset/value_sync.rs",
            value_sync.as_str(),
        ),
        (
            "asset/assets/material/material_asset/readiness.rs",
            readiness.as_str(),
        ),
        (
            "asset/assets/material/material_asset/management.rs",
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
        ("material asset doc", material_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M4 material asset value/readiness helper owner split",
                "runtime_15_material_asset_value_readiness_owner_split_static_passed_cargo_timeout_no_result",
                "asset/assets/material/material_asset.rs",
                "asset/assets/material/material_asset/value_sync.rs",
                "asset/assets/material/material_asset/readiness.rs",
                "runtime_15_material_asset_value_readiness_helpers_are_child_owners",
            ],
        );
    }
    assert_contains_all(
        "status-output row data",
        &status_rows,
        &[
            "Runtime 15 M4 material asset value/readiness helper owner split",
            "runtime_15_material_asset_value_readiness_owner_split_static_passed_cargo_timeout_no_result",
            "asset/assets/material/material_asset.rs",
            "asset/assets/material/material_asset/value_sync.rs",
            "runtime_15_material_asset_value_readiness_helpers_are_child_owners",
        ],
    );
}

#[test]
fn runtime_15_material_asset_management_records_are_child_owner() {
    let parent = read_runtime_src("asset/assets/material/material_asset.rs");
    let management = read_runtime_src("asset/assets/material/material_asset/management.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let material_doc = read_repo("docs/zircon_runtime/asset/zmeta-shader-material.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
    );

    assert_contains_all(
        "material asset parent keeps management entry points and public re-export",
        &parent,
        &[
            "mod management;",
            "pub use self::management::{",
            "MaterialAssetManagementRecordSetSummary, MaterialAssetOverview,",
            "pub fn overview(&self) -> MaterialAssetOverview",
            "pub fn management_record(&self, material_id: ResourceId) -> MaterialAssetManagementRecord",
        ],
    );
    for moved_owner in [
        "pub struct MaterialAssetOverview",
        "pub struct MaterialAssetManagementRecord",
        "pub struct MaterialAssetManagementRecordSetSummary",
        "pub struct MaterialAssetManagementRecordSet",
        "impl MaterialAssetManagementRecordSetSummary",
        "impl MaterialAssetManagementRecordSet",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "asset/assets/material/material_asset.rs should delegate {moved_owner} to material_asset/management.rs"
        );
    }
    assert_contains_all(
        "management child owns material asset overview and record aggregation DTOs",
        &management,
        &[
            "pub struct MaterialAssetOverview",
            "pub struct MaterialAssetManagementRecord",
            "pub struct MaterialAssetManagementRecordSetSummary",
            "pub struct MaterialAssetManagementRecordSet",
            "pub fn from_records(records: &[MaterialAssetManagementRecord]) -> Self",
            "pub fn degraded_count(&self) -> usize",
            "pub fn issue_row_count(&self) -> usize",
            "records.sort_by_key(|record| record.material_id);",
        ],
    );

    for (path, source) in [
        ("asset/assets/material/material_asset.rs", parent.as_str()),
        (
            "asset/assets/material/material_asset/management.rs",
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
        ("material asset doc", material_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M4 material asset management record owner split",
                "runtime_15_material_asset_management_record_owner_split_static_passed_cargo_deferred",
                "asset/assets/material/material_asset.rs",
                "asset/assets/material/material_asset/management.rs",
                "runtime_15_material_asset_management_records_are_child_owner",
            ],
        );
    }
    assert_contains_all(
        "status-output row data",
        &status_rows,
        &[
            "Runtime 15 M4 material asset management record owner split",
            "runtime_15_material_asset_management_record_owner_split_static_passed_cargo_deferred",
            "asset/assets/material/material_asset.rs",
            "asset/assets/material/material_asset/management.rs",
            "runtime_15_material_asset_management_records_are_child_owner",
        ],
    );
}
