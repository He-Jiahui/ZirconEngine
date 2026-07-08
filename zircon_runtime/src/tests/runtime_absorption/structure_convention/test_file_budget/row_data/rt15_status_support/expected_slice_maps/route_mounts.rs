use super::*;

#[test]
fn runtime_15_status_support_expected_slice_row_data_owner_is_folder_backed() {
    let parent = read_runtime_src(STATUS_SUPPORT_EXPECTED_SLICE_MAPS_PATH);

    for (module_name, path, representative_row) in EXPECTED_SLICE_MAP_CHILDREN {
        let module_mount = format!("mod {module_name};");
        let relative_path = path
            .rsplit_once("status_support/")
            .map(|(_, relative)| relative)
            .unwrap_or(path);
        let const_name = format!(
            "{}_EXPECTED_STATUS_OUTPUT_SLICES",
            module_name.to_uppercase()
        );
        assert_contains_all(
            "status-support expected-slice row-data parent mounts child owner",
            &parent,
            &[module_mount.as_str(), relative_path, const_name.as_str()],
        );

        let child_source = child_sources::expected_slice_child_source(module_name, path);
        assert_contains_all(path, &child_source, &[*representative_row]);
    }

    assert!(
        !parent.contains("pub(super) const EXPECTED_STATUS_OUTPUT_SLICES"),
        "expected_slice_maps.rs should not keep the old merged row-data group"
    );
}
