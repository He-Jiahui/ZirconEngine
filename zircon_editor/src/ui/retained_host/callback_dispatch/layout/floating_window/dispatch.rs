use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::event_bridge::UiHostEventEffects;
use crate::ui::workbench::layout::{LayoutCommand, MainPageId};

use super::super::dispatch_layout_command;

pub(crate) fn dispatch_builtin_floating_window_focus(
    runtime: &EditorHostEventController,
    window_id: &MainPageId,
) -> Option<Result<UiHostEventEffects, String>> {
    let instance_id = runtime.floating_window_focus_target(window_id)?;
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
