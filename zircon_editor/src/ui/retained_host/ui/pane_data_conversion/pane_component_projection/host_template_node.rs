use crate::ui::retained_host as host_contract;
use crate::ui::template_runtime::RetainedUiHostNodeProjection;

use super::super::super::component_contract_metadata::descriptor_for_component;
use super::clip_frame::projected_clip_frame;
use super::collection_projection::projected_collection;
use super::drag_overlay::projected_drag_overlay_data;
use super::popup_actions::projected_popup_actions;
use super::selection_options::projected_selection_options;
use super::template_node_data::{template_pane_node_data, ProjectedTemplateNodeParts};
use super::text_layout::projected_text_layout;
use super::validation_state::projected_validation_state;
use super::value_media::projected_value_media;
use super::visual_state::projected_visual_state;
use super::visual_style::projected_visual_style;
use super::world_space::projected_world_space;

pub(in super::super) fn host_template_node(
    node: RetainedUiHostNodeProjection,
) -> Option<host_contract::TemplatePaneNodeData> {
    let control_id = node.control_id?;
    let component = node.component.clone();
    let component_descriptor = descriptor_for_component(&component);
    let component_role = component_descriptor
        .map(|descriptor| descriptor.role.clone())
        .filter(|role| !role.is_empty())
        .or_else(|| {
            let role = crate::ui::layouts::views::resolve_component_role(&component);
            (!role.is_empty()).then(|| role.to_string())
        })
        .unwrap_or_default();
    let drag_overlay = projected_drag_overlay_data(component_role.as_str(), &node.attributes);
    let value_media =
        projected_value_media(component_role.as_str(), &node.attributes, &drag_overlay);
    let validation_state = projected_validation_state(
        &node.attributes,
        component_role.as_str(),
        component_descriptor.is_some(),
    );
    let selection_options = projected_selection_options(component_role.as_str(), &node.attributes);
    let collection_projection = projected_collection(&component, &node.attributes, &node.bindings);
    let world_space = projected_world_space(&component, &node.attributes);
    let popup_actions = projected_popup_actions(
        &control_id,
        component_role.as_str(),
        &node.attributes,
        &node.bindings,
        component_descriptor,
        &drag_overlay,
        validation_state.disabled,
        node.frame.x,
        node.frame.y,
        node.frame.width,
        node.frame.height,
    );
    let text_layout = projected_text_layout(
        &control_id,
        component_role.as_str(),
        &node.attributes,
        !node.bindings.is_empty(),
        &drag_overlay,
    );
    let visual_style = projected_visual_style(
        &component,
        component_role.as_str(),
        &node.attributes,
        node.z_index,
        popup_actions.popup_open,
    );
    let visual_state = projected_visual_state(&node.attributes);
    let clip_frame = projected_clip_frame(node.clip_frame.as_ref());

    Some(template_pane_node_data(ProjectedTemplateNodeParts {
        node_id: node.node_id,
        control_id,
        role: component,
        component_role,
        text_layout,
        value_media,
        validation_state,
        selection_options,
        collection: collection_projection,
        world_space,
        popup_actions,
        drag_overlay,
        visual_state,
        visual_style,
        clip_frame,
    }))
}
