use std::collections::BTreeMap;

use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};

use super::types::{ExtensionBindingEventKind, ExtensionBindingSpec};

const WORKBENCH_EXTENSION_VIEW_ID: &str = "WorkbenchExtension";

pub(super) fn insert_workbench_extension_bindings(
    bindings: &mut BTreeMap<String, EditorUiBinding>,
    specs: &[ExtensionBindingSpec],
) {
    for spec in specs {
        insert_event(
            bindings,
            WORKBENCH_EXTENSION_VIEW_ID,
            spec.control_id,
            editor_event_kind(spec.event_kind),
            EditorUiBindingPayload::menu_action(spec.action_id),
        );
    }
}

fn editor_event_kind(event_kind: ExtensionBindingEventKind) -> EditorUiEventKind {
    match event_kind {
        ExtensionBindingEventKind::Click => EditorUiEventKind::Click,
        ExtensionBindingEventKind::Change => EditorUiEventKind::Change,
        ExtensionBindingEventKind::Submit => EditorUiEventKind::Submit,
    }
}

fn insert_event(
    bindings: &mut BTreeMap<String, EditorUiBinding>,
    view_id: &str,
    control_id: &str,
    event_kind: EditorUiEventKind,
    payload: EditorUiBindingPayload,
) {
    bindings.insert(
        format!("{view_id}/{control_id}"),
        EditorUiBinding::new(view_id, control_id, event_kind, payload),
    );
}
