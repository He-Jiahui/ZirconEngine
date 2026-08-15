use std::cell::Cell;
use std::collections::BTreeSet;

use crate::ui::asset_editor::UiAssetEditorPanePresentation;
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::host_contract::{
    build_pane_template_surface_frame, FloatingWindowData, FrameRect, HostWindowPresentationData,
    PaneData, UiAssetEditorPaneData, UiHostWindow,
};
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::layout::MainPageId;

use super::floating_pane_geometry::floating_pane_content_frame;
use super::pane_data_conversion::to_host_contract_ui_asset_pane;

#[derive(Default)]
pub(crate) struct UiAssetPresentationPatch {
    pub(crate) matched_presentation: bool,
    pub(crate) damage: Vec<FrameRect>,
    pub(crate) expected_native_presenter_ids: BTreeSet<MainPageId>,
    pub(crate) floating_window_rows_visited: usize,
    pub(crate) floating_window_rows_cloned: usize,
}

pub(crate) fn build_ui_asset_presentation_patch(
    pane_presentation: UiAssetEditorPanePresentation,
    instance_id: &str,
) -> UiAssetEditorPaneData {
    to_host_contract_ui_asset_pane(pane_presentation, instance_id)
}

pub(crate) fn patch_ui_asset_presentation(
    ui: &UiHostWindow,
    instance_id: &str,
    ui_asset: &UiAssetEditorPaneData,
) -> UiAssetPresentationPatch {
    let predicate_rows_visited = Cell::new(0usize);
    ui.update_host_presentation_if(
        |presentation| {
            let probe = presentation_contains_ui_asset_pane(presentation, instance_id);
            predicate_rows_visited.set(probe.floating_window_rows_visited);
            probe.matches
        },
        |presentation| {
            let native_presenters = native_presenter_ids(presentation, instance_id);
            let pane_patch =
                patch_ui_asset_pane_in_presentation(presentation, instance_id, ui_asset);
            UiAssetPresentationPatch {
                matched_presentation: true,
                expected_native_presenter_ids: native_presenters.presenter_ids,
                damage: pane_patch.damage,
                floating_window_rows_visited: predicate_rows_visited.get()
                    + native_presenters.floating_window_rows_visited
                    + pane_patch.floating_window_rows_visited,
                floating_window_rows_cloned: pane_patch.floating_window_rows_cloned,
            }
        },
    )
    .unwrap_or_else(|| UiAssetPresentationPatch {
        floating_window_rows_visited: predicate_rows_visited.get(),
        ..UiAssetPresentationPatch::default()
    })
}

#[derive(Default)]
struct NativePresenterLookup {
    presenter_ids: BTreeSet<MainPageId>,
    floating_window_rows_visited: usize,
}

fn native_presenter_ids(
    presentation: &HostWindowPresentationData,
    instance_id: &str,
) -> NativePresenterLookup {
    let mut lookup = NativePresenterLookup::default();
    for window in presentation
        .native_floating_surface_data
        .floating_windows
        .iter()
    {
        lookup.floating_window_rows_visited += 1;
        if pane_is_ui_asset_instance(&window.active_pane, instance_id) {
            lookup
                .presenter_ids
                .insert(MainPageId::new(window.window_id.as_str()));
        }
    }
    lookup
}

#[derive(Default)]
struct PresentationProbe {
    matches: bool,
    floating_window_rows_visited: usize,
}

fn presentation_contains_ui_asset_pane(
    presentation: &HostWindowPresentationData,
    instance_id: &str,
) -> PresentationProbe {
    let scene = &presentation.host_scene_data;
    if [
        &scene.left_dock.pane,
        &scene.document_dock.pane,
        &scene.right_dock.pane,
        &scene.bottom_dock.pane,
    ]
    .into_iter()
    .any(|pane| pane_is_ui_asset_instance(pane, instance_id))
    {
        return PresentationProbe {
            matches: true,
            ..PresentationProbe::default()
        };
    }

    let mut probe = PresentationProbe::default();
    for window in scene.floating_layer.floating_windows.iter() {
        probe.floating_window_rows_visited += 1;
        if pane_is_ui_asset_instance(&window.active_pane, instance_id) {
            probe.matches = true;
            return probe;
        }
    }
    for window in presentation
        .native_floating_surface_data
        .floating_windows
        .iter()
    {
        probe.floating_window_rows_visited += 1;
        if pane_is_ui_asset_instance(&window.active_pane, instance_id) {
            probe.matches = true;
            return probe;
        }
    }
    probe
}

#[derive(Default)]
struct PanePresentationPatch {
    damage: Vec<FrameRect>,
    floating_window_rows_visited: usize,
    floating_window_rows_cloned: usize,
}

fn patch_ui_asset_pane_in_presentation(
    presentation: &mut HostWindowPresentationData,
    instance_id: &str,
    ui_asset: &UiAssetEditorPaneData,
) -> PanePresentationPatch {
    let scene = &mut presentation.host_scene_data;
    let left_content_frame = scene.left_dock.content_frame.clone();
    let document_content_frame = scene.document_dock.content_frame.clone();
    let right_content_frame = scene.right_dock.content_frame.clone();
    let bottom_content_frame = scene.bottom_dock.content_frame.clone();
    let floating_header_height_px = scene.floating_layer.header_height_px;
    let native_floating_header_height_px =
        presentation.native_floating_surface_data.header_height_px;
    let mut patch = PanePresentationPatch::default();
    patch_dock_pane(
        &mut scene.left_dock.pane,
        &left_content_frame,
        instance_id,
        ui_asset,
        &mut patch.damage,
    );
    patch_dock_pane(
        &mut scene.document_dock.pane,
        &document_content_frame,
        instance_id,
        ui_asset,
        &mut patch.damage,
    );
    patch_dock_pane(
        &mut scene.right_dock.pane,
        &right_content_frame,
        instance_id,
        ui_asset,
        &mut patch.damage,
    );
    patch_dock_pane(
        &mut scene.bottom_dock.pane,
        &bottom_content_frame,
        instance_id,
        ui_asset,
        &mut patch.damage,
    );
    patch_floating_windows(
        &mut scene.floating_layer.floating_windows,
        floating_header_height_px,
        instance_id,
        ui_asset,
        &mut patch,
    );
    patch_floating_windows(
        &mut presentation.native_floating_surface_data.floating_windows,
        native_floating_header_height_px,
        instance_id,
        ui_asset,
        &mut patch,
    );
    patch
}

fn patch_dock_pane(
    pane: &mut PaneData,
    content_frame: &FrameRect,
    instance_id: &str,
    ui_asset: &UiAssetEditorPaneData,
    damage: &mut Vec<FrameRect>,
) {
    if patch_ui_asset_pane(pane, content_frame, instance_id, ui_asset) {
        damage.push(content_frame.clone());
    }
}

fn patch_floating_windows(
    windows: &mut ModelRc<FloatingWindowData>,
    header_height_px: f32,
    instance_id: &str,
    ui_asset: &UiAssetEditorPaneData,
    patch: &mut PanePresentationPatch,
) {
    let mut changed = false;
    let mut patched_windows = Vec::with_capacity(windows.row_count());
    patch.floating_window_rows_visited += windows.row_count();
    patch.floating_window_rows_cloned += windows.row_count();
    for window in windows.iter() {
        let mut window = window.clone();
        let content_frame =
            floating_pane_content_frame(&window.frame, &window.header_frame, header_height_px);
        if patch_ui_asset_pane(
            &mut window.active_pane,
            &content_frame,
            instance_id,
            ui_asset,
        ) {
            patch.damage.push(content_frame);
            changed = true;
        }
        patched_windows.push(window);
    }
    if changed {
        *windows = model_rc(patched_windows);
    }
}

fn patch_ui_asset_pane(
    pane: &mut PaneData,
    content_frame: &FrameRect,
    instance_id: &str,
    ui_asset: &UiAssetEditorPaneData,
) -> bool {
    if !pane_is_ui_asset_instance(pane, instance_id) {
        return false;
    }
    pane.ui_asset = ui_asset.clone();
    pane.body_surface_frame = build_pane_template_surface_frame(
        pane,
        zircon_runtime_interface::ui::layout::UiSize::new(
            content_frame.width.max(1.0),
            content_frame.height.max(1.0),
        ),
    );
    true
}

fn pane_is_ui_asset_instance(pane: &PaneData, instance_id: &str) -> bool {
    pane.kind.as_str() == "UiAssetEditor" && pane.id.as_str() == instance_id
}

#[cfg(test)]
mod tests {
    use crate::ui::asset_editor::UiAssetEditorPanePresentation;
    use crate::ui::layouts::common::model_rc;
    use crate::ui::retained_host::host_contract::{
        FloatingWindowData, FrameRect, HostWindowPresentationData, PaneData,
    };
    use crate::ui::workbench::layout::MainPageId;

    use super::{
        build_ui_asset_presentation_patch, native_presenter_ids,
        patch_ui_asset_pane_in_presentation, presentation_contains_ui_asset_pane,
        to_host_contract_ui_asset_pane,
    };

    #[test]
    fn scoped_patch_builds_the_host_pane_once_before_patching_presentations() {
        let presentation = UiAssetEditorPanePresentation {
            asset_id: "res://ui/once.zui".into(),
            ..UiAssetEditorPanePresentation::default()
        };

        let pane = build_ui_asset_presentation_patch(presentation, "ui-asset-editor#once");

        assert_eq!(pane.id.as_str(), "ui-asset-editor#once");
        assert_eq!(pane.ui_asset.header.asset_id, "res://ui/once.zui");
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

        let patch = patch_ui_asset_pane_in_presentation(
            &mut presentation,
            "ui-asset-editor#first",
            &ui_asset,
        );

        assert_eq!(patch.damage.len(), 1);
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
    fn floating_patch_counts_each_cloned_row_even_when_the_instance_is_absent() {
        let mut presentation = HostWindowPresentationData::default();
        presentation.host_scene_data.floating_layer.floating_windows = model_rc(vec![
            FloatingWindowData::default(),
            FloatingWindowData::default(),
        ]);
        let ui_asset = to_host_contract_ui_asset_pane(
            UiAssetEditorPanePresentation::default(),
            "ui-asset-editor#absent",
        );

        let patch = patch_ui_asset_pane_in_presentation(
            &mut presentation,
            "ui-asset-editor#absent",
            &ui_asset,
        );

        assert!(patch.damage.is_empty());
        assert_eq!(patch.floating_window_rows_visited, 2);
        assert_eq!(patch.floating_window_rows_cloned, 2);
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
}
