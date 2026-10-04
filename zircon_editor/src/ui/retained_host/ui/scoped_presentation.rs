#[cfg(test)]
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::ui::asset_editor::UiAssetEditorPanePresentation;
#[cfg(test)]
use crate::ui::layouts::common::model_rc;
#[cfg(test)]
use crate::ui::retained_host::host_contract::FloatingWindowData;
use crate::ui::retained_host::host_contract::{
    rebuild_pane_template_hit_artifacts, FrameRect, HostPanePresentationLocation,
    HostPanePresentationPatch, HostWindowPresentationData, PaneData, UiAssetEditorPaneData,
    UiHostWindow,
};
#[cfg(test)]
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::layout::MainPageId;

use super::floating_pane_geometry::floating_pane_content_frame;
use super::pane_data_conversion::to_host_contract_ui_asset_pane;

#[derive(Clone, Default)]
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
    let mut fallback = UiAssetPresentationPatch::default();
    let committed = ui.patch_host_presentation_panes(|presentation| {
        let (transaction, result) =
            prepare_ui_asset_presentation_transaction(presentation, instance_id, ui_asset);
        fallback = result.clone();
        (!transaction.is_empty()).then_some((transaction, result))
    });
    committed.unwrap_or_else(|| {
        fallback.matched_presentation = false;
        fallback
    })
}

fn prepare_ui_asset_presentation_transaction(
    presentation: &HostWindowPresentationData,
    instance_id: &str,
    ui_asset: &UiAssetEditorPaneData,
) -> (HostPanePresentationPatch, UiAssetPresentationPatch) {
    let scene = &presentation.host_scene_data;
    let mut transaction = HostPanePresentationPatch::new();
    let mut result = UiAssetPresentationPatch::default();
    for leaf in &scene.document_leaves {
        push_ui_asset_pane_transaction(
            &mut transaction,
            HostPanePresentationLocation::DocumentLeaf {
                surface_key: leaf.surface_key.clone(),
            },
            &leaf.pane,
            &leaf.content_frame,
            instance_id,
            ui_asset,
            &mut result.damage,
        );
    }

    push_ui_asset_pane_transaction(
        &mut transaction,
        HostPanePresentationLocation::LeftDock,
        &scene.left_dock.pane,
        &scene.left_dock.content_frame,
        instance_id,
        ui_asset,
        &mut result.damage,
    );
    push_ui_asset_pane_transaction(
        &mut transaction,
        HostPanePresentationLocation::DocumentDock,
        &scene.document_dock.pane,
        &scene.document_dock.content_frame,
        instance_id,
        ui_asset,
        &mut result.damage,
    );
    push_ui_asset_pane_transaction(
        &mut transaction,
        HostPanePresentationLocation::RightDock,
        &scene.right_dock.pane,
        &scene.right_dock.content_frame,
        instance_id,
        ui_asset,
        &mut result.damage,
    );
    push_ui_asset_pane_transaction(
        &mut transaction,
        HostPanePresentationLocation::BottomDock,
        &scene.bottom_dock.pane,
        &scene.bottom_dock.content_frame,
        instance_id,
        ui_asset,
        &mut result.damage,
    );

    append_floating_ui_asset_transactions(
        &mut transaction,
        &mut result,
        presentation,
        instance_id,
        ui_asset,
    );
    result.matched_presentation = !transaction.is_empty();
    (transaction, result)
}

fn append_floating_ui_asset_transactions(
    transaction: &mut HostPanePresentationPatch,
    result: &mut UiAssetPresentationPatch,
    presentation: &HostWindowPresentationData,
    instance_id: &str,
    ui_asset: &UiAssetEditorPaneData,
) {
    let floating = &presentation.host_scene_data.floating_layer;
    result.floating_window_rows_visited += floating.floating_windows.row_count();
    for (row, window) in floating.floating_windows.iter().enumerate() {
        let content_frame = floating_pane_content_frame(
            &window.frame,
            &window.header_frame,
            floating.header_height_px,
        );
        if push_ui_asset_pane_transaction(
            transaction,
            HostPanePresentationLocation::Floating {
                row,
                window_id: window.window_id.clone(),
            },
            &window.active_pane,
            &content_frame,
            instance_id,
            ui_asset,
            &mut result.damage,
        ) {
            result.floating_window_rows_cloned += 1;
        }
    }

    let native = &presentation.native_floating_surface_data;
    result.floating_window_rows_visited += native.floating_windows.row_count();
    for (row, window) in native.floating_windows.iter().enumerate() {
        let content_frame = floating_pane_content_frame(
            &window.frame,
            &window.header_frame,
            native.header_height_px,
        );
        if push_ui_asset_pane_transaction(
            transaction,
            HostPanePresentationLocation::NativeFloating {
                row,
                window_id: window.window_id.clone(),
            },
            &window.active_pane,
            &content_frame,
            instance_id,
            ui_asset,
            &mut result.damage,
        ) {
            result.floating_window_rows_cloned += 1;
            result
                .expected_native_presenter_ids
                .insert(MainPageId::new(window.window_id.as_str()));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn push_ui_asset_pane_transaction(
    transaction: &mut HostPanePresentationPatch,
    location: HostPanePresentationLocation,
    pane: &PaneData,
    content_frame: &FrameRect,
    instance_id: &str,
    ui_asset: &UiAssetEditorPaneData,
    damage: &mut Vec<FrameRect>,
) -> bool {
    let Some(next) = next_ui_asset_pane(pane, content_frame, instance_id, ui_asset) else {
        return false;
    };
    transaction.push(location, pane, next);
    damage.push(content_frame.clone());
    true
}

fn next_ui_asset_pane(
    pane: &PaneData,
    content_frame: &FrameRect,
    instance_id: &str,
    ui_asset: &UiAssetEditorPaneData,
) -> Option<PaneData> {
    if !pane_is_ui_asset_instance(pane, instance_id) {
        return None;
    }
    let mut next = pane.clone();
    next.ui_asset = ui_asset.clone();
    rebuild_pane_template_hit_artifacts(
        &mut next,
        zircon_runtime_interface::ui::layout::UiSize::new(
            content_frame.width.max(1.0),
            content_frame.height.max(1.0),
        ),
    );
    Some(next)
}

#[cfg(test)]
#[derive(Default)]
struct NativePresenterLookup {
    presenter_ids: BTreeSet<MainPageId>,
    floating_window_rows_visited: usize,
}

#[cfg(test)]
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

#[cfg(test)]
#[derive(Default)]
struct PresentationProbe {
    matches: bool,
    floating_window_rows_visited: usize,
}

#[cfg(test)]
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

#[cfg(test)]
#[derive(Default)]
struct PanePresentationPatch {
    damage: Vec<FrameRect>,
    floating_window_rows_visited: usize,
    floating_window_rows_cloned: usize,
}

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
fn patch_floating_windows(
    windows: &mut ModelRc<FloatingWindowData>,
    header_height_px: f32,
    instance_id: &str,
    ui_asset: &UiAssetEditorPaneData,
    patch: &mut PanePresentationPatch,
) {
    patch.floating_window_rows_visited += windows.row_count();
    let mut row_patches = BTreeMap::new();
    for (row, window) in windows.iter().enumerate() {
        if !pane_is_ui_asset_instance(&window.active_pane, instance_id) {
            continue;
        }
        let mut window = window.clone();
        patch.floating_window_rows_cloned += 1;
        let content_frame =
            floating_pane_content_frame(&window.frame, &window.header_frame, header_height_px);
        if patch_ui_asset_pane(
            &mut window.active_pane,
            &content_frame,
            instance_id,
            ui_asset,
        ) {
            patch.damage.push(content_frame);
            row_patches.insert(row, window);
        }
    }
    if !row_patches.is_empty() {
        *windows = windows.with_row_patches(row_patches);
    }
}

#[cfg(test)]
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
    rebuild_pane_template_hit_artifacts(
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
#[path = "tests/scoped_presentation.rs"]
mod tests;
