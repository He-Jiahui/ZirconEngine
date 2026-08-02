#[test]
fn runtime_14_module_family_mirror_docs_match_structure_audit_counts() {
    fn count_rust_files(path: &std::path::Path) -> usize {
        std::fs::read_dir(path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
            .map(|entry| {
                entry
                    .unwrap_or_else(|error| panic!("failed to read directory entry: {error}"))
                    .path()
            })
            .map(|path| {
                if path.is_dir() {
                    count_rust_files(&path)
                } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
                    1
                } else {
                    0
                }
            })
            .sum()
    }

    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for (family, expected_count) in [
        ("animation", 17usize),
        ("navigation", 15),
        ("diagnostic_log", 31),
        ("engine_module", 8),
    ] {
        let family_dir = runtime_root.join("src").join(family);
        assert_eq!(
            count_rust_files(&family_dir),
            expected_count,
            "Runtime 14 module family `{family}` changed file count; update module_family_boundary and mirror docs before changing the root-seat judgement"
        );
    }

    let guard_sources = [
        include_str!("../runtime_root.rs"),
        include_str!("../core_spine.rs"),
        include_str!("animation_backlog.rs"),
        include_str!("animation_status_json.rs"),
        include_str!("mirror_docs.rs"),
        include_str!("navigation.rs"),
        include_str!("root_seats.rs"),
        include_str!("../../../../diagnostic_log/diagnostics.rs"),
        include_str!("../../../../engine_module/tests.rs"),
    ];
    for required_anchor in [
        "runtime_animation_backlog_boundary_requires_doc_update",
        "runtime_navigation_boundary_file_set_requires_doc_update",
        "diagnostic_log_snapshot_bridge_stays_single_owner",
        "engine_module_declared_layer_does_not_own_runtime_lifecycle",
        "runtime_14_module_family_root_seats_match_documented_judgements",
        "runtime_14_module_family_mirror_docs_match_structure_audit_counts",
        "runtime_animation_status_json_boundary_sanitizes_non_finite_values",
    ] {
        assert!(
            guard_sources
                .iter()
                .any(|source| source.contains(required_anchor)),
            "Runtime 14 module-family guard anchor `{required_anchor}` should remain present"
        );
    }

    let current_mirror_docs = [
        (
            "Runtime 14 plan",
            include_str!(
                "../../../../../../docs/plans/zircon_runtime/runtime/14-runtime-module-family-closeout.md"
            ),
        ),
        (
            "Runtime 14 current output record",
            include_str!(
                "../../../../../../docs/plans/zircon_runtime/runtime/14/2026-07-09-runtime-module-family-closeout-output-records.md"
            ),
        ),
    ];

    for (doc_name, doc_source) in current_mirror_docs {
        for required_anchor in [
            "module_family_boundary",
            "expected_family_count = 4",
            "animation = 17",
            "navigation = 15",
            "diagnostic_log = 31",
            "engine_module = 8",
            "root_seat_guard_present = true",
            "animation_status_json_guard_present = true",
            "animation_status_json_anchor_count = 8",
            "missing_animation_status_json_anchors = []",
            "module_family_guard_anchor_count = 7",
            "missing_module_family_guard_anchors = []",
            "cargo_gate_anchor_count = 5",
            "missing_cargo_gate_anchors = []",
            "risks = []",
            "runtime_14_module_family_mirror_docs_match_structure_audit_counts",
        ] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should mirror Runtime 14 module-family audit anchor `{required_anchor}`"
            );
        }
    }

    let navigation_doc =
        include_str!("../../../../../../docs/zircon_runtime/navigation/runtime.md");
    for required_anchor in [
        "15 Rust owner files",
        "operation/{mod,handler,registration}.rs",
        "runtime_navigation_boundary_file_set_requires_doc_update",
    ] {
        assert!(
            navigation_doc.contains(required_anchor),
            "navigation runtime doc should keep current Runtime 14 anchor `{required_anchor}`"
        );
    }
}
