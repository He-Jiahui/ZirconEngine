use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};
use crate::ui::retained_host as host_contract;
use crate::ui::retained_host::primitives::{ModelRc, SharedString};

fn map_model_rc<T, U, F>(model: &ModelRc<T>, mut map: F) -> ModelRc<U>
where
    T: Clone + 'static,
    U: Clone + 'static,
    F: FnMut(&T) -> U,
{
    model_rc(model.iter().map(&mut map).collect())
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
        parent_node_id: SharedString::default(),
        control_id: data.control_id.clone(),
        role: data.role.clone(),
        text: data.text.clone(),
        label_text: "".into(),
        label_color: crate::ui::retained_host::primitives::Color::from_argb_u8(0, 0, 0, 0),
        label_brightness: 1.0,
        layout_offset_x: 0.0,
        layout_offset_y: 0.0,
        layout_icon_size: 0.0,
        layout_content_offset_x: 0.0,
        layout_content_offset_y: 0.0,
        layout_first_cell_offset_x: 0.0,
        layout_second_cell_offset_x: 0.0,
        layout_third_cell_offset_x: 0.0,
        layout_fourth_cell_offset_x: 0.0,
        component_role: data.component_role.clone(),
        component_category: "".into(),
        component_layout_role: "".into(),
        component_variant: data.component_variant.clone(),
        value_text: data.value_text.clone(),
        value_number: data.value_number,
        value_percent: data.value_percent,
        value_color: crate::ui::retained_host::primitives::Color::from_argb_u8(0, 0, 0, 0),
        icon_color: crate::ui::retained_host::primitives::Color::from_argb_u8(0, 0, 0, 0),
        icon_stroke_width: 0.0,
        has_selected_segment_border_width: false,
        selected_segment_border_width: 0.0,
        selected_segment_underline_height: 0.0,
        selected_segment_underline_color: crate::ui::retained_host::primitives::Color::from_argb_u8(
            0, 0, 0, 0,
        ),
        media_source: data.media_source.clone(),
        icon_name: data.icon_name.clone(),
        has_preview_image: data.has_preview_image,
        preview_image: data.preview_image.clone(),
        vector_components: ModelRc::default(),
        sample_grid: host_contract::TemplatePaneSampleGridData::default(),
        timeline_strip: host_contract::TemplatePaneTimelineStripData::default(),
        weight_heatmap: host_contract::TemplatePaneWeightHeatmapData::default(),
        validation_level: "".into(),
        validation_message: "".into(),
        popup_open: data.popup_open,
        has_popup_anchor: false,
        popup_anchor_x: 0.0,
        popup_anchor_y: 0.0,
        selection_state: "".into(),
        search_query: "".into(),
        selected: data.selected,
        tree_depth: 0,
        tree_indent_px: 0.0,
        options_text: options_text(&data.options).into(),
        options: data.options.clone(),
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
        transition_kind: data.transition_kind.clone(),
        transition_in: data.transition_in,
        transition_entered: data.transition_entered,
        transition_progress: data.transition_progress,
        transition_duration_ms: data.transition_duration_ms,
        transition_easing: data.transition_easing.clone(),
        transition_direction: data.transition_direction.clone(),
        drop_hovered: false,
        active_drag_target: false,
        drag_payload_kind: "".into(),
        drag_payload_label: "".into(),
        drag_payload_reference: "".into(),
        has_drag_cursor: false,
        drag_cursor_x: 0.0,
        drag_cursor_y: 0.0,
        drag_offset_x: 0.0,
        drag_offset_y: 0.0,
        drag_preview_width: 0.0,
        drag_preview_height: 0.0,
        drop_allowed: false,
        has_drop_target: false,
        drop_target_x: 0.0,
        drop_target_y: 0.0,
        drop_target_width: 0.0,
        drop_target_height: 0.0,
        drop_indicator_edge: "".into(),
        drop_indicator_text: "".into(),
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
        z_index: data.z_index,
        has_clip_frame: false,
        clip_frame: host_contract::TemplateNodeFrameData::default(),
        frame: to_host_contract_template_frame(&data.frame),
    }
}

fn options_text(options: &ModelRc<SharedString>) -> String {
    let mut text = String::new();
    for option in options.iter() {
        if !text.is_empty() {
            text.push_str(", ");
        }
        text.push_str(option);
    }
    text
}

pub(crate) fn to_host_contract_template_node_owned(
    data: ViewTemplateNodeData,
) -> host_contract::TemplatePaneNodeData {
    to_host_contract_template_node(&data)
}

pub(crate) fn to_host_contract_template_nodes(
    data: &ModelRc<ViewTemplateNodeData>,
) -> ModelRc<host_contract::TemplatePaneNodeData> {
    map_model_rc(data, to_host_contract_template_node)
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    use super::*;

    struct CloneProbe(Arc<AtomicUsize>);

    impl Clone for CloneProbe {
        fn clone(&self) -> Self {
            self.0.fetch_add(1, Ordering::Relaxed);
            Self(Arc::clone(&self.0))
        }
    }

    #[test]
    fn model_mapping_borrows_source_rows() {
        let clone_count = Arc::new(AtomicUsize::new(0));
        let source = model_rc(vec![CloneProbe(Arc::clone(&clone_count))]);

        let mapped = map_model_rc(&source, |_| 7_u8);

        assert_eq!(mapped.row_data(0), Some(7));
        assert_eq!(clone_count.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn view_template_node_conversion_preserves_v2_interaction_metadata() {
        let data = ViewTemplateNodeData {
            node_id: "asset/search".into(),
            control_id: "SearchEdited".into(),
            role: "InputField".into(),
            component_role: "input-field".into(),
            component_variant: "outlined".into(),
            value_text: "albedo".into(),
            value_number: 42.0,
            value_percent: 0.42,
            options: model_rc(vec![
                "Name".into(),
                "Type".into(),
                "Size".into(),
                "Rev".into(),
            ]),
            z_index: 17,
            transition_kind: "fade".into(),
            transition_in: true,
            transition_entered: false,
            transition_progress: 0.5,
            transition_duration_ms: 225,
            transition_easing: "cubic-bezier(0.4, 0, 0.2, 1)".into(),
            popup_open: true,
            dispatch_kind: "asset".into(),
            binding_id: "AssetSurface/SearchEdited".into(),
            edit_action_id: "workbench.asset.search.edit".into(),
            commit_action_id: "workbench.asset.search.commit".into(),
            ..ViewTemplateNodeData::default()
        };

        let node = to_host_contract_template_node(&data);

        assert_eq!(node.component_role.as_str(), "input-field");
        assert_eq!(node.component_variant.as_str(), "outlined");
        assert_eq!(node.value_text.as_str(), "albedo");
        assert_eq!(node.value_number, 42.0);
        assert_eq!(node.value_percent, 0.42);
        assert_eq!(node.options_text.as_str(), "Name, Type, Size, Rev");
        assert_eq!(node.options.row_count(), 4);
        assert_eq!(node.options.row_data(0).as_deref(), Some("Name"));
        assert_eq!(node.options.row_data(3).as_deref(), Some("Rev"));
        assert_eq!(node.z_index, 17);
        assert_eq!(node.transition_kind.as_str(), "fade");
        assert!(node.transition_in);
        assert!(!node.transition_entered);
        assert_eq!(node.transition_progress, 0.5);
        assert_eq!(node.transition_duration_ms, 225);
        assert_eq!(
            node.transition_easing.as_str(),
            "cubic-bezier(0.4, 0, 0.2, 1)"
        );
        assert!(node.popup_open);
        assert_eq!(node.dispatch_kind.as_str(), "asset");
        assert_eq!(node.binding_id.as_str(), "AssetSurface/SearchEdited");
        assert_eq!(node.edit_action_id.as_str(), "workbench.asset.search.edit");
        assert_eq!(
            node.commit_action_id.as_str(),
            "workbench.asset.search.commit"
        );
    }
}
