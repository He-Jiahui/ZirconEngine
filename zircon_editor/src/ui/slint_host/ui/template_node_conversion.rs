use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};
use crate::ui::slint_host as host_contract;
use slint::{Model, ModelRc};

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
        component_role: "".into(),
        value_text: "".into(),
        value_number: 0.0,
        value_percent: 0.0,
        value_color: slint::Color::from_argb_u8(0, 0, 0, 0),
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
        selected: false,
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
        focused: false,
        hovered: false,
        pressed: false,
        dragging: false,
        drop_hovered: false,
        active_drag_target: false,
        disabled: false,
        dispatch_kind: data.dispatch_kind.clone(),
        action_id: data.action_id.clone(),
        binding_id: "".into(),
        begin_drag_action_id: "".into(),
        drag_action_id: "".into(),
        end_drag_action_id: "".into(),
        commit_action_id: "".into(),
        edit_action_id: "".into(),
        surface_variant: data.surface_variant.clone(),
        text_tone: data.text_tone.clone(),
        button_variant: data.button_variant.clone(),
        font_size: data.font_size,
        font_weight: data.font_weight,
        text_align: data.text_align.clone(),
        overflow: data.overflow.clone(),
        corner_radius: data.corner_radius,
        border_width: data.border_width,
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

pub(crate) fn to_host_contract_template_nodes_owned(
    items: Vec<ViewTemplateNodeData>,
) -> ModelRc<host_contract::TemplatePaneNodeData> {
    model_rc(
        items
            .into_iter()
            .map(to_host_contract_template_node_owned)
            .collect(),
    )
}
