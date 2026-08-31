use crate::ui::retained_host::primitives::SharedString;
use zircon_runtime_interface::ui::event_ui::UiNodeId;

use super::super::super::data::FrameRect;
use super::super::super::template_component_family::TemplateComponentFamily;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TemplateNodePointerMoveKind {
    Node,
    Option,
    MenuItem,
    TextInput,
    KeySelector,
}

impl TemplateNodePointerMoveKind {
    pub(crate) fn dispatch_kind(self) -> &'static str {
        match self {
            Self::Node => "",
            Self::Option => "workbench_option",
            Self::MenuItem => "workbench_menu_item",
            Self::TextInput => "commit_only",
            Self::KeySelector => "chord_capture",
        }
    }

    pub(crate) fn is_popup(self) -> bool {
        self != Self::Node
    }
}

/// A non-owning semantic view whose strings remain valid for its presentation generation.
pub(crate) struct TemplateNodePointerMoveHit<'a> {
    pub(crate) surface_node_id: Option<UiNodeId>,
    pub(crate) dispatchable: bool,
    pub(crate) control_id: &'a str,
    pub(crate) action_id: &'a str,
    pub(crate) value_text: &'a str,
    pub(crate) kind: TemplateNodePointerMoveKind,
    pub(crate) frame: FrameRect,
}

/// A full pane-route semantic view borrowed from one presentation generation.
pub(crate) struct TemplateNodePointerRouteHit<'a> {
    pub(crate) pane_id: &'a str,
    pub(crate) control_id: &'a str,
    pub(crate) action_id: &'a str,
    pub(crate) binding_id: &'a str,
    pub(crate) dispatch_kind: &'a str,
    pub(crate) component_role: &'a str,
    pub(crate) component_family: Option<TemplateComponentFamily>,
    pub(crate) value_text: &'a str,
    pub(crate) edit_action_id: &'a str,
    pub(crate) commit_action_id: &'a str,
    pub(crate) disabled: bool,
    pub(crate) frame: FrameRect,
    pub(crate) table_row_source_index: Option<i32>,
    pub(crate) table_row_identity_kind: &'a str,
    pub(crate) table_row_identity_text: &'a str,
    pub(crate) kind: TemplateNodePointerMoveKind,
}

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
