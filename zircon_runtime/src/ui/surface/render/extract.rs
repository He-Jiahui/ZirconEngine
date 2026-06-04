use crate::ui::surface::{build_arranged_tree, is_arranged_render_visible};
use zircon_runtime_interface::ui::surface::UiArrangedTree;
use zircon_runtime_interface::ui::surface::{UiRenderCommand, UiRenderExtract, UiRenderList};
use zircon_runtime_interface::ui::tree::UiTree;

use super::buttons::{
    button_render_commands, button_suppresses_owner_image, button_suppresses_owner_text,
};
use super::collection_rows::{
    collection_row_render_commands, collection_row_suppresses_owner_image,
    collection_row_suppresses_owner_text,
};
use super::dropdowns::{dropdown_render_commands, dropdown_suppresses_owner_text};
use super::feedback::{
    feedback_render_commands, feedback_suppresses_owner_image, feedback_suppresses_owner_text,
};
use super::node_visual_data::UiNodeVisualData;
use super::popup_menu::popup_menu_render_commands;
use super::popup_options::popup_option_render_commands;
use super::resolve::resolve_command_kind;
use super::segmented_controls::{
    segmented_control_render_commands, segmented_control_suppresses_owner_text,
};
use super::selection_controls::{
    selection_control_render_commands, selection_control_suppresses_owner_text,
};
use super::sliders::{slider_render_commands, slider_suppresses_owner_text};
use super::text_fields::{text_field_render_commands, text_field_suppresses_owner_text};
use crate::ui::text::layout_text;

pub fn extract_ui_render_tree(tree: &UiTree) -> UiRenderExtract {
    let arranged_tree = build_arranged_tree(tree);
    extract_ui_render_tree_from_arranged(tree, &arranged_tree)
}

pub fn extract_ui_render_tree_from_arranged(
    tree: &UiTree,
    arranged_tree: &UiArrangedTree,
) -> UiRenderExtract {
    let commands = arranged_tree
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
            let visual =
                UiNodeVisualData::resolve(node.template_metadata.as_ref(), &node.state_flags);
            if !is_arranged_render_visible(arranged_tree, node_id).unwrap_or(false) {
                return Vec::new();
            }
            let owner_text =
                if selection_control_suppresses_owner_text(node.template_metadata.as_ref())
                    || slider_suppresses_owner_text(node.template_metadata.as_ref())
                    || dropdown_suppresses_owner_text(node.template_metadata.as_ref())
                    || text_field_suppresses_owner_text(node.template_metadata.as_ref())
                    || button_suppresses_owner_text(node.template_metadata.as_ref())
                    || segmented_control_suppresses_owner_text(node.template_metadata.as_ref())
                    || collection_row_suppresses_owner_text(node.template_metadata.as_ref())
                    || feedback_suppresses_owner_text(node.template_metadata.as_ref())
                {
                    None
                } else {
                    visual.text.clone()
                };
            let owner_image = if button_suppresses_owner_image(node.template_metadata.as_ref())
                || collection_row_suppresses_owner_image(node.template_metadata.as_ref())
                || feedback_suppresses_owner_image(node.template_metadata.as_ref())
            {
                None
            } else {
                visual.image.clone()
            };

            let text_layout = owner_text.as_deref().map(|text| {
                let mut layout = layout_text(
                    text,
                    &visual.style,
                    arranged_node.frame,
                    Some(arranged_node.clip_frame),
                );
                layout.editable = visual.editable.clone();
                layout
            });
            let command = UiRenderCommand {
                node_id,
                kind: resolve_command_kind(
                    &visual.style,
                    owner_text.as_ref(),
                    owner_image.as_ref(),
                ),
                frame: arranged_node.frame,
                clip_frame: Some(arranged_node.clip_frame),
                z_index: arranged_node.z_index,
                style: visual.style.clone(),
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
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(selection_control_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(segmented_control_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(slider_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(dropdown_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(text_field_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
                &visual.style,
                visual.text.as_deref(),
                visual.editable.as_ref(),
            ));
            commands.extend(collection_row_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(feedback_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
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

    UiRenderExtract {
        tree_id: tree.tree_id.clone(),
        list: UiRenderList { commands },
    }
}
