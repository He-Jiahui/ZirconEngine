use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use crate::ui::retained_host::primitives::SharedString;
use thiserror::Error;
use toml::Value;
use zircon_runtime::asset::runtime_asset_path_with_dev_asset_root;
use zircon_runtime::ui::style::resolve_button_style_from_values;
use zircon_runtime::ui::surface::{extract_ui_render_tree, UiSurface};
use zircon_runtime::ui::tree::UiRuntimeTreeAccessExt;
use zircon_runtime::ui::v2::{UiV2PrototypeStoreFileCache, UiV2SurfaceBuilder};
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    event_ui::UiTreeId,
    layout::UiSize,
    surface::{UiRenderCommandKind, UiTextAlign},
    tree::{UiTemplateNodeMetadata, UiTreeError},
    v2::UiV2AssetError,
};

use super::{load_preview_image, ViewTemplateFrameData, ViewTemplateNodeData};

pub(crate) struct ViewTemplateVisualAssets {
    pub(crate) media_source: String,
    pub(crate) icon_name: String,
    pub(crate) preview_image: crate::ui::retained_host::primitives::Image,
    pub(crate) has_preview_image: bool,
}

#[derive(Debug, Error)]
pub enum ViewTemplateProjectionError {
    #[error("editor view projection requires v2 UI assets, got `{0}`")]
    LegacyAssetPath(String),
    #[error(transparent)]
    V2Asset(#[from] UiV2AssetError),
    #[error(transparent)]
    Layout(#[from] UiTreeError),
}

pub(crate) fn build_view_template_nodes(
    document_tree_id: &str,
    layout_asset_path: &str,
    style_imports: &[(&str, &str)],
    size: UiSize,
    text_overrides: &BTreeMap<String, String>,
) -> Result<Vec<ViewTemplateNodeData>, ViewTemplateProjectionError> {
    build_view_template_nodes_with_imports(
        document_tree_id,
        layout_asset_path,
        &[],
        style_imports,
        size,
        text_overrides,
    )
}

pub(crate) fn build_view_template_nodes_with_imports(
    document_tree_id: &str,
    layout_asset_path: &str,
    widget_imports: &[(&str, &str)],
    style_imports: &[(&str, &str)],
    size: UiSize,
    text_overrides: &BTreeMap<String, String>,
) -> Result<Vec<ViewTemplateNodeData>, ViewTemplateProjectionError> {
    if is_v2_asset_path(layout_asset_path) {
        return build_view_template_nodes_from_v2_asset(
            document_tree_id,
            layout_asset_path,
            style_imports,
            size,
            text_overrides,
        );
    }

    let _ = (widget_imports, style_imports, size, text_overrides);
    Err(ViewTemplateProjectionError::LegacyAssetPath(
        layout_asset_path.to_string(),
    ))
}

fn build_view_template_nodes_from_v2_asset(
    document_tree_id: &str,
    layout_asset_path: &str,
    style_imports: &[(&str, &str)],
    size: UiSize,
    text_overrides: &BTreeMap<String, String>,
) -> Result<Vec<ViewTemplateNodeData>, ViewTemplateProjectionError> {
    let outcome = view_v2_store_file_cache()
        .lock()
        .expect("view v2 store cache mutex should not be poisoned")
        .load_store(v2_source_paths(layout_asset_path, style_imports))?;
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new(document_tree_id.to_string()),
        outcome.root_document.as_ref(),
        outcome.compiled.as_ref(),
    )?;
    surface.compute_layout(size)?;

    Ok(view_template_nodes_from_surface(&surface, text_overrides))
}

fn is_v2_asset_path(path: &str) -> bool {
    path.ends_with(".v2.ui.toml")
}

fn v2_source_paths(layout_asset_path: &str, style_imports: &[(&str, &str)]) -> Vec<PathBuf> {
    let mut paths = Vec::with_capacity(1 + style_imports.len());
    paths.push(asset_path(layout_asset_path));
    paths.extend(
        style_imports
            .iter()
            .map(|(_, style_path)| asset_path(style_path)),
    );
    paths
}

fn view_v2_store_file_cache() -> &'static Mutex<UiV2PrototypeStoreFileCache> {
    static CACHE: OnceLock<Mutex<UiV2PrototypeStoreFileCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(UiV2PrototypeStoreFileCache::new()))
}

fn view_template_nodes_from_surface(
    surface: &UiSurface,
    text_overrides: &BTreeMap<String, String>,
) -> Vec<ViewTemplateNodeData> {
    let render = extract_ui_render_tree(&surface.tree);
    let mut nodes = Vec::new();
    for command in render.list.commands {
        let Some(tree_node) = surface.tree.node(command.node_id) else {
            continue;
        };
        let Some(metadata) = tree_node.template_metadata.as_ref() else {
            continue;
        };

        let role = resolve_role(&metadata.component, &command.kind, metadata);
        if role == "Group" {
            continue;
        }

        let control_id = metadata.control_id.clone().unwrap_or_default();
        let text = text_overrides
            .get(&control_id)
            .cloned()
            .or_else(|| string_attribute(metadata, "label"))
            .or_else(|| string_attribute(metadata, "text"))
            .or(command.text.clone())
            .unwrap_or_default();
        let component_role = resolve_component_role(&metadata.component);
        let binding_id = preferred_binding_id(metadata, None).unwrap_or_default();
        let edit_action_id = resolve_edit_action_id(metadata, component_role, &binding_id);
        let commit_action_id = resolve_commit_action_id(metadata);
        let value_text = resolve_node_value_text(metadata, &text, component_role);
        let visual_assets = resolve_visual_assets(metadata);
        let button_style = resolve_button_style_from_values(&metadata.style_overrides);

        nodes.push(ViewTemplateNodeData {
            node_id: tree_node.node_path.0.clone().into(),
            control_id: control_id.into(),
            role: SharedString::from(role),
            text: text.into(),
            component_role: component_role.into(),
            value_text: value_text.into(),
            dispatch_kind: string_attribute(metadata, "dispatch_kind")
                .unwrap_or_default()
                .into(),
            action_id: string_attribute(metadata, "action_id")
                .unwrap_or_default()
                .into(),
            binding_id: binding_id.into(),
            edit_action_id: edit_action_id.into(),
            commit_action_id: commit_action_id.into(),
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
                .unwrap_or(command.style.font_size.max(0.0)),
            font_weight: integer_attribute(metadata, "font_weight").unwrap_or(400),
            text_align: string_attribute(metadata, "text_align")
                .unwrap_or_else(|| text_align_name(command.style.text_align).to_string())
                .into(),
            overflow: string_attribute(metadata, "overflow")
                .unwrap_or_default()
                .into(),
            corner_radius: number_attribute(metadata, "corner_radius")
                .or(number_attribute(metadata, "radius"))
                .unwrap_or(command.style.corner_radius.max(0.0)),
            border_width: number_attribute(metadata, "border_width")
                .unwrap_or(command.style.border_width.max(0.0)),
            selected: bool_attribute(metadata, "selected").unwrap_or(false),
            focused: bool_attribute(metadata, "focused").unwrap_or(false),
            hovered: bool_attribute(metadata, "hovered").unwrap_or(false),
            pressed: bool_attribute(metadata, "pressed").unwrap_or(false),
            disabled: bool_attribute(metadata, "disabled").unwrap_or(false)
                || bool_attribute(metadata, "enabled") == Some(false),
            media_source: visual_assets.media_source.into(),
            icon_name: visual_assets.icon_name.into(),
            has_preview_image: visual_assets.has_preview_image,
            preview_image: visual_assets.preview_image,
            frame: ViewTemplateFrameData {
                x: command.frame.x,
                y: command.frame.y,
                width: command.frame.width,
                height: command.frame.height,
            },
        });
    }

    nodes
}

pub(crate) fn resolve_visual_assets(metadata: &UiTemplateNodeMetadata) -> ViewTemplateVisualAssets {
    let media_source = string_attribute(metadata, "image")
        .or_else(|| string_attribute(metadata, "source"))
        .or_else(|| string_attribute(metadata, "media"))
        .or_else(|| {
            matches!(metadata.component.as_str(), "Image" | "SvgIcon")
                .then(|| string_attribute(metadata, "value"))
                .flatten()
        })
        .unwrap_or_default();
    let icon_name = string_attribute(metadata, "icon")
        .or_else(|| {
            (metadata.component.as_str() == "Icon")
                .then(|| string_attribute(metadata, "value"))
                .flatten()
        })
        .unwrap_or_default();
    let preview_image = load_preview_image(&media_source, &icon_name);
    let preview_size = preview_image.size();
    let has_preview_image = preview_size.width > 0 && preview_size.height > 0;

    ViewTemplateVisualAssets {
        media_source,
        icon_name,
        preview_image,
        has_preview_image,
    }
}

fn asset_path(relative: &str) -> PathBuf {
    runtime_asset_path_with_dev_asset_root(relative, editor_dev_asset_root())
}

fn editor_dev_asset_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}

fn resolve_role(
    component: &str,
    kind: &UiRenderCommandKind,
    metadata: &UiTemplateNodeMetadata,
) -> &'static str {
    match component {
        "Button" => "Button",
        "Label" | "Text" => "Label",
        "InputField" | "TextField" => "InputField",
        "NumberField" => "InputField",
        "RangeField" => "RangeField",
        "Toggle" | "Checkbox" | "Radio" | "RadioField" => "Toggle",
        "ComboBox" | "Dropdown" | "EnumField" | "FlagsField" | "SearchSelect" => "ComboBox",
        "TreeView" | "TreeRow" => "TreeView",
        "EditableTable" | "Table" => "Table",
        "AssetField" | "ObjectField" | "InstanceField" => "InputField",
        "Icon" => "Icon",
        "IconButton" => "IconButton",
        "SvgIcon" => "SvgIcon",
        _ if string_attribute(metadata, "surface_variant").is_some()
            || matches!(kind, UiRenderCommandKind::Quad) =>
        {
            "Panel"
        }
        _ if metadata.control_id.is_some() => "Mount",
        _ => "Group",
    }
}

pub(crate) fn resolve_component_role(component: &str) -> &'static str {
    match component {
        "Button" => "button",
        "Text" => "text",
        "Label" => "label",
        "Image" => "image",
        "Svg" => "svg",
        "SvgIcon" => "svg-icon",
        "Canvas" => "canvas",
        "Icon" => "icon",
        "IconButton" => "icon-button",
        "InputField" | "TextField" => "input-field",
        "NumberField" => "number-field",
        "RangeField" => "range-field",
        "ProgressBar" => "progress-bar",
        "Popup" => "popup",
        "Toggle" => "toggle",
        "Checkbox" => "checkbox",
        "ComboBox" => "combo-box",
        "Dropdown" => "dropdown",
        "EnumField" => "enum-field",
        "FlagsField" => "flags-field",
        "SearchSelect" => "search-select",
        "Radio" => "radio",
        "RadioField" => "radio-field",
        "AssetField" => "asset-field",
        "ObjectField" => "object-field",
        "InstanceField" => "instance-field",
        "Foldout" => "foldout",
        "TreeView" => "tree-view",
        "TreeRow" => "tree-row",
        "EditableTable" => "editable-table",
        "Table" => "table",
        "MessageBox" => "message-box",
        _ => "",
    }
}

pub(crate) fn resolve_node_value_text(
    metadata: &UiTemplateNodeMetadata,
    display_text: &str,
    component_role: &str,
) -> String {
    if let Some(value_text) = string_attribute(metadata, "value_text") {
        return value_text;
    }
    if let Some(value) = metadata.attributes.get("value") {
        return value_to_display_text(value);
    }
    if matches!(component_role, "input-field" | "number-field") {
        let placeholder = string_attribute(metadata, "placeholder").unwrap_or_default();
        if !display_text.is_empty() && display_text != placeholder {
            return display_text.to_string();
        }
    }
    String::new()
}

pub(crate) fn preferred_binding_id(
    metadata: &UiTemplateNodeMetadata,
    event_kind: Option<UiEventKind>,
) -> Option<String> {
    metadata
        .bindings
        .iter()
        .find(|binding| event_kind.is_none_or(|event_kind| binding.event == event_kind))
        .map(|binding| binding.id.clone())
}

pub(crate) fn resolve_edit_action_id(
    metadata: &UiTemplateNodeMetadata,
    component_role: &str,
    binding_id: &str,
) -> String {
    string_attribute(metadata, "edit_action_id")
        .or_else(|| preferred_binding_id(metadata, Some(UiEventKind::Change)))
        .or_else(|| {
            matches!(component_role, "input-field" | "number-field")
                .then(|| binding_id.to_string())
                .filter(|id| !id.is_empty())
        })
        .unwrap_or_default()
}

pub(crate) fn resolve_commit_action_id(metadata: &UiTemplateNodeMetadata) -> String {
    string_attribute(metadata, "commit_action_id")
        .or_else(|| preferred_binding_id(metadata, Some(UiEventKind::Submit)))
        .unwrap_or_default()
}

fn value_to_display_text(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Integer(value) => value.to_string(),
        Value::Float(value) => value.to_string(),
        Value::Boolean(value) => value.to_string(),
        _ => value.to_string(),
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

fn bool_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<bool> {
    metadata.attributes.get(key).and_then(Value::as_bool)
}

fn integer_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<i32> {
    metadata
        .attributes
        .get(key)
        .and_then(Value::as_integer)
        .map(|value| value as i32)
}

fn text_align_name(align: UiTextAlign) -> &'static str {
    match align {
        UiTextAlign::Left => "left",
        UiTextAlign::Center => "center",
        UiTextAlign::Right => "right",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn view_template_projection_rejects_legacy_asset_paths() {
        let text_overrides = BTreeMap::new();
        let error = build_view_template_nodes(
            "view.legacy.project_overview",
            "/assets/ui/editor/project_overview.ui.toml",
            &[],
            UiSize::new(640.0, 480.0),
            &text_overrides,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            ViewTemplateProjectionError::LegacyAssetPath(path)
                if path == "/assets/ui/editor/project_overview.ui.toml"
        ));
    }

    #[test]
    fn v2_view_template_projection_uses_v2_surface_builder_without_legacy_fallback() {
        let text_overrides = BTreeMap::from([(
            "ProjectOverviewTitleText".to_string(),
            "V2 Project".to_string(),
        )]);

        let nodes = build_view_template_nodes(
            "view.v2.project_overview",
            "/assets/ui/editor/project_overview.v2.ui.toml",
            &[],
            UiSize::new(320.0, 240.0),
            &text_overrides,
        )
        .unwrap();

        assert!(
            nodes
                .iter()
                .any(|node| node.control_id == "ProjectOverviewTitleText"
                    && node.text == "V2 Project")
        );
        assert!(nodes.iter().any(|node| node.role == "Button"));
        assert!(
            view_v2_store_file_cache()
                .lock()
                .expect("v2 cache mutex should not be poisoned")
                .len()
                > 0
        );
    }

    #[test]
    fn v2_view_template_projection_reuses_cached_store_for_identical_inputs() {
        let text_overrides = BTreeMap::new();
        let cache = view_v2_store_file_cache();
        cache
            .lock()
            .expect("v2 cache mutex should not be poisoned")
            .clear();

        let first = build_view_template_nodes(
            "view.v2.project_overview.first",
            "/assets/ui/editor/project_overview.v2.ui.toml",
            &[],
            UiSize::new(640.0, 480.0),
            &text_overrides,
        )
        .unwrap();
        let cache_len_after_first = cache
            .lock()
            .expect("v2 cache mutex should not be poisoned")
            .len();

        let second = build_view_template_nodes(
            "view.v2.project_overview.second",
            "/assets/ui/editor/project_overview.v2.ui.toml",
            &[],
            UiSize::new(640.0, 480.0),
            &text_overrides,
        )
        .unwrap();
        let cache_len_after_second = cache
            .lock()
            .expect("v2 cache mutex should not be poisoned")
            .len();

        assert!(
            cache_len_after_first > 0,
            "first v2 projection should populate the store file cache"
        );
        assert_eq!(cache_len_after_second, cache_len_after_first);
        assert_eq!(first.len(), second.len());
        assert_eq!(
            first
                .iter()
                .find(|node| node.control_id == "ProjectOverviewTitleText")
                .map(|node| node.text.clone()),
            second
                .iter()
                .find(|node| node.control_id == "ProjectOverviewTitleText")
                .map(|node| node.text.clone())
        );
    }
}
