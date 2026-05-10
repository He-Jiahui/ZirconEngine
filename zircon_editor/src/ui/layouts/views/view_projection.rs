use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use crate::ui::retained_host::primitives::SharedString;
use thiserror::Error;
use toml::Value;
use zircon_runtime::asset::runtime_asset_path_with_dev_asset_root;
use zircon_runtime::ui::surface::{extract_ui_render_tree, UiSurface};
use zircon_runtime::ui::template::{
    UiDocumentCompiler, UiPrototypeStoreFileCache, UiTemplateBuildError, UiTemplateSurfaceBuilder,
};
use zircon_runtime::ui::tree::UiRuntimeTreeAccessExt;
use zircon_runtime_interface::ui::{
    event_ui::UiTreeId,
    layout::UiSize,
    surface::{UiRenderCommandKind, UiTextAlign},
    template::UiAssetError,
    tree::{UiTemplateNodeMetadata, UiTreeError},
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
    #[error(transparent)]
    Asset(#[from] UiAssetError),
    #[error(transparent)]
    Build(#[from] UiTemplateBuildError),
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
    build_view_template_nodes_from_prototype_store(
        document_tree_id,
        layout_asset_path,
        widget_imports,
        style_imports,
        size,
        text_overrides,
    )
}

fn build_view_template_nodes_from_prototype_store(
    document_tree_id: &str,
    layout_asset_path: &str,
    widget_imports: &[(&str, &str)],
    style_imports: &[(&str, &str)],
    size: UiSize,
    text_overrides: &BTreeMap<String, String>,
) -> Result<Vec<ViewTemplateNodeData>, ViewTemplateProjectionError> {
    let source_paths = prototype_source_paths(layout_asset_path, widget_imports, style_imports);
    let outcome = view_prototype_store_file_cache()
        .lock()
        .expect("view prototype store cache mutex should not be poisoned")
        .load_flat_store(source_paths)?;

    let compiled = UiDocumentCompiler::default()
        .compile_prototype_asset(&outcome.root_asset_id, outcome.store.as_ref())?;
    let mut surface = UiTemplateSurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new(document_tree_id.to_string()),
        &compiled,
    )?;
    surface.compute_layout(size)?;

    Ok(view_template_nodes_from_surface(&surface, text_overrides))
}

fn prototype_source_paths(
    layout_asset_path: &str,
    widget_imports: &[(&str, &str)],
    style_imports: &[(&str, &str)],
) -> Vec<PathBuf> {
    let mut paths = Vec::with_capacity(1 + widget_imports.len() + style_imports.len());
    paths.push(asset_path(layout_asset_path));
    paths.extend(
        widget_imports
            .iter()
            .map(|(_, widget_path)| asset_path(widget_path)),
    );
    paths.extend(
        style_imports
            .iter()
            .map(|(_, style_path)| asset_path(style_path)),
    );
    paths
}

fn view_prototype_store_file_cache() -> &'static Mutex<UiPrototypeStoreFileCache> {
    static CACHE: OnceLock<Mutex<UiPrototypeStoreFileCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(UiPrototypeStoreFileCache::new()))
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
        let visual_assets = resolve_visual_assets(metadata);

        nodes.push(ViewTemplateNodeData {
            node_id: tree_node.node_path.0.clone().into(),
            control_id: control_id.into(),
            role: SharedString::from(role),
            text: text.into(),
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
        "Label" => "Label",
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
    fn flat_view_template_projection_uses_shared_prototype_store_cache() {
        let text_overrides = BTreeMap::new();

        let nodes = build_view_template_nodes_from_prototype_store(
            "view.prototype.quest_log",
            "/assets/ui/runtime/fixtures/quest_log_dialog.ui.toml",
            &[],
            &[],
            UiSize::new(640.0, 480.0),
            &text_overrides,
        )
        .unwrap();

        assert!(nodes
            .iter()
            .any(|node| node.control_id == "QuestLogTitle" && node.text == "Quest Log"));
        assert!(
            view_prototype_store_file_cache()
                .lock()
                .expect("prototype cache mutex should not be poisoned")
                .len()
                > 0
        );
    }
}
