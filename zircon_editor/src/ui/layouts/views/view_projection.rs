use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::primitives::SharedString;
use crate::ui::v2_design_tokens::prepare_editor_v2_document;
use thiserror::Error;
use toml::Value;
use zircon_runtime::asset::runtime_asset_path_with_dev_asset_root;
use zircon_runtime::ui::style::resolve_button_style_from_values;
use zircon_runtime::ui::surface::{UiSurface, extract_ui_render_tree};
use zircon_runtime::ui::v2::{UiV2PrototypeStoreFileCache, UiV2SurfaceBuilder};
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    event_ui::UiTreeId,
    layout::UiSize,
    surface::{UiRenderCommand, UiRenderCommandKind, UiTextAlign},
    tree::{UiTemplateNodeMetadata, UiTreeError},
    v2::UiV2AssetError,
};

use super::{ViewTemplateFrameData, ViewTemplateNodeData, load_preview_image};

pub(crate) struct ViewTemplateVisualAssets {
    pub(crate) media_source: String,
    pub(crate) icon_name: String,
    pub(crate) preview_image: crate::ui::retained_host::primitives::Image,
    pub(crate) has_preview_image: bool,
}

#[derive(Debug, Error)]
pub enum ViewTemplateProjectionError {
    #[error("editor view projection requires v2 UI assets, got `{0}`")]
    NonV2AssetPath(String),
    #[error("view v2 store cache mutex poisoned")]
    V2StoreCachePoisoned,
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
    Err(ViewTemplateProjectionError::NonV2AssetPath(
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
        .map_err(|_| ViewTemplateProjectionError::V2StoreCachePoisoned)?
        .load_store(v2_source_paths(layout_asset_path, style_imports))?;
    let document = prepare_editor_v2_document(outcome.root_document.as_ref());
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new(document_tree_id.to_string()),
        &document,
        outcome.compiled.as_ref(),
    )?;
    surface.compute_layout(size)?;

    Ok(view_template_nodes_from_surface(&surface, text_overrides))
}

fn is_v2_asset_path(path: &str) -> bool {
    path.ends_with(".zui")
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
    let component_owned_text_control_ids =
        component_owned_text_control_ids(surface, &render.list.commands);
    let component_owned_frame_by_control_id =
        component_owned_frame_by_control_id(surface, &render.list.commands);
    let component_owned_text_by_control_id = component_owned_text_by_control_id(
        surface,
        &render.list.commands,
        &component_owned_text_control_ids,
    );
    let mut emitted_component_owned_control_ids = BTreeSet::new();
    let mut nodes = Vec::new();
    for command in render.list.commands {
        let Some(tree_node) = surface.tree.node(command.node_id) else {
            continue;
        };
        let Some(metadata) = tree_node.template_metadata.as_ref() else {
            continue;
        };
        if should_skip_component_owned_non_text_command(
            metadata,
            &command.kind,
            &component_owned_text_control_ids,
        ) {
            continue;
        }
        if should_skip_duplicate_component_owned_command(
            metadata,
            &mut emitted_component_owned_control_ids,
        ) {
            continue;
        }

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
            .or_else(|| string_attribute(metadata, "placeholder"))
            .or_else(|| component_owned_text_by_control_id.get(&control_id).cloned())
            .or(command.text.clone())
            .unwrap_or_default();
        let text = icon_button_hides_label(metadata)
            .then(String::new)
            .unwrap_or(text);
        let component_role = resolve_component_role(&metadata.component);
        let component_variant = resolve_component_variant(metadata);
        let binding_id = preferred_binding_id(metadata, None).unwrap_or_default();
        let edit_action_id = resolve_edit_action_id(metadata, component_role, &binding_id);
        let commit_action_id = resolve_commit_action_id(metadata);
        let value_text = resolve_node_value_text(metadata, &text, component_role);
        let value_number = resolve_node_value_number(metadata);
        let value_percent = resolve_node_value_percent(metadata, component_role, value_number);
        let options = string_array_attribute(metadata, "options");
        let visual_assets = resolve_visual_assets(metadata);
        let button_style = resolve_button_style_from_values(&metadata.style_overrides);
        let popup_open = resolve_node_popup_open(metadata);
        let transition_kind = resolve_transition_kind(metadata, component_role);
        let transition_in =
            resolve_transition_in(metadata, !transition_kind.is_empty(), popup_open);
        let transition_status = string_attribute(metadata, "transition_status")
            .unwrap_or_else(|| if transition_in { "entered" } else { "exited" }.to_string());
        let transition_progress =
            resolve_transition_progress(metadata, transition_status.as_str(), transition_in);
        let frame = component_owned_frame_for_command(
            metadata,
            &command.kind,
            &control_id,
            &command,
            &component_owned_frame_by_control_id,
        );

        nodes.push(ViewTemplateNodeData {
            node_id: tree_node.node_path.0.clone().into(),
            control_id: control_id.into(),
            role: SharedString::from(role),
            text: text.into(),
            component_role: component_role.into(),
            component_variant: component_variant.into(),
            value_text: value_text.into(),
            value_number,
            value_percent,
            options: model_rc(options.into_iter().map(SharedString::from).collect()),
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
            z_index: integer_attribute(metadata, "z_index").unwrap_or(command.z_index),
            transition_kind: transition_kind.clone().into(),
            transition_in,
            transition_entered: bool_attribute(metadata, "transition_entered")
                .or_else(|| bool_attribute(metadata, "entered"))
                .unwrap_or_else(|| {
                    transition_in && transition_status == "entered" && transition_progress >= 1.0
                }),
            transition_progress,
            transition_duration_ms: integer_attribute(metadata, "transition_duration_ms")
                .or_else(|| integer_attribute(metadata, "timeout_ms"))
                .or_else(|| integer_attribute(metadata, "duration_ms"))
                .unwrap_or_else(|| default_transition_duration_ms(&transition_kind, transition_in)),
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
            media_source: visual_assets.media_source.into(),
            icon_name: visual_assets.icon_name.into(),
            has_preview_image: visual_assets.has_preview_image,
            preview_image: visual_assets.preview_image,
            frame,
        });
    }

    nodes
}

fn component_owned_text_control_ids(
    surface: &UiSurface,
    commands: &[UiRenderCommand],
) -> BTreeSet<String> {
    commands
        .iter()
        .filter(|command| matches!(command.kind, UiRenderCommandKind::Text))
        .filter_map(|command| {
            let metadata = surface
                .tree
                .node(command.node_id)?
                .template_metadata
                .as_ref()?;
            component_owns_text_paint(metadata)
                .then(|| metadata.control_id.clone())
                .flatten()
        })
        .collect()
}

fn component_owned_frame_by_control_id(
    surface: &UiSurface,
    commands: &[UiRenderCommand],
) -> BTreeMap<String, ViewTemplateFrameData> {
    let mut frames = BTreeMap::new();
    for command in commands
        .iter()
        .filter(|command| !matches!(command.kind, UiRenderCommandKind::Text))
    {
        let Some(metadata) = surface
            .tree
            .node(command.node_id)
            .and_then(|node| node.template_metadata.as_ref())
        else {
            continue;
        };
        if !component_owns_text_paint(metadata) {
            continue;
        }
        let Some(control_id) = metadata.control_id.as_ref() else {
            continue;
        };
        let candidate = ViewTemplateFrameData {
            x: command.frame.x,
            y: command.frame.y,
            width: command.frame.width,
            height: command.frame.height,
        };
        let replace_existing = frames
            .get(control_id)
            .is_none_or(|existing| frame_area(&candidate) > frame_area(existing));
        if replace_existing {
            let _ = frames.insert(control_id.clone(), candidate);
        }
    }
    frames
}

fn component_owned_text_by_control_id(
    surface: &UiSurface,
    commands: &[UiRenderCommand],
    component_owned_text_control_ids: &BTreeSet<String>,
) -> BTreeMap<String, String> {
    commands
        .iter()
        .filter(|command| matches!(command.kind, UiRenderCommandKind::Text))
        .filter_map(|command| {
            let metadata = surface
                .tree
                .node(command.node_id)?
                .template_metadata
                .as_ref()?;
            let control_id = metadata.control_id.as_ref()?;
            if !component_owned_text_control_ids.contains(control_id) {
                return None;
            }
            let text = command
                .text
                .clone()
                .or_else(|| string_attribute(metadata, "label"))
                .or_else(|| string_attribute(metadata, "text"))?;
            (!text.trim().is_empty()).then(|| (control_id.clone(), text))
        })
        .collect()
}

fn should_skip_component_owned_non_text_command(
    metadata: &UiTemplateNodeMetadata,
    kind: &UiRenderCommandKind,
    component_owned_text_control_ids: &BTreeSet<String>,
) -> bool {
    !matches!(kind, UiRenderCommandKind::Text)
        && metadata
            .control_id
            .as_ref()
            .is_some_and(|control_id| component_owned_text_control_ids.contains(control_id))
        && component_owns_text_paint(metadata)
}

fn component_owned_frame_for_command(
    metadata: &UiTemplateNodeMetadata,
    kind: &UiRenderCommandKind,
    control_id: &str,
    command: &UiRenderCommand,
    component_owned_frame_by_control_id: &BTreeMap<String, ViewTemplateFrameData>,
) -> ViewTemplateFrameData {
    if matches!(kind, UiRenderCommandKind::Text) && component_owns_text_paint(metadata) {
        if let Some(frame) = component_owned_frame_by_control_id.get(control_id) {
            return frame.clone();
        }
    }

    ViewTemplateFrameData {
        x: command.frame.x,
        y: command.frame.y,
        width: command.frame.width,
        height: command.frame.height,
    }
}

fn frame_area(frame: &ViewTemplateFrameData) -> f32 {
    frame.width.max(0.0) * frame.height.max(0.0)
}

fn should_skip_duplicate_component_owned_command(
    metadata: &UiTemplateNodeMetadata,
    emitted_component_owned_control_ids: &mut BTreeSet<String>,
) -> bool {
    if !component_owns_text_paint(metadata) {
        return false;
    }
    let Some(control_id) = metadata
        .control_id
        .as_ref()
        .filter(|control_id| !control_id.is_empty())
    else {
        return false;
    };

    !emitted_component_owned_control_ids.insert(control_id.clone())
}

fn component_owns_text_paint(metadata: &UiTemplateNodeMetadata) -> bool {
    match metadata.component.as_str() {
        "IconButton" => icon_button_hides_label(metadata),
        "Button" | "EditableTable" | "InputField" | "NumberField" | "Table" | "TextField" => true,
        _ => false,
    }
}

fn icon_button_hides_label(metadata: &UiTemplateNodeMetadata) -> bool {
    metadata.component == "IconButton"
        && string_attribute(metadata, "icon_placement")
            .is_some_and(|placement| placement.eq_ignore_ascii_case("icon_only"))
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
        "RangeField" | "Slider" | "RangeSlider" => "RangeField",
        "Progress" | "ProgressBar" | "LinearProgress" | "CircularProgress" | "Spinner" => {
            "Progress"
        }
        "Skeleton" => "Skeleton",
        "Backdrop" => "Backdrop",
        "Paper" | "Dialog" | "AlertDialog" | "Popover" | "Popper" | "Tooltip" | "Snackbar"
        | "Menu" | "Drawer" => "Panel",
        "Toggle" | "ToggleButton" | "Switch" | "Checkbox" | "Radio" | "RadioField" => "Toggle",
        "SegmentedControl" | "Tab" | "Tabs" | "TabList" => "Toggle",
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
        "Slider" => "slider",
        "RangeSlider" => "range-slider",
        "Progress" => "progress",
        "ProgressBar" => "progress-bar",
        "LinearProgress" => "linear-progress",
        "CircularProgress" => "circular-progress",
        "Spinner" => "spinner",
        "Divider" | "Separator" => "divider",
        "Skeleton" => "skeleton",
        "Backdrop" => "backdrop",
        "Paper" => "paper",
        "Modal" => "modal",
        "Dialog" => "dialog",
        "AlertDialog" => "alert-dialog",
        "Popover" => "popover",
        "Popper" => "popper",
        "Tooltip" => "tooltip",
        "Snackbar" => "snackbar",
        "Menu" => "menu",
        "ContextMenu" => "context-menu",
        "ContextActionMenu" => "context-action-menu",
        "DropdownPopup" => "dropdown-popup",
        "Drawer" => "drawer",
        "Collapse" => "collapse",
        "Fade" => "fade",
        "Grow" => "grow",
        "Slide" => "slide",
        "Zoom" => "zoom",
        "Popup" => "popup",
        "Toggle" => "toggle",
        "ToggleButton" => "toggle-button",
        "Switch" => "switch",
        "Checkbox" => "checkbox",
        "SegmentedControl" => "segmented-control",
        "Tab" => "tab",
        "Tabs" => "tabs",
        "TabList" => "tab-list",
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

pub(crate) fn resolve_component_variant(metadata: &UiTemplateNodeMetadata) -> String {
    let mut variant = bool_attribute(metadata, "invisible")
        .filter(|invisible| *invisible)
        .map(|_| "invisible".to_string())
        .or_else(|| string_attribute(metadata, "mui_variant"))
        .or_else(|| string_attribute(metadata, "component_variant"))
        .or_else(|| string_attribute(metadata, "variant"))
        .unwrap_or_default();

    if let Some(animation) = string_attribute(metadata, "animation") {
        if !animation.is_empty() && !variant.split_whitespace().any(|part| part == animation) {
            if variant.is_empty() {
                variant = animation;
            } else {
                variant.push(' ');
                variant.push_str(&animation);
            }
        }
    }

    if resolve_component_role(metadata.component.as_str()) == "divider" {
        if let Some(orientation) = string_attribute(metadata, "orientation") {
            append_component_variant_token(&mut variant, &orientation);
        }
        if bool_attribute(metadata, "flexItem")
            .or_else(|| bool_attribute(metadata, "flex_item"))
            .unwrap_or(false)
        {
            append_component_variant_token(&mut variant, "flexItem");
        }
        if string_attribute(metadata, "text")
            .or_else(|| string_attribute(metadata, "label"))
            .is_some_and(|value| !value.is_empty())
        {
            append_component_variant_token(&mut variant, "withChildren");
        }
        if let Some(text_align) = string_attribute(metadata, "textAlign")
            .or_else(|| string_attribute(metadata, "text_align"))
        {
            if matches!(text_align.as_str(), "left" | "right") {
                append_component_variant_token(
                    &mut variant,
                    &format!("textAlign{}", pascal_case(&text_align)),
                );
            }
        }
    }

    if resolve_component_role(metadata.component.as_str()) == "input-field" {
        if variant.is_empty() {
            variant = "outlined".to_string();
        }
        if bool_attribute(metadata, "focused").unwrap_or(false) {
            append_component_variant_token(&mut variant, "focused");
        }
        if bool_attribute(metadata, "error").unwrap_or(false)
            || string_attribute(metadata, "validation_level")
                .is_some_and(|level| matches!(level.as_str(), "error" | "danger"))
        {
            append_component_variant_token(&mut variant, "error");
        }
        if let Some(size) = string_attribute(metadata, "size") {
            append_component_variant_token(&mut variant, &size);
        }
    }

    variant
}

fn append_component_variant_token(variant: &mut String, token: &str) {
    if token.is_empty()
        || variant
            .split_whitespace()
            .any(|part| part.eq_ignore_ascii_case(token))
    {
        return;
    }
    if !variant.is_empty() {
        variant.push(' ');
    }
    variant.push_str(token);
}

fn pascal_case(value: &str) -> String {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return String::new();
    };
    first.to_ascii_uppercase().to_string() + characters.as_str()
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

pub(crate) fn resolve_node_value_number(metadata: &UiTemplateNodeMetadata) -> f32 {
    number_attribute(metadata, "value")
        .or_else(|| number_attribute(metadata, "progress"))
        .unwrap_or(0.0)
}

pub(crate) fn resolve_node_value_percent(
    metadata: &UiTemplateNodeMetadata,
    component_role: &str,
    value_number: f32,
) -> f32 {
    if let Some(value_percent) = number_attribute(metadata, "value_percent")
        .or_else(|| number_attribute(metadata, "progress_percent"))
    {
        return normalize_percent_literal(value_percent);
    }

    let value = number_attribute(metadata, "progress")
        .or_else(|| number_attribute(metadata, "value"))
        .unwrap_or(value_number);
    let min = number_attribute(metadata, "min");
    let max = number_attribute(metadata, "max");
    match (min, max) {
        (Some(min), Some(max)) if max > min => ((value - min) / (max - min)).clamp(0.0, 1.0),
        _ if is_progress_component_role(component_role) && value > 1.0 => {
            normalize_percent_literal(value)
        }
        _ => value.clamp(0.0, 1.0),
    }
}

pub(crate) fn resolve_node_popup_open(metadata: &UiTemplateNodeMetadata) -> bool {
    bool_attribute(metadata, "popup_open")
        .or_else(|| bool_attribute(metadata, "open"))
        .unwrap_or(false)
}

pub(crate) fn resolve_transition_kind(
    metadata: &UiTemplateNodeMetadata,
    component_role: &str,
) -> String {
    string_attribute(metadata, "transition_kind")
        .or_else(|| string_attribute(metadata, "transition"))
        .or_else(|| match component_role {
            "collapse" | "fade" | "grow" | "slide" | "zoom" => Some(component_role.to_string()),
            _ => None,
        })
        .unwrap_or_default()
}

pub(crate) fn resolve_transition_in(
    metadata: &UiTemplateNodeMetadata,
    has_transition: bool,
    popup_open: bool,
) -> bool {
    bool_attribute(metadata, "transition_in")
        .or_else(|| bool_attribute(metadata, "in"))
        .unwrap_or_else(|| {
            if has_transition {
                popup_open || bool_attribute(metadata, "open").unwrap_or(true)
            } else {
                true
            }
        })
}

pub(crate) fn resolve_transition_progress(
    metadata: &UiTemplateNodeMetadata,
    status: &str,
    transition_in: bool,
) -> f32 {
    number_attribute(metadata, "transition_progress")
        .or_else(|| number_attribute(metadata, "animation_progress"))
        .map(|value| value.clamp(0.0, 1.0))
        .unwrap_or_else(|| match status {
            "entering" | "exiting" => 0.5,
            "entered" => 1.0,
            "exited" => 0.0,
            _ if transition_in => 1.0,
            _ => 0.0,
        })
}

pub(crate) fn default_transition_duration_ms(transition_kind: &str, transition_in: bool) -> i32 {
    match transition_kind {
        "collapse" => 300,
        "fade" | "grow" | "slide" | "zoom" if transition_in => 225,
        "fade" | "grow" | "slide" | "zoom" => 195,
        _ => 0,
    }
}

pub(crate) fn default_transition_easing(
    transition_kind: &str,
    transition_in: bool,
) -> &'static str {
    match (transition_kind, transition_in) {
        ("slide", true) => "cubic-bezier(0.0, 0, 0.2, 1)",
        ("slide", false) => "cubic-bezier(0.4, 0, 0.6, 1)",
        _ => "cubic-bezier(0.4, 0, 0.2, 1)",
    }
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

fn normalize_percent_literal(value: f32) -> f32 {
    if value > 1.0 {
        (value / 100.0).clamp(0.0, 1.0)
    } else {
        value.clamp(0.0, 1.0)
    }
}

fn is_progress_component_role(component_role: &str) -> bool {
    matches!(
        component_role,
        "progress" | "progress-bar" | "linear-progress" | "circular-progress" | "spinner"
    )
}

fn string_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<String> {
    metadata
        .attributes
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn string_array_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Vec<String> {
    metadata
        .attributes
        .get(key)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
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
        UiTextAlign::Start => "start",
        UiTextAlign::End => "end",
        UiTextAlign::Justify => "justify",
    }
}

#[cfg(test)]
mod tests;
