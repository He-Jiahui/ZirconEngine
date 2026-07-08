use super::sources::{assert_contains_all, HotspotInventorySplitSources};

pub(super) fn assert_hotspot_inventory_status_docs(sources: &HotspotInventorySplitSources) {
    for (label, source) in [
        ("Runtime 15 plan", sources.runtime_15_plan),
        ("Runtime index", sources.runtime_index),
        ("Runtime 07 plan", sources.runtime_07_plan),
        ("review findings", sources.review_findings),
        ("structure convention", sources.structure_convention),
        ("module convention doc", sources.module_doc),
        ("hotspot inventory doc", sources.hotspot_doc),
        ("status-output row data", sources.status_rows),
        ("session note", sources.session_note),
    ] {
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

    assert_contains_all(
        "status-output status slice",
        sources.status_slice,
        &[
            "Runtime 15 M3 Runtime 07 hotspot-inventory split-layout guard folder-backed split",
            "runtime_15_runtime_07_hotspot_inventory_split_layout_guard_folder_backed_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "status-output date slice",
        sources.date_slice,
        &[
            "Runtime 15 M3 Runtime 07 hotspot-inventory split-layout guard folder-backed split",
            "2026-07-06",
        ],
    );
}
