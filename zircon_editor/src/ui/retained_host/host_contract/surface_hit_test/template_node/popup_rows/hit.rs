use crate::ui::retained_host::asset_control_ids::asset_dispatch_source;
use crate::ui::retained_host::primitives::SharedString;

use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::template_component_family::{
    template_component_family, TemplateComponentFamily,
};
use super::super::{
    TemplateNodePointerHit, TemplateNodePointerMoveHit, TemplateNodePointerMoveKind,
    TemplateNodePointerRouteHit,
};

pub(in crate::ui::retained_host::host_contract) enum TemplatePopupRowHit {
    Hit(TemplateNodePointerHit),
    Blocked,
}

pub(in crate::ui::retained_host::host_contract) enum TemplatePopupRowMoveHit<'a> {
    Hit(TemplateNodePointerMoveHit<'a>),
    Blocked,
}

pub(in crate::ui::retained_host::host_contract) enum TemplatePopupRowRouteHit<'a> {
    Hit(TemplateNodePointerRouteHit<'a>),
    Blocked,
}

pub(in crate::ui::retained_host::host_contract::surface_hit_test::template_node) enum TemplatePopupRowTarget<
    'a,
> {
    Hit {
        kind: TemplateNodePointerMoveKind,
        action_id: &'a str,
        value_text: &'a str,
        frame: FrameRect,
    },
    TextInput {
        control_id: &'a str,
        edit_action_id: &'a str,
        commit_action_id: &'a str,
        value_text: &'a str,
        frame: FrameRect,
    },
    ChordInput {
        control_id: &'a str,
        capture_action_id: &'a str,
        commit_action_id: &'a str,
        value_text: &'a str,
        frame: FrameRect,
    },
    Blocked,
}

impl<'a> TemplatePopupRowTarget<'a> {
    pub(in crate::ui::retained_host::host_contract::surface_hit_test::template_node) fn into_pointer_hit(
        self,
        node: &TemplatePaneNodeData,
    ) -> TemplatePopupRowHit {
        match self {
            Self::Hit {
                kind,
                action_id,
                value_text,
                frame,
            } => {
                let action_id = if kind == TemplateNodePointerMoveKind::MenuItem {
                    super::action_id::normalized_menu_row_action_id(action_id, value_text)
                } else {
                    action_id.into()
                };
                TemplatePopupRowHit::Hit(template_popup_row_hit(
                    node,
                    frame,
                    popup_row_dispatch_kind(node, kind),
                    action_id,
                    value_text.into(),
                ))
            }
            Self::TextInput {
                control_id,
                edit_action_id,
                commit_action_id,
                value_text,
                frame,
            } => TemplatePopupRowHit::Hit(template_popup_text_input_hit(
                frame,
                control_id,
                edit_action_id,
                commit_action_id,
                value_text,
            )),
            Self::ChordInput {
                control_id,
                capture_action_id,
                commit_action_id,
                value_text,
                frame,
            } => TemplatePopupRowHit::Hit(template_popup_chord_input_hit(
                frame,
                control_id,
                capture_action_id,
                commit_action_id,
                value_text,
            )),
            Self::Blocked => TemplatePopupRowHit::Blocked,
        }
    }

    pub(in crate::ui::retained_host::host_contract::surface_hit_test::template_node) fn into_pointer_move_hit(
        self,
        node: &'a TemplatePaneNodeData,
    ) -> TemplatePopupRowMoveHit<'a> {
        match self {
            Self::Hit {
                kind,
                action_id,
                value_text,
                frame,
            } => TemplatePopupRowMoveHit::Hit(TemplateNodePointerMoveHit {
                surface_node_id: node.surface_node_id,
                dispatchable: true,
                control_id: node.control_id.as_str(),
                action_id,
                value_text,
                kind,
                frame,
            }),
            Self::TextInput {
                control_id,
                edit_action_id,
                commit_action_id: _,
                value_text,
                frame,
            } => TemplatePopupRowMoveHit::Hit(TemplateNodePointerMoveHit {
                surface_node_id: node.surface_node_id,
                dispatchable: true,
                control_id,
                action_id: edit_action_id,
                value_text,
                kind: TemplateNodePointerMoveKind::TextInput,
                frame,
            }),
            Self::ChordInput {
                control_id,
                capture_action_id,
                commit_action_id: _,
                value_text,
                frame,
            } => TemplatePopupRowMoveHit::Hit(TemplateNodePointerMoveHit {
                surface_node_id: node.surface_node_id,
                dispatchable: true,
                control_id,
                action_id: capture_action_id,
                value_text,
                kind: TemplateNodePointerMoveKind::KeySelector,
                frame,
            }),
            Self::Blocked => TemplatePopupRowMoveHit::Blocked,
        }
    }

    pub(in crate::ui::retained_host::host_contract::surface_hit_test::template_node) fn into_pointer_route_hit(
        self,
        node: &'a TemplatePaneNodeData,
    ) -> TemplatePopupRowRouteHit<'a> {
        match self {
            Self::Hit {
                kind,
                action_id,
                value_text,
                frame,
            } => TemplatePopupRowRouteHit::Hit(TemplateNodePointerRouteHit {
                pane_id: "",
                control_id: node.control_id.as_str(),
                action_id,
                binding_id: "",
                dispatch_kind: popup_row_dispatch_kind(node, kind),
                component_role: node.component_role.as_str(),
                component_family: template_component_family(node),
                value_text,
                edit_action_id: node.edit_action_id.as_str(),
                commit_action_id: node.commit_action_id.as_str(),
                disabled: node.disabled,
                frame,
                table_row_source_index: None,
                table_row_identity_kind: "",
                table_row_identity_text: "",
                kind,
            }),
            Self::TextInput {
                control_id,
                edit_action_id,
                commit_action_id,
                value_text,
                frame,
            } => TemplatePopupRowRouteHit::Hit(TemplateNodePointerRouteHit {
                pane_id: "",
                control_id,
                action_id: "",
                binding_id: "",
                dispatch_kind: "commit_only",
                component_role: "input-field",
                component_family: Some(TemplateComponentFamily::TextInput),
                value_text,
                edit_action_id,
                commit_action_id,
                disabled: node.disabled,
                frame,
                table_row_source_index: None,
                table_row_identity_kind: "",
                table_row_identity_text: "",
                kind: TemplateNodePointerMoveKind::TextInput,
            }),
            Self::ChordInput {
                control_id,
                capture_action_id,
                commit_action_id,
                value_text,
                frame,
            } => TemplatePopupRowRouteHit::Hit(TemplateNodePointerRouteHit {
                pane_id: "",
                control_id,
                action_id: "",
                binding_id: "",
                dispatch_kind: "chord_capture",
                component_role: "key-selector",
                component_family: Some(TemplateComponentFamily::KeySelector),
                value_text,
                edit_action_id: capture_action_id,
                commit_action_id,
                disabled: node.disabled,
                frame,
                table_row_source_index: None,
                table_row_identity_kind: "",
                table_row_identity_text: "",
                kind: TemplateNodePointerMoveKind::KeySelector,
            }),
            Self::Blocked => TemplatePopupRowRouteHit::Blocked,
        }
    }
}

fn popup_row_dispatch_kind(node: &TemplatePaneNodeData, kind: TemplateNodePointerMoveKind) -> &str {
    if kind == TemplateNodePointerMoveKind::Option
        && asset_dispatch_source(node.dispatch_kind.as_str()).is_some()
    {
        node.dispatch_kind.as_str()
    } else {
        kind.dispatch_kind()
    }
}

pub(super) fn template_popup_row_hit(
    node: &TemplatePaneNodeData,
    frame: FrameRect,
    dispatch_kind: &str,
    action_id: SharedString,
    value_text: SharedString,
) -> TemplateNodePointerHit {
    TemplateNodePointerHit {
        pane_id: SharedString::new(),
        control_id: node.control_id.clone(),
        action_id,
        binding_id: String::new(),
        dispatch_kind: dispatch_kind.to_string(),
        component_role: node.component_role.clone(),
        component_family: template_component_family(node),
        value_text,
        edit_action_id: node.edit_action_id.clone(),
        commit_action_id: node.commit_action_id.clone(),
        disabled: node.disabled,
        frame,
        table_row_source_index: None,
        table_row_identity_kind: SharedString::new(),
        table_row_identity_text: SharedString::new(),
    }
}

fn template_popup_text_input_hit(
    frame: FrameRect,
    control_id: &str,
    edit_action_id: &str,
    commit_action_id: &str,
    value_text: &str,
) -> TemplateNodePointerHit {
    TemplateNodePointerHit {
        pane_id: SharedString::new(),
        control_id: control_id.into(),
        action_id: SharedString::new(),
        binding_id: SharedString::new(),
        dispatch_kind: "commit_only".into(),
        component_role: "input-field".into(),
        component_family: Some(TemplateComponentFamily::TextInput),
        value_text: value_text.into(),
        edit_action_id: edit_action_id.into(),
        commit_action_id: commit_action_id.into(),
        disabled: false,
        frame,
        table_row_source_index: None,
        table_row_identity_kind: SharedString::new(),
        table_row_identity_text: SharedString::new(),
    }
}

fn template_popup_chord_input_hit(
    frame: FrameRect,
    control_id: &str,
    capture_action_id: &str,
    commit_action_id: &str,
    value_text: &str,
) -> TemplateNodePointerHit {
    TemplateNodePointerHit {
        pane_id: SharedString::new(),
        control_id: control_id.into(),
        action_id: SharedString::new(),
        binding_id: SharedString::new(),
        dispatch_kind: "chord_capture".into(),
        component_role: "key-selector".into(),
        component_family: Some(TemplateComponentFamily::KeySelector),
        value_text: value_text.into(),
        edit_action_id: capture_action_id.into(),
        commit_action_id: commit_action_id.into(),
        disabled: false,
        frame,
        table_row_source_index: None,
        table_row_identity_kind: SharedString::new(),
        table_row_identity_text: SharedString::new(),
    }
}
