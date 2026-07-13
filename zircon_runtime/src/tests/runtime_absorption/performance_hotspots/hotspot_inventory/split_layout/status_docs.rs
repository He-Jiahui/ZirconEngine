use super::sources::{assert_contains_all, HotspotInventorySplitSources};

pub(super) fn assert_hotspot_inventory_status_docs(sources: &HotspotInventorySplitSources) {
    for (label, source) in [("Runtime 07 numbered archive", sources.runtime_07_archive)] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 Runtime 07 hotspot-inventory split-layout guard folder-backed split",
                "runtime_15_runtime_07_hotspot_inventory_split_layout_guard_folder_backed_static_passed_cargo_deferred",
                "hotspot_inventory/split_layout",
                "runtime_15_runtime_07_hotspot_inventory_split_layout_guard_folder_backed_split",
                "expected_test_file_count = 54",
            ],
        );
    }
}
