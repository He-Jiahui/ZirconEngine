use super::sources::{assert_contains_all, MirrorDocsSources};

pub(super) fn assert_audit_wiring_anchors(sources: &MirrorDocsSources) {
    assert_contains_all(
        "performance hotpath anchor inventory",
        sources.audit_anchor_inventory,
        &[
            "ANIMATION_SCENE_ANCHORS",
            "MIRROR_DOCS_GUARD",
            "\"runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts\"",
        ],
    );

    assert_contains_all(
        "performance hotpath boundary audit",
        sources.audit_script,
        &[
            "from runtime_structure_audits.performance_hotpath_source_inventory import",
            "from runtime_structure_audits.performance_hotpath_anchor_inventory import",
            "\"mirror_docs_guard_present\"",
        ],
    );
}
