use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime::ui::style::resolve_button_style_from_values;
use zircon_runtime::ui::surface::{extract_ui_render_tree, UiSurface};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    surface::{UiRenderCommand, UiRenderCommandKind},
    tree::UiTemplateNodeMetadata,
};

use super::super::{ViewTemplateFrameData, ViewTemplateNodeData};
use super::{
    bool_attribute, default_transition_duration_ms, default_transition_easing, integer_attribute,
    number_attribute, preferred_binding_id, resolve_commit_action_id, resolve_component_role,
    resolve_component_variant, resolve_edit_action_id, resolve_node_popup_open,
    resolve_node_value_number, resolve_node_value_percent, resolve_role, resolve_transition_in,
    resolve_transition_kind, resolve_transition_progress, resolve_visual_assets_for_generation,
    string_array_attribute, string_attribute, text_align_name, text_binding_for_metadata,
    ViewTemplateTextBinding, ViewTemplateTextOverrideSemantics,
};
use crate::ui::retained_host::primitives::SharedString;

pub(super) struct ViewTemplateNodeMaterialization {
    pub(super) nodes: Vec<ViewTemplateNodeData>,
    pub(super) text_override_semantics: Vec<ViewTemplateTextOverrideSemantics>,
    pub(super) text_bindings: BTreeMap<String, ViewTemplateTextBinding>,
    pub(super) frame_source_node_ids: Vec<UiNodeId>,
    pub(super) row_signatures: Vec<ViewTemplateProjectionRowSignature>,
}

struct ViewTemplateFrameSource {
    node_id: UiNodeId,
    frame: ViewTemplateFrameData,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ViewTemplateProjectionRowSignature {
    pub(super) node_id: UiNodeId,
    pub(super) frame_source_node_id: UiNodeId,
    pub(super) command_kind: UiRenderCommandKind,
}

pub(super) fn view_template_nodes_from_surface(
    surface: &UiSurface,
    text_overrides: &BTreeMap<String, String>,
    resource_generation: u64,
) -> ViewTemplateNodeMaterialization {
    let render = extract_ui_render_tree(&surface.tree);
    let commands = &render.list.commands;
    let component_owned_text_control_ids = component_owned_text_control_ids(surface, commands);
    let component_owned_frame_by_control_id =
        component_owned_frame_by_control_id(surface, commands);
    let component_owned_text_by_control_id =
        component_owned_text_by_control_id(surface, commands, &component_owned_text_control_ids);
    let mut emitted_component_owned_control_ids = BTreeSet::new();
    let mut nodes = Vec::new();
    let mut text_override_semantics = Vec::new();
    let mut text_bindings = BTreeMap::new();
    let mut frame_source_node_ids = Vec::new();
    let mut row_signatures = Vec::new();

    for command in commands {
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
        ) || should_skip_duplicate_component_owned_command(
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
        let authored_requested_text = string_attribute(metadata, "label")
            .or_else(|| string_attribute(metadata, "text"))
            .or_else(|| string_attribute(metadata, "placeholder"))
            .or_else(|| component_owned_text_by_control_id.get(&control_id).cloned())
            .or(command.text.clone())
            .unwrap_or_default();
        let component_role = resolve_component_role(&metadata.component);
        let text_override_semantic = ViewTemplateTextOverrideSemantics::from_metadata(
            metadata,
            component_role,
            &authored_requested_text,
        );
        let requested_text = text_overrides.get(&control_id).map(String::as_str);
        let (text, value_text) = text_override_semantic.projected_text(requested_text);
        if !control_id.is_empty() {
            text_bindings.entry(control_id.clone()).or_insert_with(|| {
                text_binding_for_metadata(tree_node.node_id, metadata, component_role)
            });
        }
        let component_variant = resolve_component_variant(metadata);
        let binding_id = preferred_binding_id(metadata, None).unwrap_or_default();
        let edit_action_id = resolve_edit_action_id(metadata, component_role, &binding_id);
        let commit_action_id = resolve_commit_action_id(metadata);
        let value_number = resolve_node_value_number(metadata);
        let value_percent = resolve_node_value_percent(metadata, component_role, value_number);
        let options = string_array_attribute(metadata, "options");
        let visual_assets = resolve_visual_assets_for_generation(metadata, resource_generation);
        let button_style = resolve_button_style_from_values(&metadata.style_overrides);
        let popup_open = resolve_node_popup_open(metadata);
        let transition_kind = resolve_transition_kind(metadata, component_role);
        let transition_in =
            resolve_transition_in(metadata, !transition_kind.is_empty(), popup_open);
        let transition_status = string_attribute(metadata, "transition_status")
            .unwrap_or_else(|| if transition_in { "entered" } else { "exited" }.to_string());
        let transition_progress =
            resolve_transition_progress(metadata, transition_status.as_str(), transition_in);
        let frame_source = component_owned_frame_for_command(
            metadata,
            &command.kind,
            &control_id,
            command,
            &component_owned_frame_by_control_id,
        );
        text_override_semantics.push(text_override_semantic);
        frame_source_node_ids.push(frame_source.node_id);
        row_signatures.push(ViewTemplateProjectionRowSignature {
            node_id: tree_node.node_id,
            frame_source_node_id: frame_source.node_id,
            command_kind: command.kind,
        });

        nodes.push(ViewTemplateNodeData {
            node_id: tree_node.node_path.0.clone().into(),
            control_id: control_id.into(),
            role: role.into(),
            text: text.into(),
            component_role: component_role.into(),
            component_variant: component_variant.into(),
            value_text: value_text.into(),
            value_number,
            value_percent,
            options: crate::ui::layouts::common::model_rc(
                options.into_iter().map(SharedString::from).collect(),
            ),
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
            frame: frame_source.frame,
        });
    }
    ViewTemplateNodeMaterialization {
        nodes,
        text_override_semantics,
        text_bindings,
        frame_source_node_ids,
        row_signatures,
    }
}

pub(crate) fn view_template_projection_row_signatures(
    surface: &UiSurface,
) -> Vec<ViewTemplateProjectionRowSignature> {
    let render = extract_ui_render_tree(&surface.tree);
    let commands = &render.list.commands;
    let component_owned_text_control_ids = component_owned_text_control_ids(surface, commands);
    let component_owned_frame_by_control_id =
        component_owned_frame_by_control_id(surface, commands);
    let mut emitted_component_owned_control_ids = BTreeSet::new();
    let mut signatures = Vec::new();
    for command in commands {
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
        ) || should_skip_duplicate_component_owned_command(
            metadata,
            &mut emitted_component_owned_control_ids,
        ) || resolve_role(&metadata.component, &command.kind, metadata) == "Group"
        {
            continue;
        }
        let control_id = metadata.control_id.as_deref().unwrap_or_default();
        let frame_source = component_owned_frame_for_command(
            metadata,
            &command.kind,
            control_id,
            command,
            &component_owned_frame_by_control_id,
        );
        signatures.push(ViewTemplateProjectionRowSignature {
            node_id: tree_node.node_id,
            frame_source_node_id: frame_source.node_id,
            command_kind: command.kind,
        });
    }
    signatures
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
) -> BTreeMap<String, ViewTemplateFrameSource> {
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
        let candidate = ViewTemplateFrameSource {
            node_id: command.node_id,
            frame: ViewTemplateFrameData {
                x: command.frame.x,
                y: command.frame.y,
                width: command.frame.width,
                height: command.frame.height,
            },
        };
        if frames
            .get(control_id)
            .is_none_or(|existing: &ViewTemplateFrameSource| {
                frame_area(&candidate.frame) > frame_area(&existing.frame)
            })
        {
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
    component_owned_frame_by_control_id: &BTreeMap<String, ViewTemplateFrameSource>,
) -> ViewTemplateFrameSource {
    if matches!(kind, UiRenderCommandKind::Text) && component_owns_text_paint(metadata) {
        if let Some(frame_source) = component_owned_frame_by_control_id.get(control_id) {
            return ViewTemplateFrameSource {
                node_id: frame_source.node_id,
                frame: frame_source.frame.clone(),
            };
        }
    }
    ViewTemplateFrameSource {
        node_id: command.node_id,
        frame: ViewTemplateFrameData {
            x: command.frame.x,
            y: command.frame.y,
            width: command.frame.width,
            height: command.frame.height,
        },
    }
}

pub(crate) fn frame_area(frame: &ViewTemplateFrameData) -> f32 {
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

pub(super) fn component_owns_text_paint(metadata: &UiTemplateNodeMetadata) -> bool {
    match metadata.component.as_str() {
        "IconButton" => super::icon_button_hides_label(metadata),
        "Button" | "EditableTable" | "InputField" | "NumberField" | "Table" | "TextField" => true,
        _ => false,
    }
}
