use crate::core::editor_event::EditorEventRuntime;
use crate::ui::slint_host::event_bridge::SlintDispatchEffects;
use crate::ui::workbench::layout::{LayoutCommand, MainPageId};
use crate::ui::workbench::model::WorkbenchViewModel;

use super::super::dispatch_layout_command;
use super::resolution::resolve_floating_window_focus_instance;

pub(crate) fn dispatch_builtin_floating_window_focus(
    runtime: &EditorEventRuntime,
    window_id: &MainPageId,
) -> Option<Result<SlintDispatchEffects, String>> {
    let chrome = runtime.chrome_snapshot();
    let model = WorkbenchViewModel::build(&chrome);
    let window = model
        .floating_windows
        .iter()
        .find(|window| &window.window_id == window_id)?;
    let instance_id = resolve_floating_window_focus_instance(window)?;
    Some(dispatch_layout_command(
        runtime,
        LayoutCommand::FocusView { instance_id },
    ))
}

pub(crate) fn dispatch_builtin_floating_window_focus_for_source(
    runtime: &EditorEventRuntime,
    source_window_id: Option<&MainPageId>,
    last_focused_window_id: Option<&MainPageId>,
) -> Option<Result<SlintDispatchEffects, String>> {
    let window_id = source_window_id?;
    if Some(window_id) == last_focused_window_id {
        return None;
    }
    dispatch_builtin_floating_window_focus(runtime, window_id)
}
