use crate::ui::retained_host as host_contract;

pub(in super::super) struct ProjectedPopupActions {
    pub(in super::super) menu_items: Vec<String>,
    pub(in super::super) structured_menu_items: Vec<host_contract::TemplatePaneMenuItemData>,
    pub(in super::super) popup_open: bool,
    pub(in super::super) has_popup_anchor: bool,
    pub(in super::super) popup_anchor_x: f32,
    pub(in super::super) popup_anchor_y: f32,
    pub(in super::super) frame: host_contract::TemplateNodeFrameData,
    pub(in super::super) actions: Vec<host_contract::TemplatePaneActionData>,
    pub(in super::super) accepted_drag_payloads: String,
    pub(in super::super) dispatch_kind: String,
    pub(in super::super) action_id: String,
    pub(in super::super) binding_id: String,
    pub(in super::super) begin_drag_action_id: String,
    pub(in super::super) drag_action_id: String,
    pub(in super::super) end_drag_action_id: String,
    pub(in super::super) commit_action_id: String,
    pub(in super::super) edit_action_id: String,
}
