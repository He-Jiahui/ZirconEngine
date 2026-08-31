use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::event_bridge::UiHostEventEffects;
use crate::ui::workbench::layout::{LayoutCommand, MainPageId};

use super::super::dispatch_layout_command;
use super::resolution::resolve_floating_window_focus_instance;

pub(crate) fn dispatch_builtin_floating_window_focus(
    runtime: &EditorHostEventController,
    window_id: &MainPageId,
) -> Option<Result<UiHostEventEffects, String>> {
    let chrome = runtime.chrome_snapshot();
    let context = runtime.project_command_eval_snapshot(&chrome);
    let model = runtime.build_workbench_view_model(&chrome, &context);
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
    runtime: &EditorHostEventController,
    source_window_id: Option<&MainPageId>,
    last_focused_window_id: Option<&MainPageId>,
) -> Option<Result<UiHostEventEffects, String>> {
    let window_id = source_window_id?;
    if Some(window_id) == last_focused_window_id {
        return None;
    }
    dispatch_builtin_floating_window_focus(runtime, window_id)
}
