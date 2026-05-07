use std::collections::BTreeMap;
use std::path::PathBuf;

use slint::SharedString;
use thiserror::Error;
use toml::Value;
use zircon_runtime::asset::runtime_asset_path_with_dev_asset_root;
use zircon_runtime::ui::template::UiTemplateBuildError;
use zircon_runtime_interface::ui::{
    event_ui::UiTreeId,
    layout::UiSize,
    surface::{UiRenderCommandKind, UiTextAlign},
    template::UiAssetError,
    tree::{UiTemplateNodeMetadata, UiTreeError},
};

use crate::ui::layouts::views::{
    resolve_visual_assets, ViewTemplateFrameData, ViewTemplateNodeData,
};
use crate::ui::template::EditorTemplateRuntimeService;

use super::contract::{
    UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID,
    UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_HEADER_SHELL_REFERENCE,
    UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_SECTION_CARD_REFERENCE,
};

const UI_ASSET_EDITOR_LAYOUT_ASSET_PATH: &str = "/assets/ui/editor/ui_asset_editor.ui.toml";
const UI_ASSET_EDITOR_WIDGET_ASSET_PATH: &str = "/assets/ui/editor/editor_widgets.ui.toml";
const UI_ASSET_EDITOR_STYLE_ASSET_PATH: &str = "/assets/ui/theme/editor_base.ui.toml";

const CENTER_COLUMN_CONTROL_ID: &str = "CenterColumn";
const DESIGNER_PANEL_CONTROL_ID: &str = "DesignerPanel";
const DESIGNER_CANVAS_PANEL_CONTROL_ID: &str = "DesignerCanvasPanel";
const INSPECTOR_PANEL_CONTROL_ID: &str = "InspectorPanel";
const STYLESHEET_PANEL_CONTROL_ID: &str = "StylesheetPanel";

#[derive(Clone, Debug, Default)]
pub(crate) struct UiAssetEditorNodeProjection {
    pub nodes: Vec<ViewTemplateNodeData>,
    pub center_column_node: ViewTemplateNodeData,
    pub designer_panel_node: ViewTemplateNodeData,
    pub designer_canvas_panel_node: ViewTemplateNodeData,
    pub inspector_panel_node: ViewTemplateNodeData,
    pub stylesheet_panel_node: ViewTemplateNodeData,
}

#[derive(Debug, Error)]
enum UiAssetEditorNodeProjectionError {
    #[error(transparent)]
    Asset(#[from] UiAssetError),
    #[error(transparent)]
    Build(#[from] UiTemplateBuildError),
    #[error(transparent)]
    Layout(#[from] UiTreeError),
}

#[derive(Clone, Debug, Default)]
struct TemplateRenderInfo {
    is_quad: bool,
    text: Option<String>,
    font_size: f32,
    border_width: f32,
    corner_radius: f32,
    text_align: UiTextAlign,
}

pub(crate) fn ui_asset_editor_node_projection(size: UiSize) -> UiAssetEditorNodeProjection {
    build_ui_asset_editor_node_projection(size).unwrap_or_default()
}

fn build_ui_asset_editor_node_projection(
    size: UiSize,
) -> Result<UiAssetEditorNodeProjection, UiAssetEditorNodeProjectionError> {
    let template_service = EditorTemplateRuntimeService;
    let layout =
        template_service.load_document_file(asset_path(UI_ASSET_EDITOR_LAYOUT_ASSET_PATH))?;
    let widget =
        template_service.load_document_file(asset_path(UI_ASSET_EDITOR_WIDGET_ASSET_PATH))?;
    let style =
        template_service.load_document_file(asset_path(UI_ASSET_EDITOR_STYLE_ASSET_PATH))?;
    let mut widget_imports = BTreeMap::new();
    let mut style_imports = BTreeMap::new();

    for reference in [
        UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_HEADER_SHELL_REFERENCE,
        UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_SECTION_CARD_REFERENCE,
    ] {
        widget_imports.insert(reference.to_string(), widget.clone());
    }
    style_imports.insert(UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID.to_string(), style);

    let compiled = template_service.compile_document_with_import_maps(
        &layout,
        &widget_imports,
        &style_imports,
    )?;
    let mut surface = template_service.build_surface_from_compiled_document(
        UiTreeId::new("ui_asset_editor.node_projection".to_string()),
        &compiled,
    )?;
    surface.compute_layout(size)?;

    let mut render_info_by_node = BTreeMap::new();
    for command in template_service.extract_render(&surface).list.commands {
        let entry = render_info_by_node
            .entry(command.node_id)
            .or_insert(TemplateRenderInfo {
                text_align: command.style.text_align,
                ..TemplateRenderInfo::default()
            });
        entry.is_quad |= command.kind == UiRenderCommandKind::Quad;
        entry.font_size = entry.font_size.max(command.style.font_size.max(0.0));
        entry.border_width = entry.border_width.max(command.style.border_width.max(0.0));
        entry.corner_radius = entry
            .corner_radius
            .max(command.style.corner_radius.max(0.0));
        entry.text_align = command.style.text_align;
        if command.kind == UiRenderCommandKind::Text && command.text.is_some() {
            entry.text = command.text.clone();
        }
    }

    let mut nodes = surface
        .tree
        .nodes
        .values()
        .filter_map(|node| {
            let metadata = node.template_metadata.as_ref()?;
            let control_id = metadata.control_id.clone()?;
            let render_info = render_info_by_node.get(&node.node_id);
            let visual_assets = resolve_visual_assets(metadata);

            Some(ViewTemplateNodeData {
                node_id: SharedString::from(node.node_path.0.clone()),
                control_id: SharedString::from(control_id),
                role: SharedString::from(resolve_role(&metadata.component, render_info, metadata)),
                text: SharedString::from(
                    render_info
                        .and_then(|info| info.text.clone())
                        .unwrap_or_default(),
                ),
                dispatch_kind: string_attribute(metadata, "dispatch_kind")
                    .unwrap_or_default()
                    .into(),
                action_id: string_attribute(metadata, "action_id")
                    .unwrap_or_default()
                    .into(),
                surface_variant: string_attribute(metadata, "surface_variant")
                    .unwrap_or_default()
                    .into(),
                text_tone: string_attribute(metadata, "text_tone")
                    .unwrap_or_default()
                    .into(),
                button_variant: string_attribute(metadata, "button_variant")
                    .unwrap_or_default()
                    .into(),
                font_size: number_attribute(metadata, "font_size")
                    .unwrap_or_else(|| render_info.map(|info| info.font_size).unwrap_or_default()),
                font_weight: integer_attribute(metadata, "font_weight").unwrap_or(400),
                text_align: string_attribute(metadata, "text_align")
                    .unwrap_or_else(|| {
                        text_align_name(
                            render_info
                                .map(|info| info.text_align)
                                .unwrap_or(UiTextAlign::Left),
                        )
                        .to_string()
                    })
                    .into(),
                overflow: string_attribute(metadata, "overflow")
                    .unwrap_or_default()
                    .into(),
                corner_radius: number_attribute(metadata, "corner_radius")
                    .or(number_attribute(metadata, "radius"))
                    .unwrap_or_else(|| {
                        render_info
                            .map(|info| info.corner_radius)
                            .unwrap_or_default()
                    }),
                border_width: number_attribute(metadata, "border_width").unwrap_or_else(|| {
                    render_info
                        .map(|info| info.border_width)
                        .unwrap_or_default()
                }),
                selected: bool_attribute(metadata, "selected").unwrap_or(false),
                focused: bool_attribute(metadata, "focused").unwrap_or(false),
                hovered: bool_attribute(metadata, "hovered").unwrap_or(false),
                pressed: bool_attribute(metadata, "pressed").unwrap_or(false),
                disabled: bool_attribute(metadata, "disabled").unwrap_or(false)
                    || bool_attribute(metadata, "enabled") == Some(false),
                media_source: SharedString::from(visual_assets.media_source),
                icon_name: SharedString::from(visual_assets.icon_name),
                has_preview_image: visual_assets.has_preview_image,
                preview_image: visual_assets.preview_image,
                frame: ViewTemplateFrameData {
                    x: node.layout_cache.frame.x,
                    y: node.layout_cache.frame.y,
                    width: node.layout_cache.frame.width,
                    height: node.layout_cache.frame.height,
                },
            })
        })
        .collect::<Vec<_>>();
    nodes.sort_by(|left, right| left.control_id.cmp(&right.control_id));

    Ok(UiAssetEditorNodeProjection {
        center_column_node: find_node(&nodes, CENTER_COLUMN_CONTROL_ID),
        designer_panel_node: find_node(&nodes, DESIGNER_PANEL_CONTROL_ID),
        designer_canvas_panel_node: find_node(&nodes, DESIGNER_CANVAS_PANEL_CONTROL_ID),
        inspector_panel_node: find_node(&nodes, INSPECTOR_PANEL_CONTROL_ID),
        stylesheet_panel_node: find_node(&nodes, STYLESHEET_PANEL_CONTROL_ID),
        nodes,
    })
}

fn asset_path(relative: &str) -> PathBuf {
    runtime_asset_path_with_dev_asset_root(relative, editor_dev_asset_root())
}

fn editor_dev_asset_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}

fn find_node(nodes: &[ViewTemplateNodeData], control_id: &str) -> ViewTemplateNodeData {
    nodes
        .iter()
        .find(|node| node.control_id.as_str() == control_id)
        .cloned()
        .unwrap_or_default()
}

fn resolve_role(
    component: &str,
    render_info: Option<&TemplateRenderInfo>,
    metadata: &UiTemplateNodeMetadata,
) -> &'static str {
    match component {
        "Button" => "Button",
        "Label" => "Label",
        "Icon" => "Icon",
        "IconButton" => "IconButton",
        "SvgIcon" => "SvgIcon",
        _ if string_attribute(metadata, "surface_variant").is_some()
            || render_info.is_some_and(|info| info.is_quad) =>
        {
            "Panel"
        }
        _ if metadata.control_id.is_some() => "Mount",
        _ => "Group",
    }
}

fn string_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<String> {
    metadata
        .attributes
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn number_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<f32> {
    metadata.attributes.get(key).and_then(|value| match value {
        Value::Float(value) => Some(*value as f32),
        Value::Integer(value) => Some(*value as f32),
        _ => None,
    })
}

fn integer_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<i32> {
    metadata
        .attributes
        .get(key)
        .and_then(Value::as_integer)
        .map(|value| value as i32)
}

fn bool_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<bool> {
    metadata.attributes.get(key).and_then(Value::as_bool)
}

fn text_align_name(align: UiTextAlign) -> &'static str {
    match align {
        UiTextAlign::Left => "left",
        UiTextAlign::Center => "center",
        UiTextAlign::Right => "right",
    }
}
