use super::runtime_src_path;

#[test]
fn runtime_15_structure_guards_use_durable_evidence_not_session_notes() {
    let runtime_absorption_root = runtime_src_path("tests/runtime_absorption");
    let session_note_root = [".codex", "sessions", ""].join("/");
    let mut pending = vec![runtime_absorption_root];
    let mut violations = Vec::new();

    while let Some(path) = pending.pop() {
        let entries = std::fs::read_dir(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        for entry in entries {
            let entry = entry.expect("runtime absorption guard directory entry should be readable");
            let entry_path = entry.path();
            if entry_path.is_dir() {
                pending.push(entry_path);
            } else if entry_path
                .extension()
                .and_then(|extension| extension.to_str())
                == Some("rs")
            {
                let source = std::fs::read_to_string(&entry_path).unwrap_or_else(|error| {
                    panic!("failed to read {}: {error}", entry_path.display())
                });
                if source.contains(&session_note_root) {
                    violations.push(entry_path.display().to_string());
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "runtime absorption guards must use durable numbered-plan evidence instead of session notes: {violations:#?}"
    );
}
