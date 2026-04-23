use std::path::{Path, PathBuf};

use thiserror::Error;
use zircon_runtime::ui::template::{
    UiAssetError, UiAssetLoader, UiDocumentCompiler, UiTemplateBuildError, UiTemplateSurfaceBuilder,
};
use zircon_runtime::ui::{
    event_ui::{UiNodeId, UiTreeId},
    layout::{UiFrame, UiSize},
    surface::UiSurface,
    tree::UiTreeError,
};

use super::contract::{
    UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID,
    UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_HEADER_SHELL_REFERENCE,
    UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_SECTION_CARD_REFERENCE,
};

const UI_ASSET_EDITOR_LAYOUT_ASSET_PATH: &str = "/assets/ui/editor/ui_asset_editor.ui.toml";
const UI_ASSET_EDITOR_WIDGET_ASSET_PATH: &str = "/assets/ui/editor/editor_widgets.ui.toml";
const UI_ASSET_EDITOR_STYLE_ASSET_PATH: &str = "/assets/ui/theme/editor_base.ui.toml";

const HEADER_PANEL_CONTROL_ID: &str = "HeaderPanel";
const HEADER_ASSET_ROW_CONTROL_ID: &str = "HeaderAssetRow";
const HEADER_STATUS_ROW_CONTROL_ID: &str = "HeaderStatusRow";
const HEADER_ACTION_ROW_CONTROL_ID: &str = "HeaderActionRow";
const LEFT_COLUMN_CONTROL_ID: &str = "LeftColumn";
const CENTER_COLUMN_CONTROL_ID: &str = "CenterColumn";
const RIGHT_COLUMN_CONTROL_ID: &str = "RightColumn";
const PALETTE_PANEL_CONTROL_ID: &str = "PalettePanel";
const HIERARCHY_PANEL_CONTROL_ID: &str = "HierarchyPanel";
const DESIGNER_PANEL_CONTROL_ID: &str = "DesignerPanel";
const DESIGNER_CANVAS_PANEL_CONTROL_ID: &str = "DesignerCanvasPanel";
const RENDER_STACK_PANEL_CONTROL_ID: &str = "RenderStackPanel";
const ACTION_BAR_PANEL_CONTROL_ID: &str = "ActionBarPanel";
const ACTION_INSERT_ROW_CONTROL_ID: &str = "ActionInsertRow";
const ACTION_REPARENT_ROW_CONTROL_ID: &str = "ActionReparentRow";
const ACTION_STRUCTURE_ROW_CONTROL_ID: &str = "ActionStructureRow";
const SOURCE_PANEL_CONTROL_ID: &str = "SourcePanel";
const SOURCE_INFO_PANEL_CONTROL_ID: &str = "SourceInfoPanel";
const SOURCE_OUTLINE_PANEL_CONTROL_ID: &str = "SourceOutlinePanel";
const MOCK_WORKSPACE_PANEL_CONTROL_ID: &str = "MockWorkspacePanel";
const MOCK_SUBJECTS_PANEL_CONTROL_ID: &str = "MockSubjectsPanel";
const MOCK_EDITOR_PANEL_CONTROL_ID: &str = "MockEditorPanel";
const MOCK_STATE_GRAPH_PANEL_CONTROL_ID: &str = "MockStateGraphPanel";
const SOURCE_TEXT_PANEL_CONTROL_ID: &str = "SourceTextPanel";
const INSPECTOR_PANEL_CONTROL_ID: &str = "InspectorPanel";
const INSPECTOR_CONTENT_PANEL_CONTROL_ID: &str = "InspectorContentPanel";
const INSPECTOR_WIDGET_SECTION_CONTROL_ID: &str = "InspectorWidgetSection";
const INSPECTOR_PROMOTE_SECTION_CONTROL_ID: &str = "InspectorPromoteSection";
const INSPECTOR_SLOT_SECTION_CONTROL_ID: &str = "InspectorSlotSection";
const INSPECTOR_LAYOUT_SECTION_CONTROL_ID: &str = "InspectorLayoutSection";
const INSPECTOR_BINDING_SECTION_CONTROL_ID: &str = "InspectorBindingSection";
const STYLESHEET_PANEL_CONTROL_ID: &str = "StylesheetPanel";
const STYLESHEET_ACTION_ROW_CONTROL_ID: &str = "StylesheetActionRow";
const STYLESHEET_STATE_PRIMARY_ROW_CONTROL_ID: &str = "StylesheetStatePrimaryRow";
const STYLESHEET_STATE_SECONDARY_ROW_CONTROL_ID: &str = "StylesheetStateSecondaryRow";
const STYLESHEET_CONTENT_PANEL_CONTROL_ID: &str = "StylesheetContentPanel";
const STYLESHEET_THEME_SECTION_CONTROL_ID: &str = "StylesheetThemeSection";
const STYLESHEET_AUTHORING_SECTION_CONTROL_ID: &str = "StylesheetAuthoringSection";
const STYLESHEET_MATCHED_RULE_SECTION_CONTROL_ID: &str = "StylesheetMatchedRuleSection";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UiAssetEditorShellFrame {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl From<UiFrame> for UiAssetEditorShellFrame {
    fn from(frame: UiFrame) -> Self {
        Self {
            x: frame.x,
            y: frame.y,
            width: frame.width,
            height: frame.height,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UiAssetEditorShellLayout {
    pub header_panel: UiAssetEditorShellFrame,
    pub header_asset_row: UiAssetEditorShellFrame,
    pub header_status_row: UiAssetEditorShellFrame,
    pub header_action_row: UiAssetEditorShellFrame,
    pub left_column: UiAssetEditorShellFrame,
    pub center_column: UiAssetEditorShellFrame,
    pub right_column: UiAssetEditorShellFrame,
    pub palette_panel: UiAssetEditorShellFrame,
    pub hierarchy_panel: UiAssetEditorShellFrame,
    pub designer_panel: UiAssetEditorShellFrame,
    pub designer_canvas_panel: UiAssetEditorShellFrame,
    pub render_stack_panel: UiAssetEditorShellFrame,
    pub action_bar_panel: UiAssetEditorShellFrame,
    pub action_insert_row: UiAssetEditorShellFrame,
    pub action_reparent_row: UiAssetEditorShellFrame,
    pub action_structure_row: UiAssetEditorShellFrame,
    pub source_panel: UiAssetEditorShellFrame,
    pub source_info_panel: UiAssetEditorShellFrame,
    pub source_outline_panel: UiAssetEditorShellFrame,
    pub mock_workspace_panel: UiAssetEditorShellFrame,
    pub mock_subjects_panel: UiAssetEditorShellFrame,
    pub mock_editor_panel: UiAssetEditorShellFrame,
    pub mock_state_graph_panel: UiAssetEditorShellFrame,
    pub source_text_panel: UiAssetEditorShellFrame,
    pub inspector_panel: UiAssetEditorShellFrame,
    pub inspector_content_panel: UiAssetEditorShellFrame,
    pub inspector_widget_section: UiAssetEditorShellFrame,
    pub inspector_promote_section: UiAssetEditorShellFrame,
    pub inspector_slot_section: UiAssetEditorShellFrame,
    pub inspector_layout_section: UiAssetEditorShellFrame,
    pub inspector_binding_section: UiAssetEditorShellFrame,
    pub stylesheet_panel: UiAssetEditorShellFrame,
    pub stylesheet_action_row: UiAssetEditorShellFrame,
    pub stylesheet_state_primary_row: UiAssetEditorShellFrame,
    pub stylesheet_state_secondary_row: UiAssetEditorShellFrame,
    pub stylesheet_content_panel: UiAssetEditorShellFrame,
    pub stylesheet_theme_section: UiAssetEditorShellFrame,
    pub stylesheet_authoring_section: UiAssetEditorShellFrame,
    pub stylesheet_matched_rule_section: UiAssetEditorShellFrame,
}

#[derive(Debug, Error)]
pub enum UiAssetEditorShellLayoutError {
    #[error(transparent)]
    Asset(#[from] UiAssetError),
    #[error(transparent)]
    Build(#[from] UiTemplateBuildError),
    #[error(transparent)]
    Layout(#[from] UiTreeError),
    #[error("ui asset editor bootstrap shell is missing control `{control_id}`")]
    MissingControl { control_id: &'static str },
}

pub fn build_ui_asset_editor_shell_layout(
    size: UiSize,
) -> Result<UiAssetEditorShellLayout, UiAssetEditorShellLayoutError> {
    let layout = UiAssetLoader::load_toml_file(asset_path(UI_ASSET_EDITOR_LAYOUT_ASSET_PATH))?;
    let widget = UiAssetLoader::load_toml_file(asset_path(UI_ASSET_EDITOR_WIDGET_ASSET_PATH))?;
    let style = UiAssetLoader::load_toml_file(asset_path(UI_ASSET_EDITOR_STYLE_ASSET_PATH))?;
    let mut compiler = UiDocumentCompiler::default();

    for reference in [
        UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_HEADER_SHELL_REFERENCE,
        UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_SECTION_CARD_REFERENCE,
    ] {
        compiler.register_widget_import(reference.to_string(), widget.clone())?;
    }
    compiler.register_style_import(UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID.to_string(), style)?;

    let compiled = compiler.compile(&layout)?;
    let mut surface = UiTemplateSurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("ui_asset_editor.shell_layout".to_string()),
        &compiled,
    )?;
    surface.compute_layout(size)?;

    Ok(UiAssetEditorShellLayout {
        header_panel: control_frame(&surface, HEADER_PANEL_CONTROL_ID)?,
        header_asset_row: control_frame(&surface, HEADER_ASSET_ROW_CONTROL_ID)?,
        header_status_row: control_frame(&surface, HEADER_STATUS_ROW_CONTROL_ID)?,
        header_action_row: control_frame(&surface, HEADER_ACTION_ROW_CONTROL_ID)?,
        left_column: control_frame(&surface, LEFT_COLUMN_CONTROL_ID)?,
        center_column: control_frame(&surface, CENTER_COLUMN_CONTROL_ID)?,
        right_column: control_frame(&surface, RIGHT_COLUMN_CONTROL_ID)?,
        palette_panel: control_frame(&surface, PALETTE_PANEL_CONTROL_ID)?,
        hierarchy_panel: control_frame(&surface, HIERARCHY_PANEL_CONTROL_ID)?,
        designer_panel: control_frame(&surface, DESIGNER_PANEL_CONTROL_ID)?,
        designer_canvas_panel: control_frame(&surface, DESIGNER_CANVAS_PANEL_CONTROL_ID)?,
        render_stack_panel: control_frame(&surface, RENDER_STACK_PANEL_CONTROL_ID)?,
        action_bar_panel: control_frame(&surface, ACTION_BAR_PANEL_CONTROL_ID)?,
        action_insert_row: control_frame(&surface, ACTION_INSERT_ROW_CONTROL_ID)?,
        action_reparent_row: control_frame(&surface, ACTION_REPARENT_ROW_CONTROL_ID)?,
        action_structure_row: control_frame(&surface, ACTION_STRUCTURE_ROW_CONTROL_ID)?,
        source_panel: control_frame(&surface, SOURCE_PANEL_CONTROL_ID)?,
        source_info_panel: control_frame(&surface, SOURCE_INFO_PANEL_CONTROL_ID)?,
        source_outline_panel: control_frame(&surface, SOURCE_OUTLINE_PANEL_CONTROL_ID)?,
        mock_workspace_panel: control_frame(&surface, MOCK_WORKSPACE_PANEL_CONTROL_ID)?,
        mock_subjects_panel: control_frame(&surface, MOCK_SUBJECTS_PANEL_CONTROL_ID)?,
        mock_editor_panel: control_frame(&surface, MOCK_EDITOR_PANEL_CONTROL_ID)?,
        mock_state_graph_panel: control_frame(&surface, MOCK_STATE_GRAPH_PANEL_CONTROL_ID)?,
        source_text_panel: control_frame(&surface, SOURCE_TEXT_PANEL_CONTROL_ID)?,
        inspector_panel: control_frame(&surface, INSPECTOR_PANEL_CONTROL_ID)?,
        inspector_content_panel: control_frame(&surface, INSPECTOR_CONTENT_PANEL_CONTROL_ID)?,
        inspector_widget_section: control_frame(&surface, INSPECTOR_WIDGET_SECTION_CONTROL_ID)?,
        inspector_promote_section: control_frame(&surface, INSPECTOR_PROMOTE_SECTION_CONTROL_ID)?,
        inspector_slot_section: control_frame(&surface, INSPECTOR_SLOT_SECTION_CONTROL_ID)?,
        inspector_layout_section: control_frame(&surface, INSPECTOR_LAYOUT_SECTION_CONTROL_ID)?,
        inspector_binding_section: control_frame(&surface, INSPECTOR_BINDING_SECTION_CONTROL_ID)?,
        stylesheet_panel: control_frame(&surface, STYLESHEET_PANEL_CONTROL_ID)?,
        stylesheet_action_row: control_frame(&surface, STYLESHEET_ACTION_ROW_CONTROL_ID)?,
        stylesheet_state_primary_row: control_frame(
            &surface,
            STYLESHEET_STATE_PRIMARY_ROW_CONTROL_ID,
        )?,
        stylesheet_state_secondary_row: control_frame(
            &surface,
            STYLESHEET_STATE_SECONDARY_ROW_CONTROL_ID,
        )?,
        stylesheet_content_panel: control_frame(&surface, STYLESHEET_CONTENT_PANEL_CONTROL_ID)?,
        stylesheet_theme_section: control_frame(&surface, STYLESHEET_THEME_SECTION_CONTROL_ID)?,
        stylesheet_authoring_section: control_frame(
            &surface,
            STYLESHEET_AUTHORING_SECTION_CONTROL_ID,
        )?,
        stylesheet_matched_rule_section: control_frame(
            &surface,
            STYLESHEET_MATCHED_RULE_SECTION_CONTROL_ID,
        )?,
    })
}

fn asset_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative.trim_start_matches('/'))
}

fn control_frame(
    surface: &UiSurface,
    control_id: &'static str,
) -> Result<UiAssetEditorShellFrame, UiAssetEditorShellLayoutError> {
    let node_id = surface_control_node_id(surface, control_id)
        .ok_or(UiAssetEditorShellLayoutError::MissingControl { control_id })?;
    let frame = surface
        .tree
        .node(node_id)
        .map(|node| node.layout_cache.frame)
        .ok_or(UiAssetEditorShellLayoutError::MissingControl { control_id })?;
    Ok(frame.into())
}

fn surface_control_node_id(surface: &UiSurface, control_id: &str) -> Option<UiNodeId> {
    surface.tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref())
            .filter(|candidate| *candidate == control_id)
            .map(|_| node.node_id)
    })
}
