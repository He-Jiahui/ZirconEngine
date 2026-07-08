use super::support::{classify_rayon_reference, collect_rayon_references, rust_source_files};

#[test]
fn rayon_is_only_reachable_through_core_task_primitives() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_root = manifest_root.join("src");
    let files = rust_source_files(&source_root);
    let references = collect_rayon_references(manifest_root, &files);

    assert!(
        references
            .iter()
            .any(|reference| reference.path == "src/core/runtime/tasks/pool.rs"),
        "rayon boundary guard should see the task pool owner"
    );
    assert!(
        references
            .iter()
            .any(|reference| reference.path == "src/core/runtime/tasks/parallel_for.rs"),
        "rayon boundary guard should see the parallel_for owner"
    );

    let unclassified = references
        .iter()
        .filter(|reference| classify_rayon_reference(&reference.path).is_none())
        .map(|reference| {
            format!(
                "{}:{}: {}",
                reference.path, reference.line, reference.snippet
            )
        })
        .collect::<Vec<_>>();

    assert!(
        unclassified.is_empty(),
        "direct rayon usage must be routed through core task primitives:\n{}",
        unclassified.join("\n")
    );
}

#[test]
fn rayon_boundary_guard_rejects_unclassified_runtime_source() {
    assert_eq!(
        classify_rayon_reference("src/core/runtime/tasks/pool.rs"),
        Some("core-task-pool-rayon-owner")
    );
    assert_eq!(
        classify_rayon_reference("src/core/runtime/tasks/parallel_for.rs"),
        Some("core-task-parallel-for-owner")
    );
    assert_eq!(
        classify_rayon_reference("src/graphics/visibility/culling/parallel_frustum.rs"),
        None
    );
    assert_eq!(
        classify_rayon_reference("src/scene/ecs/schedule_parallel_executor.rs"),
        None
    );
}
