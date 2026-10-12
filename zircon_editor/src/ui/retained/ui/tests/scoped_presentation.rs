use std::collections::BTreeSet;

use crate::ui::asset_editor::UiAssetEditorPanePresentation;
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::host_contract::{
    FloatingWindowData, FrameRect, HostWindowPresentationData, PaneData,
};
use crate::ui::workbench::layout::MainPageId;

use super::{
    build_ui_asset_presentation_patch, native_presenter_ids, patch_ui_asset_pane_in_presentation,
    presentation_contains_ui_asset_pane, to_host_contract_ui_asset_pane,
};

#[test]
fn scoped_patch_builds_the_host_pane_once_before_patching_presentations() {
    let presentation = UiAssetEditorPanePresentation {
        asset_id: "res://ui/once.zui".into(),
        ..UiAssetEditorPanePresentation::default()
    };

    let pane = build_ui_asset_presentation_patch(presentation, "ui-asset-editor#once");

    assert_eq!(pane.header.asset_id, "res://ui/once.zui");
}

#[test]
fn ui_asset_patch_changes_only_the_matching_presented_pane() {
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.document_dock.pane = PaneData {
        id: "ui-asset-editor#first".into(),
        kind: "UiAssetEditor".into(),
        ..PaneData::default()
    };
    presentation.host_scene_data.document_dock.content_frame = FrameRect {
        x: 100.0,
        y: 80.0,
        width: 640.0,
        height: 480.0,
    };
    presentation.host_scene_data.left_dock.pane = PaneData {
        id: "ui-asset-editor#second".into(),
        kind: "UiAssetEditor".into(),
        ..PaneData::default()
    };
    presentation.host_scene_data.left_dock.content_frame = FrameRect {
        x: 0.0,
        y: 80.0,
        width: 240.0,
        height: 480.0,
    };
    let ui_asset = to_host_contract_ui_asset_pane(
        UiAssetEditorPanePresentation {
            asset_id: "res://ui/first.zui".into(),
            ..UiAssetEditorPanePresentation::default()
        },
        "ui-asset-editor#first",
    );

    let patch =
        patch_ui_asset_pane_in_presentation(&mut presentation, "ui-asset-editor#first", &ui_asset);

    assert_eq!(patch.damage.len(), 1);
    let patched_pane = &presentation.host_scene_data.document_dock.pane;
    assert!(
        patched_pane
            .body_template_hit_index
            .as_deref()
            .expect("scoped pane refresh must publish its popup hit index")
            .indexes_nodes(&patched_pane.ui_asset.nodes),
        "scoped pane hit index must bind the newly published nodes"
    );
    assert_eq!(
        presentation
            .host_scene_data
            .document_dock
            .pane
            .ui_asset
            .header
            .asset_id,
        "res://ui/first.zui"
    );
    assert!(presentation
        .host_scene_data
        .left_dock
        .pane
        .ui_asset
        .header
        .asset_id
        .is_empty());
}

#[test]
fn floating_scoped_patch_uses_the_same_per_window_content_geometry_as_full_conversion() {
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.floating_layer.header_height_px = 28.0;
    presentation.host_scene_data.floating_layer.floating_windows =
        model_rc(vec![FloatingWindowData {
            frame: FrameRect {
                x: 40.0,
                y: 60.0,
                width: 640.0,
                height: 480.0,
            },
            header_frame: FrameRect {
                x: 40.0,
                y: 60.0,
                width: 640.0,
                height: 46.0,
            },
            active_pane: PaneData {
                id: "ui-asset-editor#floating".into(),
                kind: "UiAssetEditor".into(),
                ..PaneData::default()
            },
            ..FloatingWindowData::default()
        }]);
    let ui_asset = to_host_contract_ui_asset_pane(
        UiAssetEditorPanePresentation::default(),
        "ui-asset-editor#floating",
    );

    let patch = patch_ui_asset_pane_in_presentation(
        &mut presentation,
        "ui-asset-editor#floating",
        &ui_asset,
    );

    assert_eq!(
        patch.damage,
        vec![FrameRect {
            x: 40.0,
            y: 106.0,
            width: 640.0,
            height: 433.0,
        }]
    );
    assert_eq!(patch.floating_window_rows_visited, 1);
    assert_eq!(patch.floating_window_rows_cloned, 1);
}

#[test]
fn floating_patch_does_not_clone_rows_when_the_instance_is_absent() {
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.floating_layer.floating_windows = model_rc(vec![
        FloatingWindowData::default(),
        FloatingWindowData::default(),
    ]);
    let previous = presentation
        .host_scene_data
        .floating_layer
        .floating_windows
        .clone();
    let ui_asset = to_host_contract_ui_asset_pane(
        UiAssetEditorPanePresentation::default(),
        "ui-asset-editor#absent",
    );

    let patch =
        patch_ui_asset_pane_in_presentation(&mut presentation, "ui-asset-editor#absent", &ui_asset);

    assert!(patch.damage.is_empty());
    assert_eq!(patch.floating_window_rows_visited, 2);
    assert_eq!(patch.floating_window_rows_cloned, 0);
    assert!(
        previous.shares_values_with(&presentation.host_scene_data.floating_layer.floating_windows)
    );
}

#[test]
fn floating_patch_clones_only_the_matching_row_and_reuses_other_storage() {
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.floating_layer.floating_windows = model_rc(vec![
        FloatingWindowData {
            active_pane: PaneData {
                id: "ui-asset-editor#target".into(),
                kind: "UiAssetEditor".into(),
                ..PaneData::default()
            },
            ..FloatingWindowData::default()
        },
        FloatingWindowData {
            active_pane: PaneData {
                id: "ui-asset-editor#other".into(),
                kind: "UiAssetEditor".into(),
                ..PaneData::default()
            },
            ..FloatingWindowData::default()
        },
    ]);
    let previous = presentation
        .host_scene_data
        .floating_layer
        .floating_windows
        .clone();
    let ui_asset = to_host_contract_ui_asset_pane(
        UiAssetEditorPanePresentation::default(),
        "ui-asset-editor#target",
    );

    let patch =
        patch_ui_asset_pane_in_presentation(&mut presentation, "ui-asset-editor#target", &ui_asset);
    let current = &presentation.host_scene_data.floating_layer.floating_windows;

    assert_eq!(patch.floating_window_rows_visited, 2);
    assert_eq!(patch.floating_window_rows_cloned, 1);
    assert!(!previous.shares_row_with(current, 0));
    assert!(previous.shares_row_with(current, 1));
}

#[test]
fn native_presenter_expectation_keeps_the_matching_window_identity() {
    let mut presentation = HostWindowPresentationData::default();
    presentation.native_floating_surface_data.floating_windows = model_rc(vec![
        FloatingWindowData {
            window_id: "window:target".into(),
            active_pane: PaneData {
                id: "ui-asset-editor#target".into(),
                kind: "UiAssetEditor".into(),
                ..PaneData::default()
            },
            ..FloatingWindowData::default()
        },
        FloatingWindowData {
            window_id: "window:other".into(),
            active_pane: PaneData {
                id: "ui-asset-editor#other".into(),
                kind: "UiAssetEditor".into(),
                ..PaneData::default()
            },
            ..FloatingWindowData::default()
        },
    ]);

    let lookup = native_presenter_ids(&presentation, "ui-asset-editor#target");

    assert_eq!(
        lookup.presenter_ids,
        BTreeSet::from([MainPageId::new("window:target")])
    );
    assert_eq!(lookup.floating_window_rows_visited, 2);
}

#[test]
fn missing_presentation_probe_counts_all_rows_scanned_before_the_fallback() {
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.floating_layer.floating_windows = model_rc(vec![
        FloatingWindowData::default(),
        FloatingWindowData::default(),
    ]);
    presentation.native_floating_surface_data.floating_windows = model_rc(vec![
        FloatingWindowData::default(),
        FloatingWindowData::default(),
    ]);

    let probe = presentation_contains_ui_asset_pane(&presentation, "ui-asset-editor#missing");

    assert!(!probe.matches);
    assert_eq!(probe.floating_window_rows_visited, 4);
}
