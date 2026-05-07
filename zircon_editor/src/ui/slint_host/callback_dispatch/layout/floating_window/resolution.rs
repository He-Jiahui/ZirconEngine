use crate::ui::workbench::model::FloatingWindowModel;
use crate::ui::workbench::view::ViewInstanceId;

pub(super) fn resolve_floating_window_focus_instance(
    window: &FloatingWindowModel,
) -> Option<ViewInstanceId> {
    window.focus_target_instance().cloned()
}

pub(super) fn resolve_floating_window_close_instances(
    window: &FloatingWindowModel,
) -> Option<Vec<ViewInstanceId>> {
    if window.tabs.is_empty() || window.tabs.iter().any(|tab| !tab.closeable) {
        return None;
    }
    Some(
        window
            .tabs
            .iter()
            .map(|tab| tab.instance_id.clone())
            .collect(),
    )
}
