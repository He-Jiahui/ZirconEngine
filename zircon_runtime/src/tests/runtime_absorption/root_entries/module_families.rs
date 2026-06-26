#[test]
fn runtime_navigation_boundary_file_set_requires_doc_update() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let navigation_dir = manifest_dir.join("src").join("navigation");
    let actual_entries = std::fs::read_dir(&navigation_dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", navigation_dir.display()))
        .map(|entry| {
            entry
                .unwrap_or_else(|error| panic!("failed to read navigation entry: {error}"))
                .file_name()
                .into_string()
                .unwrap_or_else(|name| panic!("non-utf8 navigation entry name: {name:?}"))
        })
        .collect::<std::collections::BTreeSet<_>>();
    let expected_entries = ["mod.rs", "module.rs", "runtime", "runtime.rs"]
        .into_iter()
        .map(String::from)
        .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(
        actual_entries, expected_entries,
        "runtime navigation fallback changed root file shape; update docs/zircon_runtime/navigation/runtime.md and Runtime 14 before adding behavior files"
    );

    let runtime_dir = navigation_dir.join("runtime");
    let actual_runtime_entries = std::fs::read_dir(&runtime_dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", runtime_dir.display()))
        .map(|entry| {
            entry
                .unwrap_or_else(|error| panic!("failed to read navigation runtime entry: {error}"))
                .file_name()
                .into_string()
                .unwrap_or_else(|name| panic!("non-utf8 navigation runtime entry name: {name:?}"))
        })
        .collect::<std::collections::BTreeSet<_>>();
    let expected_runtime_entries = [
        "avoidance.rs",
        "baked_mesh.rs",
        "math.rs",
        "state.rs",
        "tests.rs",
        "world_scan.rs",
    ]
    .into_iter()
    .map(String::from)
    .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(
        actual_runtime_entries, expected_runtime_entries,
        "runtime navigation fallback changed owner modules; update docs/zircon_runtime/navigation/runtime.md and Runtime 14 before adding behavior files"
    );

    let module_source = include_str!("../../../navigation/module.rs");
    assert!(
        module_source
            .contains("Built-in baked navmesh pathfinding and lightweight agent avoidance"),
        "navigation module descriptor should keep fallback runtime scope explicit"
    );

    let boundary_doc = include_str!("../../../../../docs/zircon_runtime/navigation/runtime.md");
    for required_anchor in [
        "Runtime 14 Boundary Judgment",
        "built-in fallback implementation",
        "folder-backed runtime owner split",
        "runtime/avoidance.rs",
        "runtime/baked_mesh.rs",
        "runtime/world_scan.rs",
        "zircon_plugins/navigation",
        "runtime_navigation_boundary_file_set_requires_doc_update",
    ] {
        assert!(
            boundary_doc.contains(required_anchor),
            "navigation boundary doc should record `{required_anchor}`"
        );
    }
}

#[test]
fn runtime_animation_backlog_boundary_requires_doc_update() {
    let animation_source = include_str!("../../../animation/mod.rs");
    assert!(
        animation_source.contains("pub use sequence::apply_sequence_to_world;"),
        "animation root should keep the public sequence application hook explicit"
    );

    let sequence_tests = include_str!("../../../animation/sequence/tests.rs");
    for required_sequence_anchor in [
        "sequence_applies_mesh_renderer_morph_weight_track",
        "MeshRenderer.morph_weights.1",
    ] {
        assert!(
            sequence_tests.contains(required_sequence_anchor),
            "animation sequence tests should keep morph-weight property-track evidence `{required_sequence_anchor}`"
        );
    }

    let boundary_doc = include_str!("../../../../../docs/zircon_runtime/animation/runtime.md");
    for required_anchor in [
        "Runtime Animation Module",
        "Root motion",
        "Backlog debt",
        "Morph targets",
        "asset/scene property/sequence tracks",
        "not as a dedicated animation-system morph solver",
        "`render` and `graphics` own GPU skinning and draw submission",
        "Editor authoring tools",
        "future expansion must coordinate asset, render, and graphics owners",
        "runtime_animation_backlog_boundary_requires_doc_update",
    ] {
        assert!(
            boundary_doc.contains(required_anchor),
            "animation runtime doc should record `{required_anchor}`"
        );
    }
}

#[test]
fn runtime_animation_status_json_boundary_sanitizes_non_finite_values() {
    let runtime_status_source = include_str!("../../../core/framework/animation/runtime_status.rs");
    for required_anchor in [
        "serialize_sanitized_non_negative_real",
        "deserialize_sanitized_non_negative_real",
        "serialize_normalized_real",
        "deserialize_normalized_real",
        "impl AnimationPlayerRuntimeStatus",
        "impl AnimationRuntimeStatus",
        "snapshot.time_seconds = self.sanitized_time_seconds()",
        "AnimationPlayerRuntimeStatus::sanitized_snapshot",
    ] {
        assert!(
            runtime_status_source.contains(required_anchor),
            "animation runtime status JSON boundary should keep `{required_anchor}`"
        );
    }

    let framework_tests = include_str!("../../../core/framework/animation/tests.rs");
    for required_anchor in [
        "runtime_status_reports_player_rig_and_gpu_readiness",
        "serde_json::from_value::<AnimationRuntimeStatus>",
        "serde_json::to_value(&status)",
        "status.sanitized_snapshot()",
    ] {
        assert!(
            framework_tests.contains(required_anchor),
            "animation framework tests should lock runtime status JSON sanitization anchor `{required_anchor}`"
        );
    }

    let framework_doc =
        include_str!("../../../../../docs/zircon_runtime/core/framework/animation.md");
    for required_anchor in [
        "AnimationPlayerRuntimeStatus",
        "JSON boundary",
        "`time_seconds` and `playback_speed` serialize and deserialize as finite non-negative values",
        "`weight` serializes and deserializes as a finite `0.0..=1.0` value",
        "AnimationRuntimeStatus::sanitized_snapshot()",
        "JSON `null` values from `NaN` or infinite runtime floats",
    ] {
        assert!(
            framework_doc.contains(required_anchor),
            "animation framework doc should record runtime status JSON sanitization anchor `{required_anchor}`"
        );
    }

    let runtime_14_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/14-runtime-module-family-closeout.md"
    );
    for required_anchor in [
        "animation runtime-status JSON 边界守卫",
        "runtime_animation_status_json_boundary_sanitizes_non_finite_values",
        "AnimationPlayerRuntimeStatus::sanitized_snapshot",
    ] {
        assert!(
            runtime_14_plan.contains(required_anchor),
            "Runtime 14 plan should record animation status JSON boundary anchor `{required_anchor}`"
        );
    }
}

#[test]
fn runtime_14_module_family_root_seats_match_documented_judgements() {
    let crate_root = include_str!("../../../lib.rs");

    for module_name in ["animation", "navigation", "diagnostic_log", "engine_module"] {
        let declaration = format!("pub mod {module_name};");
        assert!(
            crate_root.contains(&declaration),
            "Runtime 14 keeps `{module_name}` as a crate-root module family; update docs/plans/zircon_runtime/runtime/14-runtime-module-family-closeout.md before moving it"
        );

        let flattened_reexport = format!("pub use {module_name}::{{");
        assert!(
            !crate_root.contains(&flattened_reexport),
            "Runtime 14 should keep `{module_name}` behind its namespace instead of flattening the family at crate root"
        );
    }

    let plan_doc = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/14-runtime-module-family-closeout.md"
    );
    for required_anchor in [
        "animation / navigation / diagnostic_log / engine_module",
        "四族在 crate 根的席位与判词一致",
        "runtime_14_module_family_root_seats_match_documented_judgements",
    ] {
        assert!(
            plan_doc.contains(required_anchor),
            "Runtime 14 plan should record the crate-root family judgement anchor `{required_anchor}`"
        );
    }

    let animation_doc = include_str!("../../../../../docs/zircon_runtime/animation/runtime.md");
    assert!(
        animation_doc.contains("should keep its crate-root seat"),
        "animation runtime doc should keep the crate-root seat judgement"
    );

    let navigation_doc = include_str!("../../../../../docs/zircon_runtime/navigation/runtime.md");
    assert!(
        navigation_doc.contains("built-in fallback implementation"),
        "navigation runtime doc should keep the fallback root-seat judgement"
    );

    let diagnostic_log_doc =
        include_str!("../../../../../docs/zircon_runtime/diagnostic_log/mod.md");
    assert!(
        diagnostic_log_doc.contains("Keep `diagnostic_log` at crate root."),
        "diagnostic_log doc should keep the crate-root process diagnostics judgement"
    );

    let engine_module_doc =
        include_str!("../../../../../docs/zircon_runtime/engine_module/relationship.md");
    assert!(
        engine_module_doc.contains("Keep `engine_module` as a crate-root declaration family."),
        "engine_module relationship doc should keep the declared-layering root-seat judgement"
    );
}

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
        ("animation", 28usize),
        ("navigation", 9),
        ("diagnostic_log", 7),
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
        include_str!("runtime_root.rs"),
        include_str!("core_spine.rs"),
        include_str!("module_families.rs"),
        include_str!("../../../diagnostic_log/diagnostics.rs"),
        include_str!("../../../engine_module/tests.rs"),
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

    let mirror_docs = [
        (
            "Runtime 14 plan",
            include_str!(
                "../../../../../docs/plans/zircon_runtime/runtime/14-runtime-module-family-closeout.md"
            ),
        ),
        (
            "runtime index",
            include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md"),
        ),
        (
            "M0 review",
            include_str!(
                "../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
            ),
        ),
        (
            "interface convergence",
            include_str!(
                "../../../../../docs/engine-architecture/runtime-interface-convergence.md"
            ),
        ),
    ];

    for (doc_name, doc_source) in mirror_docs {
        for required_anchor in [
            "module_family_boundary",
            "expected_family_count = 4",
            "animation = 28",
            "navigation = 9",
            "diagnostic_log = 7",
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
}
