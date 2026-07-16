use super::*;

#[test]
fn runtime_15_plugin_static_manifest_contract_owners_use_domain_names() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let static_manifest_dir =
        manifest_root.join("src/tests/plugin_extensions/static_manifest_contracts");
    let feature_bundles_root = read_text(
        &static_manifest_dir.join("feature_bundles.rs"),
        "feature bundles root",
    );
    let feature_rows = read_text(
        &static_manifest_dir.join("feature_bundles/feature_rows.rs"),
        "feature bundle feature rows",
    );
    let module_rows = read_text(
        &static_manifest_dir.join("feature_bundles/module_rows.rs"),
        "feature bundle module rows",
    );
    let package_coordinates_root = read_text(
        &static_manifest_dir.join("package_coordinates.rs"),
        "package coordinates root",
    );
    let package_coordinates = read_text(
        &static_manifest_dir.join("package_coordinates/coordinates.rs"),
        "package coordinate declarations",
    );
    let resolved_ids = read_text(
        &static_manifest_dir.join("package_coordinates/resolved_ids.rs"),
        "resolved package ids",
    );
    let package_identity_root = read_text(
        &static_manifest_dir.join("package_identity.rs"),
        "package identity root",
    );
    let package_identity_directories = read_text(
        &static_manifest_dir.join("package_identity/directories.rs"),
        "package identity directories",
    );
    let package_kind_root = read_text(
        &static_manifest_dir.join("package_kind.rs"),
        "package kind root",
    );
    let package_kind_feature_rows = read_text(
        &static_manifest_dir.join("package_kind/feature_rows.rs"),
        "package kind feature rows",
    );
    let package_kind_values = read_text(
        &static_manifest_dir.join("package_kind/values.rs"),
        "package kind values",
    );

    for retired in [
        "feature_bundles/helpers.rs",
        "package_coordinates/helpers.rs",
        "package_identity/helpers.rs",
        "package_kind/helpers.rs",
    ] {
        let retired_path = static_manifest_dir.join(retired);
        assert!(
            !retired_path.exists(),
            "static manifest contracts should not keep banned-name helper owner {:?}",
            retired_path
        );
    }

    assert_contains_all(
        "feature bundles root",
        &feature_bundles_root,
        &["mod feature_bundle_rows;"],
    );
    assert_contains_all(
        "feature bundle row callers",
        &(feature_rows + &module_rows),
        &["super::feature_bundle_rows::for_each_feature_bundle"],
    );
    assert_contains_all(
        "package coordinates root",
        &package_coordinates_root,
        &["mod package_coordinate_resolution;"],
    );
    assert_contains_all(
        "package coordinate callers",
        &(package_coordinates + &resolved_ids),
        &[
            "super::package_coordinate_resolution::declares_any_coordinate_field",
            "super::package_coordinate_resolution::resolved_package_id",
        ],
    );
    assert_contains_all(
        "package identity root",
        &package_identity_root,
        &["mod package_id_tokens;"],
    );
    assert_contains_all(
        "package identity callers",
        &package_identity_directories,
        &["super::package_id_tokens::assert_package_id_token"],
    );
    assert_contains_all(
        "package kind root",
        &package_kind_root,
        &["mod package_kind_fields;"],
    );
    assert_contains_all(
        "package kind callers",
        &(package_kind_feature_rows + &package_kind_values),
        &[
            "super::package_kind_fields::{package_kind_value, table_array_row_count}",
            "super::package_kind_fields::package_kind_value",
        ],
    );

    for source in [
        feature_bundles_root,
        package_coordinates_root,
        package_identity_root,
        package_kind_root,
    ] {
        assert!(
            !source.contains("mod helpers;"),
            "static manifest contract roots should not preserve banned helper module names"
        );
    }

    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    let runtime_index = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let plugin_extension_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/plugin/package_manifest.md",
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
        ("plugin extension doc", plugin_extension_doc),
        ("status row data", status_rows),
        ("status slice", status_slice),
        ("date slice", date_slice),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                "Runtime 15 M2 plugin static manifest contract owner naming hard cutover",
                "runtime_15_plugin_static_manifest_contract_owner_naming_hard_cutover_static_passed_cargo_deferred",
                "plugin_extensions/static_manifest_contracts/feature_bundles/feature_bundle_rows.rs",
                "plugin_extensions/static_manifest_contracts/package_coordinates/package_coordinate_resolution.rs",
                "plugin_extensions/static_manifest_contracts/package_identity/package_id_tokens.rs",
                "plugin_extensions/static_manifest_contracts/package_kind/package_kind_fields.rs",
                "runtime_15_plugin_static_manifest_contract_owners_use_domain_names",
            ],
        );
    }
}
