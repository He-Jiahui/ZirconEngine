use crate::core::commands::{CommandEvalCtx, EditorCommandRegistry};
use crate::ui::binding::{
    EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind, WelcomeCommand,
};

use super::{normalize_editor_event_binding, EditorEventNormalizationError};

#[test]
fn normalization_preserves_unsupported_binding_as_a_typed_error() {
    let binding = EditorUiBinding::new(
        "WelcomeView",
        "CreateProjectButton",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::welcome_command(WelcomeCommand::CreateProject),
    );

    let error = normalize_editor_event_binding(
        &binding,
        &EditorCommandRegistry::default(),
        &CommandEvalCtx::default(),
    )
    .expect_err("welcome payload must not normalize as an editor event");

    assert!(matches!(
        error,
        EditorEventNormalizationError::UnsupportedBinding { ref native_binding }
            if native_binding.contains("WelcomeCommand.CreateProject")
    ));
}

#[test]
fn normalization_preserves_asset_relocation_identity() {
    let binding = EditorUiBinding::new(
        "AssetTree",
        "RelocateAsset",
        EditorUiEventKind::Drop,
        EditorUiBindingPayload::asset_command(crate::ui::binding::AssetCommand::RelocateAsset {
            asset_uuid: "00112233-4455-6677-8899-aabbccddeeff".to_owned(),
            target_locator: "res://environment/cube.zmodel".to_owned(),
        }),
    );

    let event = normalize_editor_event_binding(
        &binding,
        &EditorCommandRegistry::default(),
        &CommandEvalCtx::default(),
    )
    .unwrap();

    assert!(matches!(
        event,
        crate::core::editor_event::EditorEvent::Asset(
            crate::core::editor_event::EditorAssetEvent::RelocateAsset {
                asset_uuid,
                target_locator,
            }
        ) if asset_uuid == "00112233-4455-6677-8899-aabbccddeeff"
            && target_locator == "res://environment/cube.zmodel"
    ));
}

#[test]
fn u09_folder_asset_rename_reimport_normalization_preserves_context_identity() {
    use crate::core::editor_event::{EditorAssetEvent, EditorEvent};
    use crate::ui::binding::AssetCommand;

    let uuid = "00112233-4455-6677-8899-aabbccddeeff".to_owned();
    for (command, expected) in [
        (
            AssetCommand::CreateFolder {
                parent_folder_id: "res://levels".to_owned(),
            },
            EditorAssetEvent::CreateFolder {
                parent_folder_id: "res://levels".to_owned(),
            },
        ),
        (
            AssetCommand::RenameFolder {
                folder_id: "res://levels/old".to_owned(),
            },
            EditorAssetEvent::RenameFolder {
                folder_id: "res://levels/old".to_owned(),
            },
        ),
        (
            AssetCommand::DeleteFolder {
                folder_id: "res://levels/empty".to_owned(),
            },
            EditorAssetEvent::DeleteFolder {
                folder_id: "res://levels/empty".to_owned(),
            },
        ),
        (
            AssetCommand::RenameAsset {
                asset_uuid: uuid.clone(),
            },
            EditorAssetEvent::RenameAsset {
                asset_uuid: uuid.clone(),
            },
        ),
        (
            AssetCommand::ReimportAsset {
                asset_uuid: uuid.clone(),
            },
            EditorAssetEvent::ReimportAsset {
                asset_uuid: uuid.clone(),
            },
        ),
    ] {
        let binding = EditorUiBinding::new(
            "AssetContextMenu",
            "Mutation",
            EditorUiEventKind::Click,
            EditorUiBindingPayload::asset_command(command),
        );
        assert_eq!(
            normalize_editor_event_binding(
                &binding,
                &EditorCommandRegistry::default(),
                &CommandEvalCtx::default()
            )
            .unwrap(),
            EditorEvent::Asset(expected)
        );
    }
}
