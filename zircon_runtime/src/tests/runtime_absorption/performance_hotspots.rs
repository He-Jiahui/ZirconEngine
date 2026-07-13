#[path = "performance_hotspots/artifact_render_diagnostics_splits.rs"]
mod artifact_render_diagnostics_splits;
#[path = "performance_hotspots/hotspot_inventory.rs"]
mod hotspot_inventory;
#[path = "performance_hotspots/owner_budget.rs"]
mod owner_budget;
#[path = "performance_hotspots/scene_project_splits.rs"]
mod scene_project_splits;
#[path = "performance_hotspots/submit_context.rs"]
mod submit_context;
#[path = "performance_hotspots/submit_error_paths.rs"]
mod submit_error_paths;

#[test]
fn runtime_07_performance_guards_use_durable_evidence_not_session_notes() {
    let runtime_src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let root = runtime_src.join("tests/runtime_absorption/performance_hotspots");
    let session_path = [".codex", "sessions", ""].join("/");
    let mut pending = vec![root];
    let mut source_files =
        vec![runtime_src.join("tests/runtime_absorption/performance_hotspots.rs")];
    let mut violations = Vec::new();

    while let Some(path) = pending.pop() {
        for entry in std::fs::read_dir(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
        {
            let entry = entry.expect("performance guard directory entry should be readable");
            let entry_path = entry.path();
            if entry_path.is_dir() {
                pending.push(entry_path);
            } else if entry_path.extension().and_then(|value| value.to_str()) == Some("rs") {
                source_files.push(entry_path);
            }
        }
    }

    for source_file in source_files {
        let source = std::fs::read_to_string(&source_file)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", source_file.display()));
        if source.contains(&session_path) {
            violations.push(source_file.display().to_string());
        }
    }

    assert!(
        violations.is_empty(),
        "Runtime 07 performance guards must use numbered-plan evidence: {violations:#?}"
    );
}
