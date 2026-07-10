use std::path::Path;

use super::behavior_anchors::assert_runtime_01_behavior_anchors;
use super::guard_anchors::assert_runtime_01_guard_anchors;
use super::manifest_inventory::{assert_runtime_01_manifests_exist, EXPECTED_RUNTIME_01_MANIFESTS};

#[test]
fn runtime_01_tech_stack_mirror_docs_match_structure_audit_counts() {
    assert_eq!(EXPECTED_RUNTIME_01_MANIFESTS.len(), 5);

    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert_runtime_01_manifests_exist(runtime_root);
    assert_runtime_01_guard_anchors();
    assert_runtime_01_behavior_anchors();
    assert_mirror_docs_match_structure_audit();
}

fn assert_mirror_docs_match_structure_audit() {
    let evidence_docs = [
        (
            "runtime tech-stack doc",
            include_str!("../../../../../docs/engine-architecture/runtime-tech-stack.md"),
        ),
        (
            "Runtime 01 output archive",
            include_str!(
                "../../../../../docs/plans/zircon_runtime/runtime/01/2026-07-09-tech-stack-and-dependency-governance-output-records.md"
            ),
        ),
        (
            "M0 review",
            include_str!("../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"),
        ),
        (
            "interface convergence",
            include_str!("../../../../../docs/engine-architecture/runtime-interface-convergence.md"),
        ),
    ];

    for (doc_name, doc_source) in evidence_docs {
        for required_anchor in [
            "tech_stack_boundary",
            "tech_stack_source_inventory.py",
            "tech_stack_anchor_inventory.py",
            "expected_manifest_count = 5",
            "expected_non_dependency_count = 5",
            "zip_dependency_count = 1",
            "expected_zip_dependency_count = 1",
            "zip_dependency_violations = []",
            "tech_stack_guard_count = 12",
            "editor_only_candidate_count = 3",
            "behavior_test_anchor_count = 6",
            "missing_behavior_test_anchors = []",
            "jolt_feature_slot_count = 2",
            "declared_removed_dependencies = []",
            "rapier_or_avian_dependencies = []",
            "mirror_docs_guard_present = true",
            "risks = []",
            "runtime_01_tech_stack_mirror_docs_match_structure_audit_counts",
        ] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should mirror Runtime 01 tech-stack audit anchor `{required_anchor}`"
            );
        }
    }

    for (doc_name, doc_source, route_anchor) in [
        (
            "Runtime 01 plan",
            include_str!(
                "../../../../../docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md"
            ),
            "01/2026-07-09-tech-stack-and-dependency-governance-output-records.md",
        ),
        (
            "runtime index",
            include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md"),
            "01-tech-stack-and-dependency-governance.md",
        ),
    ] {
        assert!(
            doc_source.contains(route_anchor)
                && doc_source.contains("此处仅展示当前现状的概述"),
            "{doc_name} should route Runtime 01 concrete evidence to its numbered archive"
        );
    }
}
