use super::*;

#[path = "exports/runtime_15_m3_parent.rs"]
mod runtime_15_m3_parent;
#[path = "exports/runtime_15_parent.rs"]
mod runtime_15_parent;
#[path = "exports/status_mirrors.rs"]
mod status_mirrors;
#[path = "exports/top_level.rs"]
mod top_level;

const EXPORT_CHILDREN: &[(&str, &str, &str, &[&str])] = &[
    (
        "runtime_15_m3_parent",
        EXPORTS_RUNTIME_15_M3_PARENT_PATH,
        "runtime_15_m3_child_groups_exports_runtime_15_m3_parent_is_child_owned",
        &["m3/production_guard_support.rs"],
    ),
    (
        "runtime_15_parent",
        EXPORTS_RUNTIME_15_PARENT_PATH,
        "runtime_15_m3_child_groups_exports_runtime_15_parent_is_child_owned",
        &[
            "m3::PRODUCTION_GUARD_SUPPORT_CORE_AND_EVIDENCE_EVIDENCE_ANCHOR_EXPECTED_STATUS_OUTPUT_SLICES",
            "m3::PRODUCTION_GUARD_SUPPORT_MODULE_LAYOUT_BASE_EXPECTED_STATUS_OUTPUT_SLICES",
            "m3::PRODUCTION_GUARD_SUPPORT_MODULE_LAYOUT_CHILD_SUMMARY_EXPECTED_STATUS_OUTPUT_SLICES",
            "m3::PRODUCTION_GUARD_SUPPORT_STATUS_DOCS_FOUNDATION_M2_EXPECTED_STATUS_OUTPUT_SLICES",
            "m3::PRODUCTION_GUARD_SUPPORT_STATUS_DOCS_CHILD_GROUP_STATUS_DOC_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    ),
    (
        "status_mirrors",
        EXPORTS_STATUS_MIRRORS_PATH,
        EXPORTS_CHILD_SPLIT_GUARD_NAME,
        &[EXPORTS_CHILD_SPLIT_STATUS_NAME, EXPORTS_CHILD_SPLIT_STATUS_ID],
    ),
    (
        "top_level",
        EXPORTS_TOP_LEVEL_PATH,
        "runtime_15_m3_child_groups_exports_top_level_is_child_owned",
        &[
            "runtime_15::RUNTIME_15_M3_PRODUCTION_GUARD_SUPPORT_CORE_AND_EVIDENCE_EVIDENCE_ANCHOR_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_PRODUCTION_GUARD_SUPPORT_MODULE_LAYOUT_BASE_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_PRODUCTION_GUARD_SUPPORT_MODULE_LAYOUT_CHILD_SUMMARY_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_PRODUCTION_GUARD_SUPPORT_STATUS_DOCS_FOUNDATION_M2_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_PRODUCTION_GUARD_SUPPORT_EXPECTED_SLICE_GUARDS_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    ),
];

#[test]
fn runtime_15_status_output_m3_row_data_child_owner_split() {
    let route_source = read_runtime_src(EXPORTS_GUARD_PATH);
    let core_status_source = m3_child_group_core_status_source_blob();
    let route_metadata = [route_source.as_str(), core_status_source.as_str()].join("\n");

    for (module_name, path, guard_name, labels) in EXPORT_CHILDREN {
        let module_mount = format!("mod {module_name};");
        assert_contains_all(
            "M3 child-groups exports route mounts child owner",
            &route_source,
            &[
                format!("#[path = \"exports/{module_name}.rs\"]").as_str(),
                module_mount.as_str(),
            ],
        );
        assert_contains_all(
            "M3 child-groups exports route records child guard metadata",
            &route_metadata,
            &[*guard_name],
        );
        let child_source = read_runtime_src(path);
        let child_label_source = [child_source.as_str(), core_status_source.as_str()].join("\n");
        assert_contains_all(path, &child_source, &[*guard_name]);
        assert_contains_all(path, &child_label_source, labels);
        assert!(
            child_source.lines().count() < 100,
            "{path} should stay below the exports child-owner budget"
        );
    }
}
