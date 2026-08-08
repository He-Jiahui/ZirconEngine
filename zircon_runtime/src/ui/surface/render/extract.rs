use std::collections::{BTreeMap, BTreeSet};

use crate::ui::surface::{
    build_arranged_tree, component_state::UiSurfaceComponentStateStore, is_arranged_render_visible,
};
use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::surface::UiArrangedTree;
use zircon_runtime_interface::ui::surface::{UiRenderCommand, UiRenderExtract, UiRenderList};
use zircon_runtime_interface::ui::tree::UiTree;

use super::buttons::{
    button_render_commands, button_suppresses_owner_image, button_suppresses_owner_text,
};
use super::chrome::{
    chrome_render_commands, chrome_suppresses_owner_image, chrome_suppresses_owner_surface,
    chrome_suppresses_owner_text,
};
use super::collection_rows::{
    collection_row_render_commands, collection_row_suppresses_owner_image,
    collection_row_suppresses_owner_surface, collection_row_suppresses_owner_text,
};
use super::command_palette::{
    command_palette_render_commands, command_palette_suppresses_owner_image,
    command_palette_suppresses_owner_surface, command_palette_suppresses_owner_text,
};
use super::dialog::{
    dialog_render_commands, dialog_suppresses_owner_image, dialog_suppresses_owner_surface,
    dialog_suppresses_owner_text,
};
use super::divider::{
    divider_render_commands, divider_suppresses_owner_image, divider_suppresses_owner_surface,
    divider_suppresses_owner_text,
};
use super::drag_overlay::{
    drag_overlay_render_commands, drag_overlay_suppresses_owner_image,
    drag_overlay_suppresses_owner_surface, drag_overlay_suppresses_owner_text,
};
use super::dropdowns::{dropdown_render_commands, dropdown_suppresses_owner_text};
use super::feedback::{
    feedback_render_commands, feedback_suppresses_owner_image, feedback_suppresses_owner_surface,
    feedback_suppresses_owner_text,
};
use super::node_visual_data::UiNodeVisualData;
use super::notification_center::{
    notification_center_render_commands, notification_center_suppresses_owner_image,
    notification_center_suppresses_owner_surface, notification_center_suppresses_owner_text,
};
use super::popup_menu::popup_menu_render_commands;
use super::popup_options::popup_option_render_commands;
use super::progress::{
    progress_render_commands, progress_suppresses_owner_image, progress_suppresses_owner_surface,
    progress_suppresses_owner_text,
};
use super::resolve::resolve_command_kind;
use super::segmented_controls::{
    segmented_control_render_commands, segmented_control_suppresses_owner_text,
};
use super::selection_controls::{
    selection_control_render_commands, selection_control_suppresses_owner_text,
};
use super::skeleton::{
    skeleton_render_commands, skeleton_suppresses_owner_image, skeleton_suppresses_owner_surface,
    skeleton_suppresses_owner_text,
};
use super::sliders::{slider_render_commands, slider_suppresses_owner_text};
use super::text_fields::{text_field_render_commands, text_field_suppresses_owner_text};
use super::text_prewarm::{
    prewarm_render_command_text, resolve_missing_render_command_text_layouts,
};
use crate::text::TextDocumentKey;
use crate::ui::text::{
    prepare_render_command_text_artifacts, resolve_text_layout, UiTextLayoutRequest,
    UiTextLayoutResolution, UiTextMeasureCache, UiTextViewport,
};

pub fn extract_ui_render_tree(tree: &UiTree) -> UiRenderExtract {
    let arranged_tree = build_arranged_tree(tree);
    extract_ui_render_tree_from_arranged(tree, &arranged_tree)
}

pub fn extract_ui_render_tree_from_arranged(
    tree: &UiTree,
    arranged_tree: &UiArrangedTree,
) -> UiRenderExtract {
    extract_ui_render_tree_from_arranged_with_component_states(tree, arranged_tree, None)
}

pub(crate) fn extract_ui_render_tree_from_arranged_with_component_states(
    tree: &UiTree,
    arranged_tree: &UiArrangedTree,
    component_states: Option<&UiSurfaceComponentStateStore>,
) -> UiRenderExtract {
    extract_ui_render_tree_from_arranged_with_component_states_and_text_measure_cache(
        tree,
        arranged_tree,
        component_states,
        None,
    )
}

pub(crate) fn extract_ui_render_tree_from_arranged_with_component_states_and_text_measure_cache(
    tree: &UiTree,
    arranged_tree: &UiArrangedTree,
    component_states: Option<&UiSurfaceComponentStateStore>,
    mut text_measure_cache: Option<&mut UiTextMeasureCache>,
) -> UiRenderExtract {
    let mut commands: Vec<UiRenderCommand> = arranged_tree
        .draw_order
        .iter()
        .copied()
        .into_iter()
        .flat_map(|node_id| {
            let Some(node) = tree.nodes.get(&node_id) else {
                return Vec::new();
            };
            let Some(arranged_node) = arranged_tree.get(node_id) else {
                return Vec::new();
            };
            if !is_arranged_render_visible(arranged_tree, node_id).unwrap_or(false) {
                return Vec::new();
            }
            let component_state = component_states.and_then(|states| states.get(node_id));
            let visual = UiNodeVisualData::resolve(
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
            );
            let owner_text =
                if selection_control_suppresses_owner_text(node.template_metadata.as_ref())
                    || slider_suppresses_owner_text(node.template_metadata.as_ref())
                    || dropdown_suppresses_owner_text(node.template_metadata.as_ref())
                    || text_field_suppresses_owner_text(node.template_metadata.as_ref())
                    || button_suppresses_owner_text(node.template_metadata.as_ref())
                    || segmented_control_suppresses_owner_text(node.template_metadata.as_ref())
                    || progress_suppresses_owner_text(node.template_metadata.as_ref())
                    || divider_suppresses_owner_text(node.template_metadata.as_ref())
                    || skeleton_suppresses_owner_text(node.template_metadata.as_ref())
                    || collection_row_suppresses_owner_text(node.template_metadata.as_ref())
                    || feedback_suppresses_owner_text(node.template_metadata.as_ref())
                    || dialog_suppresses_owner_text(node.template_metadata.as_ref())
                    || command_palette_suppresses_owner_text(node.template_metadata.as_ref())
                    || notification_center_suppresses_owner_text(node.template_metadata.as_ref())
                    || drag_overlay_suppresses_owner_text(node.template_metadata.as_ref())
                    || chrome_suppresses_owner_text(node.template_metadata.as_ref())
                {
                    None
                } else {
                    visual.text.clone()
                };
            let owner_image = if button_suppresses_owner_image(node.template_metadata.as_ref())
                || collection_row_suppresses_owner_image(node.template_metadata.as_ref())
                || progress_suppresses_owner_image(node.template_metadata.as_ref())
                || divider_suppresses_owner_image(node.template_metadata.as_ref())
                || skeleton_suppresses_owner_image(node.template_metadata.as_ref())
                || feedback_suppresses_owner_image(node.template_metadata.as_ref())
                || dialog_suppresses_owner_image(node.template_metadata.as_ref())
                || command_palette_suppresses_owner_image(node.template_metadata.as_ref())
                || notification_center_suppresses_owner_image(node.template_metadata.as_ref())
                || drag_overlay_suppresses_owner_image(node.template_metadata.as_ref())
                || chrome_suppresses_owner_image(node.template_metadata.as_ref())
            {
                None
            } else {
                visual.image.clone()
            };
            let owner_style =
                if collection_row_suppresses_owner_surface(node.template_metadata.as_ref())
                    || progress_suppresses_owner_surface(node.template_metadata.as_ref())
                    || divider_suppresses_owner_surface(node.template_metadata.as_ref())
                    || skeleton_suppresses_owner_surface(node.template_metadata.as_ref())
                    || feedback_suppresses_owner_surface(node.template_metadata.as_ref())
                    || dialog_suppresses_owner_surface(node.template_metadata.as_ref())
                    || command_palette_suppresses_owner_surface(node.template_metadata.as_ref())
                    || notification_center_suppresses_owner_surface(node.template_metadata.as_ref())
                    || drag_overlay_suppresses_owner_surface(node.template_metadata.as_ref())
                    || chrome_suppresses_owner_surface(node.template_metadata.as_ref())
                {
                    let mut style = visual.style.clone();
                    style.background_color = None;
                    style.border_color = None;
                    style.border_width = 0.0;
                    style.corner_radius = 0.0;
                    style
                } else {
                    visual.style.clone()
                };

            let text_layout = owner_text.as_deref().map(|text| {
                let mut request = UiTextLayoutRequest::new(
                    text,
                    &owner_style,
                    arranged_node.frame,
                    Some(arranged_node.clip_frame),
                )
                .with_document_key(TextDocumentKey::new(
                    node_id.0,
                    node.layout_cache.text_layout_revision,
                ));
                if visual.editable.is_none() {
                    if let Some(viewport) = UiTextViewport::from_document_and_clip(
                        arranged_node.frame,
                        arranged_node.clip_frame,
                    ) {
                        request = request.with_viewport(viewport);
                    }
                }
                let mut layout =
                    resolve_text_layout_with_cache(&request, text_measure_cache.as_deref_mut())
                        .layout;
                layout.editable = visual.editable.clone();
                layout
            });
            let command = UiRenderCommand {
                node_id,
                kind: resolve_command_kind(&owner_style, owner_text.as_ref(), owner_image.as_ref()),
                frame: arranged_node.frame,
                clip_frame: Some(arranged_node.clip_frame),
                z_index: arranged_node.z_index,
                style: owner_style,
                text_layout,
                text: owner_text,
                image: owner_image,
                opacity: visual.opacity,
            };
            let mut commands = vec![command];
            commands.extend(button_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(selection_control_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(segmented_control_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(slider_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(dropdown_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(text_field_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
                &visual.style,
                visual.text.as_deref(),
                visual.editable.as_ref(),
                text_measure_cache.as_deref_mut(),
            ));
            commands.extend(collection_row_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(feedback_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(progress_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(divider_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(skeleton_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(dialog_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(command_palette_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(notification_center_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(drag_overlay_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(chrome_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(popup_menu_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(popup_option_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands
        })
        .collect();

    if let Some(cache) = text_measure_cache.as_deref_mut() {
        prewarm_render_command_text(&commands, cache);
    }
    resolve_missing_render_command_text_layouts(&mut commands, text_measure_cache.as_deref_mut());
    prepare_render_command_text_artifacts(&mut commands);

    UiRenderExtract {
        tree_id: tree.tree_id.clone(),
        list: UiRenderList { commands },
        raster_scale: 1.0,
    }
}

pub(crate) fn extract_ui_render_commands_for_nodes_with_component_states_and_text_measure_cache(
    tree: &UiTree,
    arranged_tree: &UiArrangedTree,
    arranged_node_indices: &BTreeMap<UiNodeId, usize>,
    changed_node_ids: &BTreeSet<UiNodeId>,
    component_states: Option<&UiSurfaceComponentStateStore>,
    text_measure_cache: Option<&mut UiTextMeasureCache>,
) -> Result<UiRenderExtract, ()> {
    let mut partial_nodes = Vec::new();
    let mut included = BTreeSet::new();
    for node_id in changed_node_ids {
        let mut current = Some(*node_id);
        while let Some(current_id) = current {
            let Some(index) = arranged_node_indices.get(&current_id).copied() else {
                return Err(());
            };
            let Some(node) = arranged_tree.nodes.get(index) else {
                return Err(());
            };
            if node.node_id != current_id || tree.node(current_id).is_none() {
                return Err(());
            }
            current = node.parent;
            if included.insert(current_id) {
                partial_nodes.push(node.clone());
            }
        }
    }
    let partial_tree = UiArrangedTree {
        tree_id: arranged_tree.tree_id.clone(),
        roots: Vec::new(),
        nodes: partial_nodes,
        draw_order: changed_node_ids.iter().copied().collect(),
        canvas_layers: Vec::new(),
    };
    Ok(
        extract_ui_render_tree_from_arranged_with_component_states_and_text_measure_cache(
            tree,
            &partial_tree,
            component_states,
            text_measure_cache,
        ),
    )
}

pub(super) fn resolve_text_layout_with_cache(
    request: &UiTextLayoutRequest<'_>,
    text_measure_cache: Option<&mut UiTextMeasureCache>,
) -> UiTextLayoutResolution {
    match text_measure_cache {
        Some(cache) => cache.resolve_or_shape(request),
        None => resolve_text_layout(request),
    }
}
