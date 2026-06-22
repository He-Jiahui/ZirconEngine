use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::TemplatePaneNodeData;
use super::super::canvas::UiAssetPaletteDragData;
use super::super::inspector::UiAssetInspectorPanelData;
use super::super::preview::UiAssetPreviewPanelData;
use super::super::source::UiAssetSourcePanelData;
use super::super::style::UiAssetStylePanelData;
use super::actions::UiAssetActionStateData;
use super::collections::UiAssetCollectionPanelData;
use super::header::UiAssetPaneHeaderData;
use super::runtime_report::UiAssetRuntimeReportData;
use super::tools::UiAssetDesignerToolStateData;

#[derive(Clone, Default)]
pub(crate) struct UiAssetEditorPaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
    pub center_column_node: TemplatePaneNodeData,
    pub designer_panel_node: TemplatePaneNodeData,
    pub designer_canvas_panel_node: TemplatePaneNodeData,
    pub inspector_panel_node: TemplatePaneNodeData,
    pub stylesheet_panel_node: TemplatePaneNodeData,
    pub header: UiAssetPaneHeaderData,
    pub actions: UiAssetActionStateData,
    pub collections: UiAssetCollectionPanelData,
    pub source: UiAssetSourcePanelData,
    pub preview: UiAssetPreviewPanelData,
    pub runtime_report: UiAssetRuntimeReportData,
    pub designer_tools: UiAssetDesignerToolStateData,
    pub palette_drag: UiAssetPaletteDragData,
    pub style: UiAssetStylePanelData,
    pub inspector: UiAssetInspectorPanelData,
}
