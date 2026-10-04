use std::collections::BTreeMap;

use crate::ui::asset_editor::{
    apply_external_effects_to_asset_sources, UiAssetEditorExternalEffect, UiAssetEditorMode,
    UiAssetEditorRoute, UiAssetEditorSession,
};
use zircon_runtime_interface::ui::{layout::UiSize, template::UiAssetKind};

use super::fixtures::LOCAL_THEME_LAYOUT_ASSET_TOML;

#[test]
fn ui_asset_editor_session_undo_and_redo_replay_return_applied_external_effects() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/replay_theme.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        LOCAL_THEME_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("replay theme session");

    let promoted_style = session
        .promote_local_theme_to_external_style_asset(
            "res://ui/themes/replay_theme.zui",
            "ui.theme.replay_theme",
            "Replay Theme",
        )
        .expect("promote local theme")
        .expect("promoted style document");
    let promoted_style_source =
        crate::ui::asset_editor::serialize_authoring_document_as_v2(&promoted_style)
            .expect("serialize promoted style document as v2");

    let undone = session.undo_replay().expect("undo replay");
    assert!(undone.changed);
    assert_eq!(undone.label, "Promote Local Theme");
    assert_eq!(
        undone.external_effects,
        vec![UiAssetEditorExternalEffect::RemoveAssetSource {
            asset_id: "res://ui/themes/replay_theme.zui".to_string(),
        }]
    );

    let redone = session.redo_replay().expect("redo replay");
    assert!(redone.changed);
    assert_eq!(redone.label, "Promote Local Theme");
    assert_eq!(
        redone.external_effects,
        vec![UiAssetEditorExternalEffect::UpsertAssetSource {
            asset_id: "res://ui/themes/replay_theme.zui".to_string(),
            source: promoted_style_source,
        }]
    );
}

#[test]
fn ui_asset_editor_external_effects_apply_to_asset_source_maps_in_order() {
    let mut asset_sources: BTreeMap<String, String> = [
        (
            "res://ui/theme/editor_base.zui".to_string(),
            "[asset]\nid = \"ui.theme.editor_base\"\n".to_string(),
        ),
        (
            "res://ui/theme/editor_local.zui".to_string(),
            "[asset]\nid = \"ui.theme.editor_local\"\n".to_string(),
        ),
    ]
    .into_iter()
    .collect();

    assert!(apply_external_effects_to_asset_sources(
        &mut asset_sources,
        &[
            UiAssetEditorExternalEffect::RemoveAssetSource {
                asset_id: "res://ui/theme/editor_base.zui".to_string(),
            },
            UiAssetEditorExternalEffect::UpsertAssetSource {
                asset_id: "res://ui/theme/editor_theme_clone.zui".to_string(),
                source: "[asset]\nid = \"ui.theme.editor_theme_clone\"\n".to_string(),
            },
        ],
    ));
    assert!(!asset_sources.contains_key("res://ui/theme/editor_base.zui"));
    assert_eq!(
        asset_sources.get("res://ui/theme/editor_theme_clone.zui"),
        Some(&"[asset]\nid = \"ui.theme.editor_theme_clone\"\n".to_string())
    );

    assert!(!apply_external_effects_to_asset_sources(
        &mut asset_sources,
        &[UiAssetEditorExternalEffect::UpsertAssetSource {
            asset_id: "res://ui/theme/editor_theme_clone.zui".to_string(),
            source: "[asset]\nid = \"ui.theme.editor_theme_clone\"\n".to_string(),
        }],
    ));
}

#[test]
fn ui_asset_editor_external_effects_restore_previous_asset_source_when_replaying_overwrite() {
    let asset_id = "res://ui/theme/editor_base.zui".to_string();
    let previous_source = "[asset]\nid = \"ui.theme.editor_base\"\n".to_string();
    let overwritten_source = "[asset]\nid = \"ui.theme.editor_base.updated\"\n".to_string();
    let mut asset_sources: BTreeMap<String, String> = [(asset_id.clone(), previous_source.clone())]
        .into_iter()
        .collect();

    assert!(apply_external_effects_to_asset_sources(
        &mut asset_sources,
        &[UiAssetEditorExternalEffect::UpsertAssetSource {
            asset_id: asset_id.clone(),
            source: overwritten_source.clone(),
        }],
    ));
    assert_eq!(asset_sources.get(&asset_id), Some(&overwritten_source));

    assert!(apply_external_effects_to_asset_sources(
        &mut asset_sources,
        &[UiAssetEditorExternalEffect::RestoreAssetSource {
            asset_id: asset_id.clone(),
            source: previous_source.clone(),
        }],
    ));
    assert_eq!(asset_sources.get(&asset_id), Some(&previous_source));

    assert!(!apply_external_effects_to_asset_sources(
        &mut asset_sources,
        &[UiAssetEditorExternalEffect::RestoreAssetSource {
            asset_id,
            source: previous_source,
        }],
    ));
}

#[test]
fn ui_asset_editor_session_replay_effects_can_rebuild_cross_file_asset_sources() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/replay_theme.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        LOCAL_THEME_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("replay theme session");

    let promoted_style = session
        .promote_local_theme_to_external_style_asset(
            "res://ui/themes/replay_theme.zui",
            "ui.theme.replay_theme",
            "Replay Theme",
        )
        .expect("promote local theme")
        .expect("promoted style document");
    let promoted_style_source =
        crate::ui::asset_editor::serialize_authoring_document_as_v2(&promoted_style)
            .expect("serialize promoted style document as v2");
    let mut asset_sources: BTreeMap<String, String> = [(
        "res://ui/themes/replay_theme.zui".to_string(),
        promoted_style_source.clone(),
    )]
    .into_iter()
    .collect();

    let undone = session.undo_replay().expect("undo replay");
    assert!(apply_external_effects_to_asset_sources(
        &mut asset_sources,
        &undone.external_effects,
    ));
    assert!(!asset_sources.contains_key("res://ui/themes/replay_theme.zui"));

    let redone = session.redo_replay().expect("redo replay");
    assert!(apply_external_effects_to_asset_sources(
        &mut asset_sources,
        &redone.external_effects,
    ));
    assert_eq!(
        asset_sources.get("res://ui/themes/replay_theme.zui"),
        Some(&promoted_style_source)
    );
}
