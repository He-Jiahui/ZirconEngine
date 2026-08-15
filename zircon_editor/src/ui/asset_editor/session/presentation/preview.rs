use crate::ui::asset_editor::presentation::{
    UiAssetEditorPreviewCanvasNode, UiAssetEditorPreviewCanvasSlotTarget,
};
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

use super::super::{
    palette::PaletteInsertMode,
    palette_drop::build_palette_drag_slot_target_overlays,
    palette_target_chooser::UiAssetPaletteTargetChooser,
    preview_host::UiAssetPreviewHost,
    preview_projection::{build_preview_projection, UiAssetPreviewProjection},
    ui_asset_editor_session::UiAssetEditorSession,
};

pub(super) struct UiAssetPreviewPaneData {
    pub(super) summary: String,
    pub(super) items: Vec<String>,
    pub(super) canvas_items: Vec<UiAssetEditorPreviewCanvasNode>,
    pub(super) selected_index: i32,
    pub(super) surface_width: f32,
    pub(super) surface_height: f32,
    pub(super) palette_drag_target_preview_index: i32,
    pub(super) palette_drag_target_action: String,
    pub(super) palette_drag_target_label: String,
    pub(super) palette_drag_slot_target_items: Vec<UiAssetEditorPreviewCanvasSlotTarget>,
    pub(super) palette_drag_candidate_items: Vec<String>,
    pub(super) palette_drag_candidate_selected_index: i32,
    pub(super) palette_target_chooser_active: bool,
}

impl UiAssetEditorSession {
    pub(super) fn preview_pane_presentation(&self) -> UiAssetPreviewPaneData {
        zircon_runtime::profile_scope!("editor", "asset_editor.presentation", "preview",);
        let summary = preview_summary(self.preview_host.as_ref());
        let projection = build_preview_projection(
            &self.last_valid_document,
            self.preview_host.as_ref(),
            &self.selection,
        );
        let selected_palette_drag_target = self.selected_palette_drag_target();
        let palette_drag_slot_target_items = selected_palette_drag_target
            .map(|drag_target| {
                build_palette_drag_slot_target_overlays(
                    &self.last_valid_document,
                    drag_target,
                    self.palette_catalog.reference_imports(),
                    &projection,
                )
                .into_iter()
                .map(|item| UiAssetEditorPreviewCanvasSlotTarget {
                    label: item.label,
                    detail: item.detail,
                    x: item.x,
                    y: item.y,
                    width: item.width,
                    height: item.height,
                    selected: item.selected,
                })
                .collect()
            })
            .unwrap_or_default();
        let palette_drag_candidate_items = self
            .palette_target_chooser
            .as_ref()
            .map(|chooser| {
                chooser
                    .resolution()
                    .candidates
                    .iter()
                    .map(|candidate| {
                        if candidate.detail.is_empty() {
                            candidate.key.clone()
                        } else {
                            format!("{} • {}", candidate.key, candidate.detail)
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();
        record_current_ui_perf_counter(UiPerfCounter::AssetEditorPanePreviewBuildCount, 1.0);
        let UiAssetPreviewProjection {
            items,
            canvas_nodes,
            selected_index,
            surface_width,
            surface_height,
        } = projection;
        UiAssetPreviewPaneData {
            summary,
            items,
            canvas_items: canvas_nodes
                .into_iter()
                .map(|item| UiAssetEditorPreviewCanvasNode {
                    node_id: item.node_id,
                    label: item.label,
                    kind: item.kind,
                    x: item.x,
                    y: item.y,
                    width: item.width,
                    height: item.height,
                    depth: item.depth,
                    z_index: item.z_index,
                    selected: item.selected,
                })
                .collect(),
            selected_index,
            surface_width,
            surface_height,
            palette_drag_target_preview_index: selected_palette_drag_target
                .and_then(|target| target.preview_index.map(|index| index as i32))
                .unwrap_or(-1),
            palette_drag_target_action: selected_palette_drag_target
                .map(|target| palette_insert_mode_action(target.plan.mode).to_string())
                .unwrap_or_default(),
            palette_drag_target_label: selected_palette_drag_target
                .map(|target| target.plan.label.clone())
                .unwrap_or_default(),
            palette_drag_slot_target_items,
            palette_drag_candidate_items,
            palette_drag_candidate_selected_index: self
                .palette_target_chooser
                .as_ref()
                .map(|chooser| chooser.resolution().selected_index as i32)
                .unwrap_or(-1),
            palette_target_chooser_active: self
                .palette_target_chooser
                .as_ref()
                .map(UiAssetPaletteTargetChooser::sticky)
                .unwrap_or(false),
        }
    }
}

fn palette_insert_mode_action(mode: PaletteInsertMode) -> &'static str {
    match mode {
        PaletteInsertMode::Child => "palette.insert.child",
        PaletteInsertMode::After => "palette.insert.after",
    }
}

fn preview_summary(preview_host: Option<&UiAssetPreviewHost>) -> String {
    let Some(preview_host) = preview_host else {
        return "preview unavailable".to_string();
    };
    format!(
        "{} rendered nodes @ {:.0}x{:.0}",
        preview_host.surface().render_extract.list.commands.len(),
        preview_host.preview_size().width,
        preview_host.preview_size().height
    )
}
