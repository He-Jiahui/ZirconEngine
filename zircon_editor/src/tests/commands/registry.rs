use std::fs;
use std::path::Path;

use crate::core::commands::{
    EditorCommandAction, EditorCommandCategory, EditorCommandDescriptor, EditorCommandRegistry,
    EditorCommandRegistryError,
};
use crate::core::editor_event::{EditorEvent, EditorEventTransient};
use crate::core::editor_operation::EditorOperationPath;

#[test]
fn command_registry_rejects_duplicate_operation_path_ids() {
    let id = EditorOperationPath::parse("test.command.duplicate").unwrap();
    let descriptor = || {
        EditorCommandDescriptor::new(
            id.clone(),
            "Duplicate",
            EditorCommandCategory::Command,
            EditorCommandAction::Emit(EditorEvent::Transient(
                EditorEventTransient::OpenCommandPalette,
            )),
        )
    };

    let error = EditorCommandRegistry::new(vec![descriptor(), descriptor()]).unwrap_err();

    assert_eq!(error, EditorCommandRegistryError::DuplicateCommand(id));
}

#[test]
fn command_owner_hard_cut_leaves_no_ui_host_registry_or_retired_symbols() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(!manifest_dir.join("src/ui/host/commands").exists());

    let retired_symbols = [
        ["EditorOperation", "Descriptor"].concat(),
        ["EditorOperation", "Registry"].concat(),
        ["EditorCommand", "Context"].concat(),
        ["EditorCommand", "Enablement"].concat(),
        ["operation_", "capability_error"].concat(),
    ];
    let source_root = manifest_dir.join("src");
    let mut pending = vec![source_root];
    let mut retired_symbol_hits = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory).expect("read editor source directory") {
            let entry = entry.expect("read editor source entry");
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
                let source = fs::read_to_string(&path).expect("read editor Rust source");
                retired_symbol_hits.extend(
                    retired_symbols
                        .iter()
                        .filter(|retired| source.contains(retired.as_str()))
                        .map(|retired| format!("{}: {retired}", path.display())),
                );
            }
        }
    }

    assert!(
        retired_symbol_hits.is_empty(),
        "retired symbols remain:\n{}",
        retired_symbol_hits.join("\n")
    );
}
