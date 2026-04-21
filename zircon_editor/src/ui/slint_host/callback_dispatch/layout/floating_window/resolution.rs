use crate::core::editor_event::EditorEventRuntime;
use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::model::{FloatingWindowModel, WorkbenchViewModel};
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

pub(crate) fn resolve_builtin_floating_window_close_instances(
    runtime: &EditorEventRuntime,
    window_id: &MainPageId,
) -> Option<Vec<ViewInstanceId>> {
    let chrome = runtime.chrome_snapshot();
    let model = WorkbenchViewModel::build(&chrome);
    let window = model
        .floating_windows
        .iter()
        .find(|window| &window.window_id == window_id)?;
    resolve_floating_window_close_instances(window)
}
