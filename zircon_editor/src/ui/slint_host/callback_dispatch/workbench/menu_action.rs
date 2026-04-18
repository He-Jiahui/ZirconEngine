use crate::core::editor_event::{host_adapter, EditorEventRuntime};
use crate::ui::slint_host::event_bridge::SlintDispatchEffects;
use crate::LayoutCommand;

use super::super::common::dispatch_envelope;
use super::control::dispatch_builtin_workbench_menu_action;

pub(crate) fn dispatch_menu_action(
    runtime: &EditorEventRuntime,
    action: &str,
) -> Result<SlintDispatchEffects, String> {
    if let Some(name) = action.strip_prefix("SavePreset.") {
        let name = if name.is_empty() { "current" } else { name };
        return super::super::dispatch_layout_command(
            runtime,
            LayoutCommand::SavePreset {
                name: name.to_string(),
            },
        );
    }

    if let Some(name) = action.strip_prefix("LoadPreset.") {
        let name = if name.is_empty() { "current" } else { name };
        return super::super::dispatch_layout_command(
            runtime,
            LayoutCommand::LoadPreset {
                name: name.to_string(),
            },
        );
    }

    dispatch_envelope(runtime, host_adapter::slint_menu_action(action)?)
}

pub(crate) fn dispatch_workbench_menu_action_with_template_fallback(
    runtime: &EditorEventRuntime,
    bridge: &super::super::BuiltinWorkbenchTemplateBridge,
    action: &str,
) -> Result<SlintDispatchEffects, String> {
    if let Some(result) = dispatch_builtin_workbench_menu_action(runtime, bridge, action) {
        return result;
    }
    dispatch_menu_action(runtime, action)
}
