use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};
use crate::ui::retained_host as host_contract;
use crate::ui::retained_host::primitives::ModelRc;

fn map_model_rc<T, U, F>(model: &ModelRc<T>, mut map: F) -> ModelRc<U>
where
    T: Clone + 'static,
    U: Clone + 'static,
    F: FnMut(T) -> U,
{
    model_rc(
        (0..model.row_count())
            .filter_map(|row| model.row_data(row))
            .map(&mut map)
            .collect(),
    )
}

fn to_host_contract_template_frame(
    frame: &ViewTemplateFrameData,
) -> host_contract::TemplateNodeFrameData {
    host_contract::TemplateNodeFrameData {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    }
}

pub(crate) fn to_host_contract_template_node(
    data: &ViewTemplateNodeData,
) -> host_contract::TemplatePaneNodeData {
    host_contract::TemplatePaneNodeData {
        node_id: data.node_id.clone(),
        control_id: data.control_id.clone(),
        role: data.role.clone(),
        text: data.text.clone(),
        component_role: data.component_role.clone(),
        value_text: data.value_text.clone(),
        value_number: 0.0,
        value_percent: 0.0,
        value_color: crate::ui::retained_host::primitives::Color::from_argb_u8(0, 0, 0, 0),
        media_source: data.media_source.clone(),
        icon_name: data.icon_name.clone(),
        has_preview_image: data.has_preview_image,
        preview_image: data.preview_image.clone(),
        vector_components: ModelRc::default(),
        validation_level: "".into(),
        validation_message: "".into(),
        popup_open: false,
        has_popup_anchor: false,
        popup_anchor_x: 0.0,
        popup_anchor_y: 0.0,
        selection_state: "".into(),
        search_query: "".into(),
        selected: data.selected,
        tree_depth: 0,
        tree_indent_px: 0.0,
        options_text: "".into(),
        options: ModelRc::default(),
        structured_options: ModelRc::default(),
        collection_items: ModelRc::default(),
        collection_fields: ModelRc::default(),
        virtualization_enabled: false,
        virtualization_item_extent: 0.0,
        virtualization_overscan: 0,
        virtualization_total_count: 0,
        virtualization_visible_start: 0,
        virtualization_visible_count: 0,
        pagination_page_index: 0,
        pagination_page_size: 0,
        pagination_page_count: 0,
        pagination_total_count: 0,
        world_space_enabled: false,
        world_position_x: 0.0,
        world_position_y: 0.0,
        world_position_z: 0.0,
        world_rotation_x: 0.0,
        world_rotation_y: 0.0,
        world_rotation_z: 0.0,
        world_scale_x: 1.0,
        world_scale_y: 1.0,
        world_scale_z: 1.0,
        world_width: 0.0,
        world_height: 0.0,
        world_pixels_per_meter: 0.0,
        world_billboard: false,
        world_depth_test: false,
        world_render_order: 0,
        world_camera_target: "".into(),
        menu_items: ModelRc::default(),
        structured_menu_items: ModelRc::default(),
        actions: ModelRc::default(),
        accepted_drag_payloads: "".into(),
        drop_source_summary: "".into(),
        checked: false,
        expanded: false,
        focused: data.focused,
        hovered: data.hovered,
        pressed: data.pressed,
        dragging: false,
        enter_pressed: false,
        state_layer_enabled: false,
        state_layer_color: crate::ui::retained_host::primitives::Color::from_argb_u8(0, 0, 0, 0),
        ripple_enabled: false,
        ripple_pressed_x: 0.0,
        ripple_pressed_y: 0.0,
        ripple_unclipped: false,
        drop_hovered: false,
        active_drag_target: false,
        disabled: data.disabled,
        dispatch_kind: data.dispatch_kind.clone(),
        action_id: data.action_id.clone(),
        binding_id: data.binding_id.clone(),
        begin_drag_action_id: "".into(),
        drag_action_id: "".into(),
        end_drag_action_id: "".into(),
        commit_action_id: data.commit_action_id.clone(),
        edit_action_id: data.edit_action_id.clone(),
        surface_variant: data.surface_variant.clone(),
        text_tone: data.text_tone.clone(),
        button_variant: data.button_variant.clone(),
        button_style: data.button_style.clone(),
        font_size: data.font_size,
        font_weight: data.font_weight,
        text_align: data.text_align.clone(),
        overflow: data.overflow.clone(),
        corner_radius: data.corner_radius,
        border_width: data.border_width,
        elevation: 0.0,
        has_clip_frame: false,
        clip_frame: host_contract::TemplateNodeFrameData::default(),
        frame: to_host_contract_template_frame(&data.frame),
    }
}

pub(crate) fn to_host_contract_template_node_owned(
    data: ViewTemplateNodeData,
) -> host_contract::TemplatePaneNodeData {
    to_host_contract_template_node(&data)
}

pub(crate) fn to_host_contract_template_nodes(
    data: &ModelRc<ViewTemplateNodeData>,
) -> ModelRc<host_contract::TemplatePaneNodeData> {
    map_model_rc(data, |node| to_host_contract_template_node(&node))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_template_node_conversion_preserves_v2_interaction_metadata() {
        let data = ViewTemplateNodeData {
            node_id: "asset/search".into(),
            control_id: "SearchEdited".into(),
            role: "InputField".into(),
            component_role: "input-field".into(),
            value_text: "albedo".into(),
            dispatch_kind: "asset".into(),
            binding_id: "AssetSurface/SearchEdited".into(),
            edit_action_id: "AssetSurface/SearchEdited".into(),
            commit_action_id: "AssetSurface/SearchCommitted".into(),
            ..ViewTemplateNodeData::default()
        };

        let node = to_host_contract_template_node(&data);

        assert_eq!(node.component_role.as_str(), "input-field");
        assert_eq!(node.value_text.as_str(), "albedo");
        assert_eq!(node.dispatch_kind.as_str(), "asset");
        assert_eq!(node.binding_id.as_str(), "AssetSurface/SearchEdited");
        assert_eq!(node.edit_action_id.as_str(), "AssetSurface/SearchEdited");
        assert_eq!(
            node.commit_action_id.as_str(),
            "AssetSurface/SearchCommitted"
        );
    }
}
