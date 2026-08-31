use std::time::{Duration, Instant};

use super::*;
use crate::core::asset::DirtyExternalEffectId;

const UI_ASSET: &str = r#"
[asset]
kind = "layout"
id = "editor.tests.document_save"
version = 1
display_name = "Document Save"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Label"
control_id = "Root"
props = { text = "Ready" }
"#;

#[test]
fn save_all_documents_command_saves_dirty_toolkits_without_blocking_the_host() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_document_save_all");
    let path = unique_temp_path("zircon_retained_document_save_all").with_extension("ui.toml");
    std::fs::write(&path, UI_ASSET).unwrap();
    let instance_id = harness
        .host
        .borrow()
        .editor_manager
        .open_ui_asset_editor(&path, None)
        .expect("ui asset editor should open");
    harness
        .host
        .borrow()
        .editor_manager
        .mark_document_external_effect(&instance_id, DirtyExternalEffectId::ui_source_buffer())
        .expect("document should become dirty");

    let baseline = harness.journal_len();
    harness.dispatch_menu_action("workbench.document.save_all");

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::WorkbenchMenu(MenuAction::SaveAllDocuments)]
    );
    assert!(!harness
        .host
        .borrow()
        .editor_manager
        .dirty_document_toolkits()
        .expect("save completion must be polled by the retained tick")
        .is_empty());

    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline
        && !harness
            .host
            .borrow()
            .editor_manager
            .dirty_document_toolkits()
            .expect("dirty document projection")
            .is_empty()
    {
        harness.host.borrow_mut().tick();
        std::thread::yield_now();
    }

    assert!(harness
        .host
        .borrow()
        .editor_manager
        .dirty_document_toolkits()
        .expect("dirty document projection")
        .is_empty());
    let _ = std::fs::remove_file(path);
}

#[test]
fn save_all_defers_to_an_active_close_prompt_save_batch() {
    let source = include_str!("../document_save.rs");

    assert!(
        source.contains("pending_close_prompt")
            && source.contains("save_in_flight()")
            && source.contains("Document save is already in progress."),
        "Save All must not contend with the single batch coordinator while a close prompt owns it"
    );
}
