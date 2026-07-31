use crate::ui::retained_host::primitives::SharedString;

use super::super::super::data::FrameRect;
use super::super::super::template_component_family::TemplateComponentFamily;

#[derive(Clone)]
pub(crate) struct TemplateNodePointerHit {
    pub(crate) pane_id: SharedString,
    pub(crate) control_id: SharedString,
    pub(crate) action_id: SharedString,
    pub(crate) binding_id: SharedString,
    pub(crate) dispatch_kind: SharedString,
    pub(crate) component_role: SharedString,
    pub(crate) component_family: Option<TemplateComponentFamily>,
    pub(crate) value_text: SharedString,
    pub(crate) edit_action_id: SharedString,
    pub(crate) commit_action_id: SharedString,
    pub(crate) disabled: bool,
    pub(crate) frame: FrameRect,
    pub(crate) table_row_source_index: Option<i32>,
    pub(crate) table_row_identity_kind: SharedString,
    pub(crate) table_row_identity_text: SharedString,
}
