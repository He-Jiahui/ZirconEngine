use crate::ui::retained_host::primitives::SharedString;

#[derive(Clone, Default)]
pub(crate) struct WorkbenchContextMenuRequestData {
    pub target_control_id: SharedString,
    pub target_action_id: SharedString,
    pub target_dispatch_kind: SharedString,
    pub target_role: SharedString,
    pub target_value_text: SharedString,
    pub target_path: SharedString,
    pub popup_anchor_x: f32,
    pub popup_anchor_y: f32,
    pub menu_items: Vec<SharedString>,
}
