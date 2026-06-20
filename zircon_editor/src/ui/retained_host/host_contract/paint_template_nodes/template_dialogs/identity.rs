use super::super::super::data::TemplatePaneNodeData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum DialogKind {
    Dialog,
    ConfirmDialog,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum DialogPaintState {
    NotDialog,
    Closed,
    Open(DialogKind),
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dialog_paint_state(
    node: &TemplatePaneNodeData,
) -> DialogPaintState {
    let Some(kind) = dialog_kind(node) else {
        return DialogPaintState::NotDialog;
    };
    if node.popup_open {
        DialogPaintState::Open(kind)
    } else {
        DialogPaintState::Closed
    }
}

fn dialog_kind(node: &TemplatePaneNodeData) -> Option<DialogKind> {
    match (node.role.as_str(), node.component_role.as_str()) {
        ("Dialog", _) | (_, "dialog") => Some(DialogKind::Dialog),
        ("ConfirmDialog", _) | (_, "confirm-dialog") | ("AlertDialog", _) | (_, "alert-dialog") => {
            Some(DialogKind::ConfirmDialog)
        }
        _ => None,
    }
}
