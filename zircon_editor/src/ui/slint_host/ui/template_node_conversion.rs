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
        media_source: "".into(),
        icon_name: "".into(),
        has_preview_image: false,
        preview_image: slint::Image::default(),
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
