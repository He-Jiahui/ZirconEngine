use crate::core::asset::AssetWriteAccess;
use crate::core::commands::{
    CommandEvalCtx, EditorCommandDescriptor, EditorCommandRegistry, WhenClause,
};
use crate::core::editor_operation::EditorOperationPath;

fn descriptor(id: &str) -> EditorCommandDescriptor {
    EditorCommandDescriptor::operation(EditorOperationPath::parse(id).unwrap(), id)
}

#[test]
fn required_capabilities_are_metadata_and_effective_when_conjuncts() {
    let descriptor = descriptor("test.required.capabilities")
        .with_when(WhenClause::ProjectOpen)
        .with_required_capabilities(["editor.weather", "editor.weather"])
        .with_required_capabilities(["editor.clouds"]);

    assert_eq!(
        descriptor.required_capabilities(),
        &["editor.clouds".to_string(), "editor.weather".to_string()]
    );
    assert!(!descriptor.is_enabled(
        &CommandEvalCtx::interactive()
            .with_project_open(true)
            .with_capabilities(["editor.weather"])
    ));
    assert!(descriptor.is_enabled(
        &CommandEvalCtx::interactive()
            .with_project_open(true)
            .with_capabilities(["editor.clouds", "editor.weather"])
    ));
    assert!(!descriptor.is_enabled(&CommandEvalCtx::headless([
        "editor.clouds",
        "editor.weather",
    ])));
}

#[test]
fn menu_and_palette_use_the_same_descriptor_when_evaluation() {
    let registry = EditorCommandRegistry::new(vec![descriptor("test.selection.action")
        .with_menu_path("Tools/Selection Action")
        .with_when(WhenClause::SelectionNonEmpty)])
    .unwrap();
    let empty = CommandEvalCtx::interactive().with_selection_count(0);
    let selected = CommandEvalCtx::interactive().with_selection_count(1);

    let empty_menu = registry.menu_model("Tools", &empty).unwrap();
    let selected_menu = registry.menu_model("Tools", &selected).unwrap();
    assert!(!empty_menu.items[0].enabled);
    assert!(selected_menu.items[0].enabled);
    assert!(registry
        .command_palette_query_window(&empty, "", 0, 16)
        .is_empty());
    assert_eq!(
        registry
            .command_palette_query_window(&selected, "", 0, 16)
            .len(),
        1
    );
}

#[test]
fn descriptor_when_survives_serde_without_materializing_duplicate_capabilities() {
    let descriptor = descriptor("test.serde.capabilities")
        .with_when(WhenClause::Any(vec![
            WhenClause::UndoAvailable,
            WhenClause::RedoAvailable,
        ]))
        .with_required_capabilities(["editor.history"]);

    let first_effective = descriptor.effective_when();
    let second_effective = descriptor.effective_when();
    assert_eq!(first_effective, second_effective);
    let mut encoded = serde_json::to_value(&descriptor).unwrap();
    encoded["required_capabilities"] =
        serde_json::json!(["editor.history", "editor.history", "editor.timeline"]);
    let decoded: EditorCommandDescriptor = serde_json::from_value(encoded).unwrap();

    assert_eq!(decoded.when(), descriptor.when());
    assert_eq!(
        decoded.required_capabilities(),
        &["editor.history".to_string(), "editor.timeline".to_string()]
    );
    assert!(decoded.effective_when().eval(
        &CommandEvalCtx::interactive()
            .with_undo_available(true)
            .with_capabilities(["editor.history", "editor.timeline"])
    ));
}

#[test]
fn asset_write_target_adds_a_serializable_writable_when_guard() {
    let descriptor = descriptor("test.asset.mutate")
        .with_asset_write_target_arguments("asset_type", "asset_locator");

    assert_eq!(
        descriptor
            .asset_write_target()
            .unwrap()
            .asset_type_argument(),
        "asset_type"
    );
    assert_eq!(
        descriptor.asset_write_target().unwrap().locator_argument(),
        "asset_locator"
    );
    assert!(!descriptor.is_enabled(
        &CommandEvalCtx::interactive().with_asset_write_access(AssetWriteAccess::ReadOnly)
    ));
    assert!(descriptor.is_enabled(
        &CommandEvalCtx::interactive().with_asset_write_access(AssetWriteAccess::Writable)
    ));

    let encoded = serde_json::to_string(&descriptor).unwrap();
    let decoded: EditorCommandDescriptor = serde_json::from_str(&encoded).unwrap();
    assert_eq!(
        decoded.asset_write_target(),
        descriptor.asset_write_target()
    );
    assert!(matches!(
        decoded.effective_when(),
        WhenClause::AssetWritable
    ));
}
