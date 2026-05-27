use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

use crate::ui::retained_host::primitives::SharedString;
use thiserror::Error;
use toml::Value;
use zircon_runtime::asset::runtime_asset_path_with_dev_asset_root;
use zircon_runtime::ui::{
    style::resolve_button_style_from_values,
    surface::{extract_ui_render_tree, UiSurface},
    v2::{UiV2CompiledDocument, UiV2PrototypeStoreFileCache, UiV2SurfaceBuilder},
};
use zircon_runtime_interface::ui::{
    event_ui::UiTreeId,
    layout::UiSize,
    surface::{UiRenderCommandKind, UiTextAlign},
    tree::{UiTemplateNodeMetadata, UiTreeError},
    v2::{UiV2AssetDocument, UiV2AssetError},
};

use crate::ui::layouts::views::{
    default_transition_duration_ms, default_transition_easing, preferred_binding_id,
    resolve_commit_action_id, resolve_component_role, resolve_component_variant,
    resolve_edit_action_id, resolve_node_popup_open, resolve_node_value_number,
    resolve_node_value_percent, resolve_node_value_text, resolve_transition_in,
    resolve_transition_kind, resolve_transition_progress, resolve_visual_assets,
    ViewTemplateFrameData, ViewTemplateNodeData,
};

const UI_ASSET_EDITOR_LAYOUT_ASSET_PATH: &str = "/assets/ui/editor/ui_asset_editor.v2.ui.toml";
const UI_ASSET_EDITOR_STYLE_ASSET_PATH: &str = "/assets/ui/theme/editor_material.v2.ui.toml";

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
    V2Asset(#[from] UiV2AssetError),
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
    let session =
        NODE_PROJECTION_SESSION.get_or_init(|| Mutex::new(NodeProjectionSession::default()));
    session
        .lock()
        .map(|mut session| session.project(size).unwrap_or_default())
        .unwrap_or_default()
}

static NODE_PROJECTION_SESSION: OnceLock<Mutex<NodeProjectionSession>> = OnceLock::new();

#[derive(Default)]
struct NodeProjectionSession {
    document: Option<NodeProjectionDocument>,
    surface: Option<UiSurface>,
    size: Option<UiSize>,
}

impl NodeProjectionSession {
    fn project(
        &mut self,
        size: UiSize,
    ) -> Result<UiAssetEditorNodeProjection, UiAssetEditorNodeProjectionError> {
        if self.document.is_none() {
            self.document = Some(load_node_projection_document()?);
        }

        if self.surface.is_none() {
            let document = self
                .document
                .as_ref()
                .expect("v2 projection document should be initialized");
            let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
                UiTreeId::new("ui_asset_editor.node_projection".to_string()),
                document.root_document.as_ref(),
                document.compiled.as_ref(),
            )?;
            surface.compute_layout(size)?;
            self.surface = Some(surface);
            self.size = Some(size);
        } else if self.size != Some(size) {
            if let Some(surface) = self.surface.as_mut() {
                mark_surface_roots_layout_dirty(surface);
                surface.rebuild_dirty(size)?;
            }
            self.size = Some(size);
        }

        project_ui_asset_editor_nodes(
            self.surface
                .as_ref()
                .expect("projection surface should be initialized"),
        )
    }
}

#[derive(Clone)]
struct NodeProjectionDocument {
    root_document: Arc<UiV2AssetDocument>,
    compiled: Arc<UiV2CompiledDocument>,
}

fn load_node_projection_document(
) -> Result<NodeProjectionDocument, UiAssetEditorNodeProjectionError> {
    let outcome = node_projection_v2_store_file_cache()
        .lock()
        .expect("ui asset editor v2 projection cache mutex should not be poisoned")
        .load_store([
            asset_path(UI_ASSET_EDITOR_LAYOUT_ASSET_PATH),
            asset_path(UI_ASSET_EDITOR_STYLE_ASSET_PATH),
        ])?;

    Ok(NodeProjectionDocument {
        root_document: outcome.root_document,
        compiled: outcome.compiled,
    })
}

fn node_projection_v2_store_file_cache() -> &'static Mutex<UiV2PrototypeStoreFileCache> {
    static CACHE: OnceLock<Mutex<UiV2PrototypeStoreFileCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(UiV2PrototypeStoreFileCache::new()))
}

fn mark_surface_roots_layout_dirty(surface: &mut UiSurface) {
    for root_id in surface.tree.roots.clone() {
        if let Some(root) = surface.tree.nodes.get_mut(&root_id) {
            root.dirty.layout = true;
            root.dirty.hit_test = true;
            root.dirty.render = true;
        }
    }
}

fn project_ui_asset_editor_nodes(
    surface: &UiSurface,
) -> Result<UiAssetEditorNodeProjection, UiAssetEditorNodeProjectionError> {
    let mut render_info_by_node = BTreeMap::new();
    for command in extract_ui_render_tree(&surface.tree).list.commands {
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
            let text = render_info
                .and_then(|info| info.text.clone())
                .unwrap_or_default();
            let component_role = resolve_component_role(&metadata.component);
            let binding_id = preferred_binding_id(metadata, None).unwrap_or_default();
            let edit_action_id = resolve_edit_action_id(metadata, component_role, &binding_id);
            let commit_action_id = resolve_commit_action_id(metadata);
            let component_variant = resolve_component_variant(metadata);
            let value_text = resolve_node_value_text(metadata, &text, component_role);
            let value_number = resolve_node_value_number(metadata);
            let value_percent = resolve_node_value_percent(metadata, component_role, value_number);
            let visual_assets = resolve_visual_assets(metadata);
            let button_style = resolve_button_style_from_values(&metadata.style_overrides);
            let popup_open = resolve_node_popup_open(metadata);
            let transition_kind = resolve_transition_kind(metadata, component_role);
            let transition_in =
                resolve_transition_in(metadata, !transition_kind.is_empty(), popup_open);
            let transition_status =
                string_attribute(metadata, "transition_status").unwrap_or_else(|| {
                    if transition_in {
                        "entered".to_string()
                    } else {
                        "exited".to_string()
                    }
                });
            let transition_progress =
                resolve_transition_progress(metadata, transition_status.as_str(), transition_in);

            Some(ViewTemplateNodeData {
                node_id: SharedString::from(node.node_path.0.clone()),
                control_id: SharedString::from(control_id),
                role: SharedString::from(resolve_role(&metadata.component, render_info, metadata)),
                text: SharedString::from(text),
                component_role: SharedString::from(component_role),
                component_variant: SharedString::from(component_variant),
                value_text: SharedString::from(value_text),
                value_number,
                value_percent,
                dispatch_kind: string_attribute(metadata, "dispatch_kind")
                    .unwrap_or_default()
                    .into(),
                action_id: string_attribute(metadata, "action_id")
                    .unwrap_or_default()
                    .into(),
                binding_id: SharedString::from(binding_id),
                edit_action_id: SharedString::from(edit_action_id),
                commit_action_id: SharedString::from(commit_action_id),
                surface_variant: string_attribute(metadata, "surface_variant")
                    .unwrap_or_default()
                    .into(),
                text_tone: string_attribute(metadata, "text_tone")
                    .unwrap_or_default()
                    .into(),
                button_variant: string_attribute(metadata, "button_variant")
                    .unwrap_or_default()
                    .into(),
                button_style,
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
                z_index: integer_attribute(metadata, "z_index").unwrap_or(node.z_index),
                transition_kind: SharedString::from(transition_kind.clone()),
                transition_in,
                transition_entered: bool_attribute(metadata, "transition_entered")
                    .or_else(|| bool_attribute(metadata, "entered"))
                    .unwrap_or_else(|| {
                        transition_in
                            && transition_status == "entered"
                            && transition_progress >= 1.0
                    }),
                transition_progress,
                transition_duration_ms: integer_attribute(metadata, "transition_duration_ms")
                    .or_else(|| integer_attribute(metadata, "timeout_ms"))
                    .or_else(|| integer_attribute(metadata, "duration_ms"))
                    .unwrap_or_else(|| {
                        default_transition_duration_ms(&transition_kind, transition_in)
                    }),
                transition_easing: string_attribute(metadata, "transition_easing")
                    .or_else(|| string_attribute(metadata, "easing"))
                    .unwrap_or_else(|| {
                        default_transition_easing(&transition_kind, transition_in).to_string()
                    })
                    .into(),
                transition_direction: string_attribute(metadata, "transition_direction")
                    .or_else(|| string_attribute(metadata, "direction"))
                    .unwrap_or_else(|| {
                        if transition_kind == "slide" {
                            "down".to_string()
                        } else {
                            String::new()
                        }
                    })
                    .into(),
                selected: bool_attribute(metadata, "selected").unwrap_or(false),
                popup_open,
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
        "Label" | "Text" => "Label",
        "InputField" | "TextField" => "InputField",
        "NumberField" => "InputField",
        "RangeField" | "Slider" => "RangeField",
        "Progress" | "ProgressBar" | "LinearProgress" | "CircularProgress" | "Spinner" => {
            "Progress"
        }
        "Skeleton" => "Skeleton",
        "Backdrop" => "Backdrop",
        "Paper" | "Dialog" | "AlertDialog" | "Popover" | "Popper" | "Tooltip" | "Snackbar"
        | "Menu" | "Drawer" => "Panel",
        "Toggle" | "Checkbox" | "Radio" | "RadioField" => "Toggle",
        "ComboBox" | "Dropdown" | "EnumField" | "FlagsField" | "SearchSelect" => "ComboBox",
        "TreeView" | "TreeRow" => "TreeView",
        "EditableTable" | "Table" => "Table",
        "AssetField" | "ObjectField" | "InstanceField" => "InputField",
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
