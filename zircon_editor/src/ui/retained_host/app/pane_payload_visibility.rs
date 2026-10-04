use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::ViewContentKind;

pub(super) fn should_collect_payload_for_kind(
    model: &WorkbenchViewModel,
    kind: ViewContentKind,
) -> bool {
    payload_visibility_for_pair(model, kind, kind).0
}

pub(super) fn payload_visibility_for_pair(
    model: &WorkbenchViewModel,
    first: ViewContentKind,
    second: ViewContentKind,
) -> (bool, bool) {
    let mut visible = (false, false);

    for tab in &model.document_tabs {
        if tab.active {
            record_visible_kind(&mut visible, tab.content_kind, first, second);
        }
        if visible.0 && visible.1 {
            return visible;
        }
    }

    for stack in model.tool_windows.values() {
        if !stack.visible {
            continue;
        }
        for tab in &stack.tabs {
            if tab.active || stack.active_tab.as_ref() == Some(&tab.instance_id) {
                record_visible_kind(&mut visible, tab.content_kind, first, second);
            }
            if visible.0 && visible.1 {
                return visible;
            }
        }
    }

    for window in &model.floating_windows {
        for tab in &window.tabs {
            if tab.active {
                record_visible_kind(&mut visible, tab.content_kind, first, second);
            }
            if visible.0 && visible.1 {
                return visible;
            }
        }
    }

    visible
}

#[inline]
fn record_visible_kind(
    visible: &mut (bool, bool),
    kind: ViewContentKind,
    first: ViewContentKind,
    second: ViewContentKind,
) {
    visible.0 |= kind == first;
    visible.1 |= kind == second;
}

#[cfg(test)]
#[path = "tests/pane_payload_visibility_paired_visibility_tests.rs"]
mod paired_visibility_tests;
