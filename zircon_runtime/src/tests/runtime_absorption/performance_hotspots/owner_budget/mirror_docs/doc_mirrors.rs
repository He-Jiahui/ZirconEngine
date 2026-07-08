use super::sources::{assert_contains_all, MirrorDocsSources};

pub(super) fn assert_runtime_07_mirror_docs(sources: &MirrorDocsSources) {
    for (doc_name, doc_source) in sources.mirror_docs() {
        assert_contains_all(
            doc_name,
            doc_source,
            &[
                "performance_hotpath_boundary",
                "expected_source_file_count = 46",
                "expected_test_file_count = 91",
                "frame_span_anchor_count = 9",
                "query_counter_anchor_count = 32",
                "change_counter_anchor_count = 13",
                "extract_counter_anchor_count = 21",
                "asset_worker_anchor_count = 13",
                "animation_scene_anchor_count = 19",
                "profile_counter_hotspot_anchor_count = 8",
                "hotspot_guard_anchor_count = 32",
                "test_anchor_count = 29",
                "doc_anchor_count = 35",
                "cargo_gate_anchor_count = 5",
                "stale_hotspot_placeholder_present = false",
                "large_file_m1_gate_status = classified-and-clear",
                "large_file_hotspot_count = 0",
                "large_file_migration_debt_count = 0",
                "large_file_owner_class_count = 0",
                "large_file_unclassified_hotspot_count = 0",
                "missing_large_file_owner_classes = []",
                "missing_doc_anchors = []",
                "missing_cargo_gate_anchors = []",
                "mirror_docs_guard_present = true",
                "risks = []",
                "runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts",
            ],
        );
    }
}
