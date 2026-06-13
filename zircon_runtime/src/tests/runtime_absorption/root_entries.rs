#[test]
fn builtin_root_stays_structural_after_runtime_module_split() {
    let source = include_str!("../../builtin/mod.rs");

    for required in [
        "mod runtime_modules;",
        "pub use runtime_modules::builtin_runtime_modules;",
    ] {
        assert!(
            source.contains(required),
            "expected builtin/mod.rs to keep structural wiring `{required}`"
        );
    }

    for forbidden in [
        "use std::sync::Arc;",
        "use crate::engine_module::EngineModule;",
        "pub fn builtin_runtime_modules()",
        "fn runtime_extension_modules()",
        "Arc::new(",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected builtin/mod.rs to stay structural after split, found `{forbidden}`"
        );
    }
}

#[test]
fn runtime_crate_root_does_not_flatten_plugin_surface() {
    let source = include_str!("../../lib.rs");

    assert!(
        source.contains("pub mod plugin;"),
        "runtime crate root should expose the plugin namespace owner"
    );

    assert!(
        !source.contains("pub use plugin::{"),
        "plugin DTOs, native ABI types, export plans, and catalogs should be imported through zircon_runtime::plugin"
    );

    for flattened_symbol in [
        "PluginPackageManifest",
        "RuntimePluginCatalog",
        "NativePluginLoader",
        "ExportBuildPlan",
        "RuntimeExtensionRegistry",
    ] {
        assert!(
            !source.contains(flattened_symbol),
            "runtime crate root should not flatten plugin symbol `{flattened_symbol}`"
        );
    }
}

#[test]
fn runtime_crate_root_does_not_flatten_builtin_module_assembly_functions() {
    let source = include_str!("../../lib.rs");

    assert!(
        source.contains("pub mod builtin;"),
        "runtime crate root should expose the builtin namespace owner"
    );

    for flattened_function in [
        "builtin_runtime_modules",
        "default_manifest_for_target",
        "manifest_for_runtime_profile",
        "manifest_with_mode_baseline",
        "runtime_core_modules",
        "runtime_modules_for_runtime_profile",
        "runtime_modules_for_target",
    ] {
        assert!(
            !source.contains(flattened_function),
            "runtime crate root should not flatten builtin module assembly function `{flattened_function}`"
        );
    }
}

#[test]
fn core_root_retires_channel_and_service_alias_fragments() {
    let source = include_str!("../../core/mod.rs");
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let core_dir = manifest_dir.join("src").join("core");

    for forbidden in [
        "mod channel_util;",
        "mod types;",
        "pub use channel_util::",
        "pub use types::",
        "ChannelReceiver",
        "ChannelSender",
        "ServiceObject",
        "spawn_named_thread",
        "recv_latest",
        "wait_for",
    ] {
        assert!(
            !source.contains(forbidden),
            "core/mod.rs should route `{forbidden}` through the decided framework/runtime owners"
        );
    }

    for removed_file in ["channel_util.rs", "types.rs"] {
        assert!(
            !core_dir.join(removed_file).exists(),
            "core root should not keep retired fragment file `{removed_file}`"
        );
    }

    let required_files: &[&[&str]] = &[
        &["framework", "channel.rs"],
        &["runtime", "tasks", "mod.rs"],
        &["runtime", "descriptors", "service_object.rs"],
    ];

    for required_file in required_files {
        let mut path = core_dir.clone();
        for segment in required_file.iter().copied() {
            path.push(segment);
        }
        assert!(
            path.exists(),
            "expected migrated owner file to exist at {}",
            path.display()
        );
    }
}

#[test]
fn core_root_retires_runtime_kernel_fragment_files() {
    let source = include_str!("../../core/mod.rs");
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let core_dir = manifest_dir.join("src").join("core");

    for forbidden in [
        "mod error;",
        "mod job_scheduler;",
        "mod lifecycle;",
        "mod time;",
        "pub mod modules;",
        "pub mod state;",
        "pub mod tasks;",
        "pub use error::",
        "pub use job_scheduler::",
        "pub use lifecycle::",
        "pub use state::",
        "pub use tasks::",
        "pub use time::",
    ] {
        assert!(
            !source.contains(forbidden),
            "core/mod.rs should re-export `{forbidden}` through core::runtime, not root-owned fragments"
        );
    }

    for removed_file in ["error.rs", "job_scheduler.rs", "lifecycle.rs", "time.rs"] {
        assert!(
            !core_dir.join(removed_file).exists(),
            "core root should not keep retired runtime kernel fragment file `{removed_file}`"
        );
    }

    assert!(
        !core_dir.join("modules").exists(),
        "core root should not keep retired runtime module descriptor directory `modules`"
    );
    assert!(
        !core_dir.join("state").exists(),
        "core root should not keep retired framework state contract directory `state`"
    );
    assert!(
        !core_dir.join("tasks").exists(),
        "core root should not keep retired runtime task pool directory `tasks`"
    );

    let required_files: &[&[&str]] = &[
        &["framework", "error.rs"],
        &["framework", "state", "mod.rs"],
        &["runtime", "lifecycle.rs"],
        &["runtime", "modules", "mod.rs"],
        &["runtime", "tasks", "pool.rs"],
        &["runtime", "tasks", "pools.rs"],
        &["runtime", "tasks", "report.rs"],
        &["runtime", "tasks", "thread_assignment.rs"],
        &["runtime", "tasks", "job_scheduler.rs"],
        &["runtime", "time.rs"],
    ];

    for required_file in required_files {
        let mut path = core_dir.clone();
        for segment in required_file.iter().copied() {
            path.push(segment);
        }
        assert!(
            path.exists(),
            "expected migrated runtime owner file to exist at {}",
            path.display()
        );
    }
}

#[test]
fn core_root_splits_event_dto_from_runtime_event_bus() {
    let source = include_str!("../../core/mod.rs");
    let framework_source = include_str!("../../core/framework/mod.rs");
    let runtime_source = include_str!("../../core/runtime/mod.rs");
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let core_dir = manifest_dir.join("src").join("core");

    for forbidden in ["mod event_bus;", "pub use event_bus::"] {
        assert!(
            !source.contains(forbidden),
            "core/mod.rs should not keep retired event bus fragment wiring `{forbidden}`"
        );
    }

    for removed_entry in ["event_bus.rs", "event_bus"] {
        assert!(
            !core_dir.join(removed_entry).exists(),
            "core root should not keep retired event bus fragment `{removed_entry}`"
        );
    }

    assert!(
        source.contains("pub use framework::events::EngineEvent;"),
        "core root should route EngineEvent through core::framework::events"
    );
    assert!(
        source.contains("EventBus"),
        "core root should keep the curated EventBus facade from the runtime owner"
    );
    assert!(
        framework_source.contains("pub mod events;"),
        "core::framework should own the event DTO namespace"
    );
    assert!(
        runtime_source.contains("mod events;"),
        "core::runtime should own the event bus implementation namespace"
    );
    assert!(
        runtime_source.contains("pub use events::EventBus;"),
        "core::runtime should re-export EventBus from its owner module"
    );

    let required_files: &[&[&str]] = &[
        &["framework", "events.rs"],
        &["runtime", "events.rs"],
        &["runtime", "events", "failure.rs"],
        &["runtime", "events", "prune.rs"],
        &["runtime", "events", "publish.rs"],
        &["runtime", "events", "subscribe.rs"],
    ];

    for required_file in required_files {
        let mut path = core_dir.clone();
        for segment in required_file.iter().copied() {
            path.push(segment);
        }
        assert!(
            path.exists(),
            "expected migrated event owner file to exist at {}",
            path.display()
        );
    }
}

#[test]
fn core_root_reexports_runtime_diagnostics_without_root_directory() {
    let source = include_str!("../../core/mod.rs");
    let runtime_source = include_str!("../../core/runtime/mod.rs");
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let core_dir = manifest_dir.join("src").join("core");

    assert!(
        !source.contains("pub mod diagnostics;"),
        "core/mod.rs should not own diagnostics as a root source directory"
    );
    assert!(
        source.contains("pub use runtime::diagnostics;"),
        "core/mod.rs should keep the curated diagnostics facade through the runtime owner"
    );
    assert!(
        runtime_source.contains("pub mod diagnostics;"),
        "core::runtime should own the diagnostics namespace"
    );
    assert!(
        !core_dir.join("diagnostics").exists(),
        "core root should not keep retired diagnostics directory"
    );
    assert!(
        core_dir
            .join("runtime")
            .join("diagnostics")
            .join("mod.rs")
            .exists(),
        "expected diagnostics owner directory under core/runtime/diagnostics"
    );
}

#[test]
fn core_module_tree_matches_decided_spine_shape() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let core_dir = manifest_dir.join("src").join("core");
    let actual_entries = std::fs::read_dir(&core_dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", core_dir.display()))
        .map(|entry| {
            entry
                .unwrap_or_else(|error| panic!("failed to read core entry: {error}"))
                .file_name()
                .into_string()
                .unwrap_or_else(|name| panic!("non-utf8 core entry name: {name:?}"))
        })
        .collect::<std::collections::BTreeSet<_>>();
    let expected_entries = [
        "framework",
        "manager",
        "math",
        "mod.rs",
        "resource",
        "runtime",
    ]
    .into_iter()
    .map(String::from)
    .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(
        actual_entries, expected_entries,
        "core root should contain only the decided spine directories plus mod.rs"
    );
}

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
    let expected_entries = ["mod.rs", "module.rs", "runtime.rs"]
        .into_iter()
        .map(String::from)
        .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(
        actual_entries, expected_entries,
        "runtime navigation fallback changed file shape; update docs/zircon_runtime/navigation/runtime.md and Runtime 14 before adding behavior files"
    );

    let module_source = include_str!("../../navigation/module.rs");
    assert!(
        module_source
            .contains("Built-in baked navmesh pathfinding and lightweight agent avoidance"),
        "navigation module descriptor should keep fallback runtime scope explicit"
    );

    let boundary_doc = include_str!("../../../../docs/zircon_runtime/navigation/runtime.md");
    for required_anchor in [
        "Runtime 14 Boundary Judgment",
        "built-in fallback implementation",
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
    let animation_source = include_str!("../../animation/mod.rs");
    assert!(
        animation_source.contains("pub use sequence::apply_sequence_to_world;"),
        "animation root should keep the public sequence application hook explicit"
    );

    let sequence_tests = include_str!("../../animation/sequence/tests.rs");
    for required_sequence_anchor in [
        "sequence_applies_mesh_renderer_morph_weight_track",
        "MeshRenderer.morph_weights.1",
    ] {
        assert!(
            sequence_tests.contains(required_sequence_anchor),
            "animation sequence tests should keep morph-weight property-track evidence `{required_sequence_anchor}`"
        );
    }

    let boundary_doc = include_str!("../../../../docs/zircon_runtime/animation/runtime.md");
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
fn runtime_14_module_family_root_seats_match_documented_judgements() {
    let crate_root = include_str!("../../lib.rs");

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
        "../../../../docs/plans/zircon_runtime/runtime/14-runtime-module-family-closeout.md"
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

    let animation_doc = include_str!("../../../../docs/zircon_runtime/animation/runtime.md");
    assert!(
        animation_doc.contains("should keep its crate-root seat"),
        "animation runtime doc should keep the crate-root seat judgement"
    );

    let navigation_doc = include_str!("../../../../docs/zircon_runtime/navigation/runtime.md");
    assert!(
        navigation_doc.contains("built-in fallback implementation"),
        "navigation runtime doc should keep the fallback root-seat judgement"
    );

    let diagnostic_log_doc = include_str!("../../../../docs/zircon_runtime/diagnostic_log/mod.md");
    assert!(
        diagnostic_log_doc.contains("Keep `diagnostic_log` at crate root."),
        "diagnostic_log doc should keep the crate-root process diagnostics judgement"
    );

    let engine_module_doc =
        include_str!("../../../../docs/zircon_runtime/engine_module/relationship.md");
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
        ("animation", 27usize),
        ("navigation", 3),
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

    let mirror_docs = [
        (
            "Runtime 14 plan",
            include_str!("../../../../docs/plans/zircon_runtime/runtime/14-runtime-module-family-closeout.md"),
        ),
        (
            "runtime index",
            include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md"),
        ),
        (
            "M0 review",
            include_str!("../../../../docs/engine-architecture/runtime-architecture-review-m0.md"),
        ),
        (
            "interface convergence",
            include_str!("../../../../docs/engine-architecture/runtime-interface-convergence.md"),
        ),
    ];

    for (doc_name, doc_source) in mirror_docs {
        for required_anchor in [
            "module_family_boundary",
            "expected_family_count = 4",
            "animation = 27",
            "navigation = 3",
            "diagnostic_log = 7",
            "engine_module = 8",
            "root_seat_guard_present = true",
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
