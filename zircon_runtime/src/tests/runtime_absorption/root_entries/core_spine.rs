#[test]
fn core_root_retires_channel_and_service_alias_fragments() {
    let source = include_str!("../../../core/mod.rs");
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
    let source = include_str!("../../../core/mod.rs");
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
    let source = include_str!("../../../core/mod.rs");
    let framework_source = include_str!("../../../core/framework/mod.rs");
    let runtime_source = include_str!("../../../core/runtime/mod.rs");
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
        &["runtime", "events", "diagnostics.rs"],
        &["runtime", "events", "prune.rs"],
        &["runtime", "events", "publish.rs"],
        &["runtime", "events", "subscribe.rs"],
        &["runtime", "events", "subscriber.rs"],
        &["runtime", "events", "topic.rs"],
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
    let source = include_str!("../../../core/mod.rs");
    let runtime_source = include_str!("../../../core/runtime/mod.rs");
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
