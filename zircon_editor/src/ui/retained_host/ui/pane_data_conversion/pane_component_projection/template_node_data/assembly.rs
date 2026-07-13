use crate::ui::retained_host as host_contract;

use super::content::assign_content_fields;
use super::identity::assign_identity_fields;
use super::interaction::assign_interaction_fields;
use super::options_collection::assign_options_collection_fields;
use super::parts::ProjectedTemplateNodeParts;
use super::sample_grid::assign_sample_grid_fields;
use super::spatial::assign_spatial_fields;
use super::timeline_strip::assign_timeline_strip_fields;
use super::visual::assign_visual_fields;
use super::weight_heatmap::assign_weight_heatmap_fields;

pub(in super::super) fn template_pane_node_data(
    parts: ProjectedTemplateNodeParts,
) -> host_contract::TemplatePaneNodeData {
    let ProjectedTemplateNodeParts {
        node_id,
        control_id,
        role,
        component_role,
        text_layout,
        value_media,
        validation_state,
        selection_options,
        sample_grid,
        timeline_strip,
        weight_heatmap,
        collection,
        world_space,
        popup_actions,
        drag_overlay,
        visual_state,
        visual_style,
        clip_frame,
    } = parts;

    let mut node = host_contract::TemplatePaneNodeData::default();
    assign_identity_fields(&mut node, node_id, control_id, role, component_role);
    assign_content_fields(&mut node, text_layout, value_media);
    assign_options_collection_fields(&mut node, selection_options, collection);
    assign_sample_grid_fields(&mut node, sample_grid);
    assign_timeline_strip_fields(&mut node, timeline_strip);
    assign_weight_heatmap_fields(&mut node, weight_heatmap);
    assign_spatial_fields(&mut node, world_space, clip_frame);
    assign_interaction_fields(&mut node, popup_actions, drag_overlay);
    assign_visual_fields(&mut node, validation_state, visual_state, visual_style);
    node
}
